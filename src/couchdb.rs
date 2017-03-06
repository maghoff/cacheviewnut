#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct RowWithId<Key, Value> {
	pub id: String,
	pub key: Key,
	pub value: Value,
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct Row<Key, Value> {
	pub key: Key,
	pub value: Value,
}

#[derive(Debug, RustcDecodable)]
pub struct View<Key, Value> {
	pub total_rows: u32,
	pub offset: u32,
	pub rows: Vec<RowWithId<Key, Value>>,
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct ReducedView<Key, Value> {
	pub rows: Vec<Row<Key, Value>>,
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
pub struct ReducedViewWithUpdateSeq<Key, Value> {
	pub update_seq: String,
	pub rows: Vec<Row<Key, Value>>,
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
	pub last_seq: String,
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
