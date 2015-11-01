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


pub struct SharebillBalances;

impl SharebillBalances {
	pub fn map<Emit>(&self, doc: &TransactionDocument, mut emit: Emit)
		where Emit : FnMut(&String, &Rational)
	{
		for (account, value) in &doc.transaction.debets {
			emit(&account, &Rational(-value.0.clone()));
		}
		for (account, value) in &doc.transaction.credits {
			emit(&account, &value);
		}
	}

	pub fn unmap<Emit>(&self, doc: &TransactionDocument, mut emit: Emit)
		where Emit : FnMut(&String, &Rational)
	{
		for (account, value) in &doc.transaction.debets {
			emit(&account, &value);
		}
		for (account, value) in &doc.transaction.credits {
			emit(&account, &Rational(-value.0.clone()));
		}
	}

	pub fn reduce(&self, _key: &str, values: &Vec<Rational>) -> Rational {
		let mut sum = values[0].0.clone();
		for value in &values[1..] {
			sum = &sum + &value.0;
		}
		return Rational(sum);
	}
}
