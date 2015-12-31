use std::convert;
use std::io;
use std::fs::File;
use rustc_serialize::{json, Decodable};

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct ConfigUrls {
	pub changes: String,
	pub view: String,
	pub doc_root: String,
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct Config {
	pub poll_timeout: Option<u32>,
	pub urls: ConfigUrls,
}

#[derive(Debug)]
pub enum Error {
	Io(io::Error),
	DecoderError(json::DecoderError),
	ParserError(json::ParserError),
}

impl convert::From<io::Error> for Error {
	fn from(err: io::Error) -> Error {
		Error::Io(err)
	}
}

impl convert::From<json::DecoderError> for Error {
	fn from(err: json::DecoderError) -> Error {
		Error::DecoderError(err)
	}
}

impl convert::From<json::ParserError> for Error {
	fn from(err: json::ParserError) -> Error {
		Error::ParserError(err)
	}
}

impl Config {
	pub fn from_file(filename : &str) -> Result<Config, Error> {
		let mut file = try!(File::open(filename));
		let json = try!(json::Json::from_reader(&mut file));
		let mut decoder = json::Decoder::new(json);
		Ok(try!(Decodable::decode(&mut decoder)))
	}
}
