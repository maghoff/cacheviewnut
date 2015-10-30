#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct RowWithId<T> {
	pub id: String,
	pub key: String,
	pub value: T,
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct Row<T> {
	pub key: String,
	pub value: T,
}

#[derive(Debug, RustcDecodable)]
pub struct View<Value> {
	pub total_rows: u32,
	pub offset: u32,
	pub rows: Vec<RowWithId<Value>>,
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct ReducedView<Value> {
	pub rows: Vec<Row<Value>>,
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct ReducedViewWithUpdateSeq<Value> {
	pub update_seq: u32,
	pub rows: Vec<Row<Value>>,
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct Revision {
	pub rev: String,
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct Change {
	pub seq: u32,
	pub id: String,
	pub changes: Vec<Revision>,
	pub deleted: Option<bool>,
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct Changes {
	pub results: Vec<Change>,
	pub last_seq: u32,
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct Meta {
	timestamp: String,
	description: String,
}

use std::collections::HashMap;
use rational::Rational;
#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct Transaction {
	pub credits: HashMap<String, Rational>,
	pub debets: HashMap<String, Rational>,
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub enum RevisionStatus {
	available,
	missing,
	deleted,
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct RevInfo {
	pub rev: String,
	pub status: RevisionStatus
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct TransactionDocument {
	pub _id: String,
	pub _rev: String,
	pub _revs_info: Option<Vec<RevInfo>>,
	pub meta: Meta,
	pub transaction: Transaction,
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct Revisions {
	pub start: usize,
	pub ids: Vec<String>,
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct RevisionsDocument {
	pub _revisions: Revisions,
}
