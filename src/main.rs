#![feature(core)]
#![feature(scoped)]
extern crate rustc_serialize;
extern crate num;

mod couchdb;
mod http_helper;
mod rational;
mod sharebill;
mod config;

use std::sync::{Arc, Mutex};
use std::thread;
use std::collections::BTreeMap;
use std::collections::btree_map::Entry;

use couchdb::{ReducedViewWithUpdateSeq, Changes, RevisionsDocument, ReducedView, Row};
use http_helper::{get_url, get_json};
use rustc_serialize::{json, Encodable, Decodable};

use sharebill::SharebillBalances;

extern crate iron;
extern crate router;
use iron::prelude::*;
use iron::status;
use iron::mime::Mime;
use router::Router;


pub trait View<DocumentType, KeyType, ValueType> where KeyType: Clone, ValueType: Clone {
	fn map<Emit>(&self, doc: &DocumentType, mut emit: &mut Emit)
		where Emit : FnMut(&KeyType, &ValueType);

	fn unmap<Emit>(&self, doc: &DocumentType, mut emit: &mut Emit)
		where Emit : FnMut(&KeyType, &ValueType);

	fn reduce(&self, _key: &KeyType, values: &Vec<ValueType>) -> ValueType;
}

fn monitor_changes<DocumentType, KeyType, ValueType, ViewType>(
	view: ViewType,
	changes_url: &str,
	doc_root: &str,
	poll_timeout: &Option<u32>,
	balances_lock: Arc<Mutex<BTreeMap<KeyType, ValueType>>>,
	initial_update_seq: u32
)
	where
		DocumentType: Decodable,
		KeyType: Clone + Ord,
		ValueType: Clone,
		ViewType: View<DocumentType, KeyType, ValueType>
{
	let mut update_seq = initial_update_seq;

	let timeout_section =
		if let &Some(timeout) = poll_timeout {
			format!("&timeout={}", timeout)
		} else {
			"".to_string()
		};

	loop {
		let poll_url = format!("{}?feed=longpoll&since={}{}", changes_url, update_seq, timeout_section);
		let changes: Changes = get_json(&poll_url).unwrap();

		if changes.results.len() > 0 {
			println!("{:?}", changes);

			let mut balances = balances_lock.lock().unwrap();
			let mut key_value_list = BTreeMap::<KeyType, Vec<ValueType>>::new();
			for (key, value) in &*balances {
				key_value_list.insert(key.clone(), vec![(*value).clone()]);
			}

			{
				let mut emit = |key: &KeyType, value: &ValueType| {
					match key_value_list.entry(key.clone()) {
						Entry::Occupied(o) => { o.into_mut().push(value.clone()); },
						Entry::Vacant(v) => { v.insert(vec![value.clone()]); }
					};
				};

				for change in changes.results {
					assert_eq!(change.changes.len(), 1);
					let revision = &change.changes[0];

					let docurl = format!("{}{}?rev={}&revs=true", doc_root, change.id, revision.rev);
					let doc: RevisionsDocument = get_json(&docurl).unwrap();
					let revisions = doc._revisions;
					let rev_split = revision.rev.split("-").collect::<Vec<&str>>();
					assert_eq!(rev_split.len(), 2);
					let rev_number = rev_split[0].parse::<usize>().unwrap();
					let change_index = revisions.start - rev_number;
					assert_eq!(revisions.ids[change_index], rev_split[1]);

					if change_index + 1 < revisions.ids.len() {
						let prev_rev = format!("{}-{}", revisions.start - (change_index+1), &revisions.ids[change_index + 1]);
						let remove_doc_url = format!("{}{}?rev={}", doc_root, change.id, prev_rev);
						let remove_doc: DocumentType = get_json(&remove_doc_url).unwrap();
						view.unmap(&remove_doc, &mut emit);
					}

					if change.deleted != Some(true) {
						let add_doc_url = format!("{}{}?rev={}", doc_root, change.id, revision.rev);
						let add_doc: DocumentType = get_json(&add_doc_url).unwrap();
						view.map(&add_doc, &mut emit);
					}
				}
			}

			for (key, values) in &key_value_list {
				let value = match values.len() {
					1 => values[0].clone(),
					_ => view.reduce(&key, &values)
				};
				match balances.entry(key.clone()) {
					Entry::Occupied(mut o) => { o.insert(value); },
					Entry::Vacant(v) => { v.insert(value); }
				};
			}
		}

		update_seq = changes.last_seq;
	}
}

fn serve_view<KeyType, ValueType>(_: &mut Request, shared_data: Arc<Mutex<BTreeMap<KeyType, ValueType>>>) -> IronResult<Response>
	where
		KeyType: Clone + Encodable,
		ValueType: Clone + Encodable
{
	let application_json = "application/json".parse::<Mime>().unwrap();
	let data = shared_data.lock().unwrap();

	let mut generated_view = ReducedView::<KeyType, ValueType> { rows: vec!() };
	for (key, value) in &*data {
		generated_view.rows.push(
			Row::<KeyType, ValueType> {
				key: key.clone(),
				value: value.clone(),
			}
		);
	}

	Ok(Response::with((application_json, status::Ok, json::encode(&generated_view).unwrap())))
}

fn cacheviewnut<DocumentType, KeyType, ValueType, ViewType>(
	view: ViewType,
	view_url: &str,
	changes_url: &str,
	doc_root: &str,
	poll_timeout: &Option<u32>,
)
	where
		DocumentType: Decodable,
		KeyType: Send + std::any::Any + Encodable + Decodable + Clone + Ord,
		ValueType: Send + std::any::Any + Encodable + Decodable + Clone,
		ViewType: Send + View<DocumentType, KeyType, ValueType>
{
	println!("Loading initial state from origin server ({})...", &view_url);
	let data: ReducedViewWithUpdateSeq<KeyType, ValueType> = json::decode(&get_url(&view_url).unwrap()).unwrap();

	let mut data_map = BTreeMap::<KeyType, ValueType>::new();
	for row in &data.rows {
		data_map.insert(row.key.clone(), row.value.clone());
	}

	let shared_data_map = Arc::new(Mutex::new(data_map));

	let monitor_thread = {
		let shared_data_map = shared_data_map.clone();
		thread::scoped(move || {
			monitor_changes(
				view,
				&changes_url,
				&doc_root,
				&poll_timeout,
				shared_data_map,
				data.update_seq
			);
		})
	};

	let mut router = Router::new();
	router.get("/", move |r: &mut Request| serve_view(r, shared_data_map.clone()));

	println!("Ready at http://localhost:4000");
	Iron::new(router).http("localhost:4000").unwrap();

	monitor_thread.join();
}

fn main() {
	let config = config::Config::from_file("config.json").unwrap();

	cacheviewnut(
		SharebillBalances,
		&config.urls.view,
		&config.urls.changes,
		&config.urls.doc_root,
		&config.poll_timeout,
	);
}
