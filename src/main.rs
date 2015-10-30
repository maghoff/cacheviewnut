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
use std::collections::HashMap;

use couchdb::{ReducedViewWithUpdateSeq, Changes, TransactionDocument, RevisionsDocument, ReducedView, Row};
use http_helper::{get_url, get_json};
use rational::Rational;
use rustc_serialize::json;

extern crate iron;
extern crate router;
use iron::prelude::*;
use iron::status;
use iron::mime::Mime;
use router::Router;


fn update_balances(balances: &mut HashMap<String, Rational>, update: &HashMap<String, Rational>, multiplier_int: i32) {
	let multiplier = &rational::from_i32(multiplier_int).0;
	let zero = rational::from_i32(0);
	println!("    Update by {} x {:?}", &multiplier, update);

	for (key, value) in update {
		let scaled = value.0.clone() * multiplier;
		let new_value = Rational(balances.get(key).unwrap_or(&zero).0.clone() + scaled);
		balances.insert(key.to_string(), new_value);
	}
	println!("    New balances: {}", json::encode(balances).unwrap());
}

fn monitor_changes(changes_url: String, doc_root: String, balances_lock: Arc<Mutex<HashMap<String, Rational>>>, initial_update_seq: u32) {
	thread::spawn(move || {
		let mut update_seq = initial_update_seq;

		loop {
			let poll_url = format!("{}?feed=longpoll&since={}&timeout=30000", changes_url, update_seq);

			let changes: Changes = get_json(&poll_url).unwrap();
			if changes.results.len() > 0 {
				println!("{:?}", changes);

				let mut balances = balances_lock.lock().unwrap();

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
						update_balances(&mut balances, &remove_doc.transaction.debets, 1);
						update_balances(&mut balances, &remove_doc.transaction.credits, -1);
					}

					if change.deleted != Some(true) {
						let add_doc_url = format!("{}{}?rev={}", doc_root, change.id, revision.rev);
						let add_doc: TransactionDocument = get_json(&add_doc_url).unwrap();
						update_balances(&mut balances, &add_doc.transaction.debets, -1);
						update_balances(&mut balances, &add_doc.transaction.credits, 1);
					}
				}
			}

			update_seq = changes.last_seq;
		}
	});
}

fn serve_balances(_: &mut Request, balances_lock: Arc<Mutex<HashMap<String, Rational>>>) -> IronResult<Response> {
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

	let mut balances_map = HashMap::<String, Rational>::new();
	for balance in &balances.rows {
		balances_map.insert(balance.key.clone(), Rational(balance.value.0.clone()));
	}

	let shared_balances_map = Arc::new(Mutex::new(balances_map));

	monitor_changes(config.urls.changes, config.urls.doc_root, shared_balances_map.clone(), balances.update_seq);

	let mut router = Router::new();
	router.get("/", move |r: &mut Request| serve_balances(r, shared_balances_map.clone()));

	println!("Ready at http://localhost:4000");
	Iron::new(router).http("localhost:4000").unwrap();
}
