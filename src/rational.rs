extern crate core;
extern crate num;
extern crate rustc_serialize;
extern crate regex;

use self::core::str::FromStr;
use self::num::rational::BigRational as BR;
use self::regex::Regex;
use self::rustc_serialize::{Decodable, Decoder, Encodable, Encoder};
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub struct Rational(pub BR);

pub fn parse_mixed_number(number : &str) -> Result<Rational, String> {
	let mixed_number = Regex::new(r"^((-)?(\d+)( (\d+/\d+))?|(-?\d+/\d+))$").unwrap();
	match mixed_number.captures(number) {
		Some(groups) => {
			let mut result = BR::from_str("0").unwrap();
			if let Some(x) = groups.at(3) { result = result + BR::from_str(x).unwrap(); }
			if let Some(x) = groups.at(5) { result = result + BR::from_str(x).unwrap(); }
			if let Some(x) = groups.at(6) { result = result + BR::from_str(x).unwrap(); }
			if let Some(_) = groups.at(2) { result = -result; }
			Ok(Rational(result))
		},
		None => Err("Not a valid mixed number".to_string())
	}
}

impl Decodable for Rational {
	fn decode<D : Decoder>(decoder: &mut D) -> Result<Rational, D::Error> {
		match parse_mixed_number(&&try!{decoder.read_str()}) {
			Ok(x) => Ok(x),
			Err(err) => Err(decoder.error(&err.to_string()))
		}
	}
}

impl Encodable for Rational {
	fn encode<E : Encoder>(&self, encoder: &mut E) -> Result<(), E::Error> {
		encoder.emit_str(&format!("{}", self.0))
	}
}

impl fmt::Display for Rational {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}
