extern crate hyper;
extern crate rustc_serialize;

use std::convert;
use std::io;
use std::io::Read;
use self::hyper::Client;
use self::hyper::header::Connection;

use rustc_serialize::json;

#[derive(Debug)]
pub enum Error {
	Io(io::Error),
	Hyper(self::hyper::error::Error),
	DecoderError(json::DecoderError),
}

impl convert::From<io::Error> for Error {
	fn from(err: io::Error) -> Error {
		Error::Io(err)
	}
}

impl convert::From<hyper::error::Error> for Error {
	fn from(err: hyper::error::Error) -> Error {
		Error::Hyper(err)
	}
}

impl convert::From<json::DecoderError> for Error {
	fn from(err: json::DecoderError) -> Error {
		Error::DecoderError(err)
	}
}

pub fn get_url(url:&str) -> Result<String, Error> {
	let client = Client::new();

	let mut res = try!{
		client.get(url)
		.header(Connection::close())
		.send()};

	let mut body = String::new();
	try!{res.read_to_string(&mut body)};

	Ok(body)
}

pub fn get_json<T : rustc_serialize::Decodable>(url: &str) -> Result<T, Error> {
	println!("Getting {}", url);
	Ok(try!{json::decode(&&try!{get_url(url)})})
}
