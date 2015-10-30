use std::convert;
use std::io;
use std::io::prelude::*;
use std::fs::File;
use rustc_serialize::json;

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct ConfigUrls {
	pub changes: String,
	pub view: String,
	pub doc_root: String,
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct Config {
	pub urls : ConfigUrls,
}

#[derive(Debug)]
pub enum Error {
	Io(io::Error),
	DecoderError(json::DecoderError),
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

impl Config {
	pub fn from_file(filename : &str) -> Result<Config, Error> {
		let mut file = try!{File::open(filename)};
		let mut buffer = String::new();
		try!{file.read_to_string(&mut buffer)};

		Ok(try!{json::decode::<Config>(&buffer)})
	}
}
