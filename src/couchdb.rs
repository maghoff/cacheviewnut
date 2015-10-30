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
pub struct Revisions {
	pub start: usize,
	pub ids: Vec<String>,
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct RevisionsDocument {
	pub _revisions: Revisions,
}
