#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct RowWithId<T>  {
	pub id: String,
	pub key: String,
	pub value: T,
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct Row<T>  {
	pub key: String,
	pub value: T,
}

#[derive(Debug, RustcDecodable)]
pub struct View<Value>  {
	pub total_rows: u32,
	pub offset: u32,
	pub rows: Vec<RowWithId<Value>>,
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct ReducedView<Value>  {
	pub rows: Vec<Row<Value>>,
}
