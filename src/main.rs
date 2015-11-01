#![feature(core)]
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
use sharebill::{TransactionDocument, SharebillBalances};
use http_helper::{get_url, get_json};
use rational::Rational;
use rustc_serialize::json;

extern crate iron;
extern crate router;
use iron::prelude::*;
use iron::status;
use iron::mime::Mime;
use router::Router;


fn monitor_changes(
	view: SharebillBalances,
	changes_url: String,
	doc_root: String,
	poll_timeout: Option<u32>,
	balances_lock: Arc<Mutex<BTreeMap<String, Rational>>>,
	initial_update_seq: u32
) {
	let mut update_seq = initial_update_seq;

	loop {
		let timeout_section =
			if let Some(timeout) = poll_timeout {
				format!("&timeout={}", timeout)
			} else {
				"".to_string()
			};
		let poll_url = format!("{}?feed=longpoll&since={}{}", changes_url, update_seq, timeout_section);

		let changes: Changes = get_json(&poll_url).unwrap();
		if changes.results.len() > 0 {
			println!("{:?}", changes);

			let mut balances = balances_lock.lock().unwrap();
			let mut key_value_list = BTreeMap::<String, Vec<Rational>>::new();
			for (key, value) in &*balances {
				key_value_list.insert(key.clone(), vec![(*value).clone()]);
			}

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
					let remove_doc: TransactionDocument = get_json(&remove_doc_url).unwrap();
					view.unmap(&remove_doc, |key, value| {
						let mut v = match key_value_list.entry(key.clone()) {
							Entry::Occupied(o) => o.into_mut(),
							Entry::Vacant(v) => v.insert(Vec::<Rational>::new())
						};
						v.push(value.clone());
					});
				}

				if change.deleted != Some(true) {
					let add_doc_url = format!("{}{}?rev={}", doc_root, change.id, revision.rev);
					let add_doc: TransactionDocument = get_json(&add_doc_url).unwrap();
					view.map(&add_doc, |key, value| {
						let mut v = match key_value_list.entry(key.clone()) {
							Entry::Occupied(o) => o.into_mut(),
							Entry::Vacant(v) => v.insert(Vec::<Rational>::new())
						};
						v.push(value.clone());
					});
				}
			}

			for (key, values) in &key_value_list {
				let value = view.reduce(&key, &values);
				match balances.entry(key.clone()) {
					Entry::Occupied(mut o) => { o.insert(value); },
					Entry::Vacant(v) => { v.insert(value); }
				};
			}
		}

		update_seq = changes.last_seq;
	}
}

fn monitor_changes_starter(
	view: SharebillBalances,
	changes_url: String,
	doc_root: String,
	poll_timeout: Option<u32>,
	balances_lock: Arc<Mutex<BTreeMap<String, Rational>>>,
	initial_update_seq: u32
) {
	thread::spawn(move || {
		monitor_changes(view, changes_url, doc_root, poll_timeout, balances_lock, initial_update_seq);
	});
}

fn serve_balances(_: &mut Request, balances_lock: Arc<Mutex<BTreeMap<String, Rational>>>) -> IronResult<Response> {
	let application_json = "application/json".parse::<Mime>().unwrap();
	let balances = balances_lock.lock().unwrap();

	let mut balances_list = ReducedView::<Rational> { rows: vec!() };
	for (account, balance) in &*balances {
		let row = Row::<Rational> {
			key: account.clone(),
			value: Rational(balance.0.clone()),
		};
		balances_list.rows.push(row);
	}

	Ok(Response::with((application_json, status::Ok, json::encode(&balances_list).unwrap())))
}

fn main() {
	let config = config::Config::from_file("config.json").unwrap();

	println!("Loading initial state from origin server ({})...", &config.urls.view);
	let balances : ReducedViewWithUpdateSeq<Rational> = json::decode(&get_url(&config.urls.view).unwrap()).unwrap();

	let mut balances_map = BTreeMap::<String, Rational>::new();
	for balance in &balances.rows {
		balances_map.insert(balance.key.clone(), Rational(balance.value.0.clone()));
	}

	let shared_balances_map = Arc::new(Mutex::new(balances_map));

	monitor_changes_starter(
		SharebillBalances,
		config.urls.changes,
		config.urls.doc_root,
		config.poll_timeout,
		shared_balances_map.clone(),
		balances.update_seq
	);

	let mut router = Router::new();
	router.get("/", move |r: &mut Request| serve_balances(r, shared_balances_map.clone()));

	println!("Ready at http://localhost:4000");
	Iron::new(router).http("localhost:4000").unwrap();
}
