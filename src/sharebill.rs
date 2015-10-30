use std::collections::btree_map::BTreeMap;
use rational::Rational;

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct Meta {
	pub timestamp: String,
	pub description: String,
}

#[derive(Debug, RustcDecodable)]
pub struct Transaction {
	pub debets: BTreeMap<String, Rational>,
	pub credits: BTreeMap<String, Rational>,
}

#[derive(Debug, RustcDecodable)]
pub struct TransactionDocument {
	pub _id: String,
	pub _rev: String,
	pub meta: Meta,
	pub transaction: Transaction,
}
