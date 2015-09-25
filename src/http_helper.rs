extern crate hyper;

use std::io::Read;
use self::hyper::Client;
use self::hyper::header::Connection;

pub fn get_url(url:&str) -> String {
	let client = Client::new();

	let mut res = client.get(url)
		.header(Connection::close())
		.send().unwrap();

	let mut body = String::new();
	res.read_to_string(&mut body).unwrap();

	return body;
}
