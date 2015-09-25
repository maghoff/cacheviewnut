use std::collections::btree_map::BTreeMap;
use rational::Rational;

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct Meta  {
	timestamp: String,
	description: String,
}

#[derive(Debug, RustcDecodable)]
pub struct Transaction  {
	debets: BTreeMap<String, Rational>,
	credits: BTreeMap<String, Rational>,
}

#[derive(Debug, RustcDecodable)]
pub struct Post  {
	_id: String,
	_rev: String,
	meta: Meta,
	transaction: Transaction,
}
