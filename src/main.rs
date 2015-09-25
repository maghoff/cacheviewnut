#![feature(core)]
extern crate rustc_serialize;

mod couchdb;
mod http_helper;
mod rational;
mod sharebill;

use couchdb::ReducedView;
use http_helper::get_url;
use rational::Rational;
use rustc_serialize::json;
use std::env;

fn main() {
	let balances : ReducedView<Rational> = json::decode(&get_url(&env::args().nth(1).unwrap())).unwrap();

	println!("{}", json::encode(&balances).unwrap());

	let zero = rational::parse_mixed_number("0").unwrap();
	for row in balances.rows {
		if row.value != zero {
			println!("{}: {}", row.key, row.value);
		}
	}
}
