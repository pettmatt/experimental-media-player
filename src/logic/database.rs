use std::collections::HashMap;

use rusqlite::{Connection, ErrorCode, Row, ToSql};
use super::custom::ErrorHandler;

struct SourceIndex {
	id: i32,
	path: String
}

#[derive(Debug, Clone)]
pub struct MediaFile {
	pub id: usize,
	pub name: String,
	pub artist: String,
	pub path: String,
	pub extension: String,
	pub file_size: u64,
}

#[derive(Debug, Clone)]
pub struct Source {
	pub origin: String,
	pub path: String,
}

#[derive(Debug, Clone)]
pub struct QueueItem {
	pub media_id: usize,
	pub currently_playing: bool,
}

impl std::fmt::Display for MediaFile {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}, {}, {}, {}, {}", self.name, self.artist, self.path, self.extension, self.file_size)
	}
}

impl std::fmt::Display for Source {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}, {}", self.origin, self.path)
	}
}

pub trait Instanceable {
	fn new() -> Self;
}

impl Instanceable for MediaFile {
	fn new() -> Self {
		Self {
			id: 0,
			name: "".to_string(),
			artist: "".to_string(),
			path: "".to_string(),
			extension: "".to_string(),
			file_size: 0,
		}
	}
}

impl Instanceable for Source {
	fn new() -> Self {
		Self { origin: "".to_string(), path: "".to_string() }
	}
}

impl Instanceable for QueueItem {
	fn new() -> Self {
		Self { media_id: 0, currently_playing: false }
	}
}

trait FromRow {
	fn from_row(row: &Row) -> Result<Self, Box<dyn std::error::Error>> where Self: Sized;
}

impl FromRow for MediaFile {
	fn from_row(row: &Row) -> Result<Self, Box<dyn std::error::Error>> {
		Ok(Self {
			id: row.get("id")?,
			name: row.get("name")?,
			artist: row.get("artist")?,
			extension: row.get("extension")?,
			path: row.get("path")?,
			file_size: row.get("file_size")?,
		})
	}
}

impl FromRow for Source {
	fn from_row(row: &Row) -> Result<Self, Box<dyn std::error::Error>> {
		Ok(Self {
			origin: row.get("origin")?,
			path: row.get("path")?,
		})
	}
}

impl FromRow for QueueItem {
	fn from_row(row: &Row) -> Result<Self, Box<dyn std::error::Error>> {
		Ok(Self {
			media_id: row.get("media_id")?,
			currently_playing: row.get("is_playing")?,
		})
	}
}

impl FromRow for SourceIndex {
	fn from_row(row: &Row) -> Result<Self, Box<dyn std::error::Error>> {
		Ok(SourceIndex {
			id: row.get("id")?,
			path: row.get("path")?,
		})
	}
}

trait CreateKey {
	fn create_key(&self) -> String;
}

impl CreateKey for MediaFile {
	fn create_key(&self) -> String {
		format!("{}", self.path)
	}
}

impl CreateKey for Source {
	fn create_key(&self) -> String {
		String::from(&self.path)
	}
}

impl CreateKey for QueueItem {
	fn create_key(&self) -> String {
		self.media_id.to_string()
	}
}

impl CreateKey for SourceIndex {
	fn create_key(&self) -> String {
		self.id.to_string()
	}
}

impl rusqlite::ToSql for MediaFile {
	#[inline]
	fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
		self.to_sql()
	}
}

impl rusqlite::ToSql for Source {
	fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
		self.to_sql()
	}
}

impl rusqlite::ToSql for QueueItem {
	fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
		self.to_sql()
	}
}

enum SqlQueries {
	Insert,
	Select
}

trait GetQuery {
	fn get_query(&self, query: SqlQueries) -> String;
}

impl GetQuery for Source {
	fn get_query(&self, query: SqlQueries) -> String {
		match query {
			SqlQueries::Insert => String::from("
				INSERT INTO sources (origin, path)
				VALUES (?, ?);
			"),
			SqlQueries::Select => String::from("SELECT * FROM sources;"),
		}
	}
}

// impl GetQuery for SourceIndex {
// 	fn get_query(&self, query: SqlQueries) -> String {
// 		match query {
// 			SqlQueries::Insert => String::from("INSERT INTO sources (origin, path) VALUES (?2);"),
// 			SqlQueries::Select => String::from("SELECT * FROM (?1);"),
// 		}
// 	}
// }

impl GetQuery for MediaFile {
	fn get_query(&self, query: SqlQueries) -> String {
		match query {
			SqlQueries::Insert => {
				String::from("
					INSERT INTO main (name, artist, path, extension, file_size)
					VALUES (?, ?, ?, ?, ?);
				")
			},
			SqlQueries::Select => String::from("SELECT * FROM main;"),
		}
	}
}

impl GetQuery for QueueItem {
	fn get_query(&self, query: SqlQueries) -> String {
		match query {
			SqlQueries::Insert => {
				String::from("
					INSERT INTO queue (media_id)
					VALUES (?);
				")
			},
			SqlQueries::Select => String::from("SELECT * FROM queue;"),
		}
	}
}

trait ToSqlParams {
	fn to_sql_params(&self) -> Vec<&dyn ToSql>;
}

impl ToSqlParams for MediaFile {
	fn to_sql_params(&self) -> Vec<&dyn ToSql> {
		vec![
			&self.name as &dyn ToSql,
			&self.artist as &dyn ToSql,
			&self.path as &dyn ToSql,
			&self.extension as &dyn ToSql,
			&self.file_size as &dyn ToSql,
		]
	}
}

impl ToSqlParams for Source {
	fn to_sql_params(&self) -> Vec<&dyn ToSql> {
		vec![
			&self.origin as &dyn ToSql,
			&self.path as &dyn ToSql,
		]
	}
}

impl ToSqlParams for QueueItem {
	fn to_sql_params(&self) -> Vec<&dyn ToSql> {
		vec![
			&self.media_id as &dyn ToSql,
		]
	}
}

fn connect() -> Result<Connection, ErrorHandler> {
	let db_path = String::from("./database/main.db");
	let result: Result<Connection, rusqlite::Error> = Connection::open(&db_path);

	match result {
		Ok(connect) => Ok(connect),
		Err(error) => {
			println!("Error occured: {}", error);
			println!("Trying to solve the problem by creating it.");

			if error.sqlite_error_code() == Some(ErrorCode::NotFound) {
				let mut builder = std::fs::DirBuilder::new();
				builder.recursive(true).create("./database")?;

				match std::fs::File::create(db_path) {
					Ok(file) => {
						println!("Created sqlite db file: {:?}", file);
						return self::connect();
					},
					Err(message) => {
						println!("Error occured: {}", message);
						return Err(ErrorHandler::Io(message));
					}
				}
			}

			println!("Unhandled SQLite error occured: {}", error);
			Err(ErrorHandler::Sqlite(error))
		}
	}
}

pub fn initialize_tables() -> Result<(), ()> {
	if let Ok(connection) = connect() {
		let queries = [
			"PRAGMA foreign_keys = ON;",
			"CREATE TABLE IF NOT EXISTS sources (
				id			INTEGER PRIMARY KEY AUTOINCREMENT,
				origin 		TEXT NOT NULL,
				path 		TEXT NOT NULL UNIQUE,
				created 	DATETIME DEFAULT (datetime('now', 'localtime'))
			);",
			// source_id 	INTEGER,
			// FOREIGN KEY (source) REFERENCES sources(id)
			"CREATE TABLE IF NOT EXISTS main (
				id			INTEGER PRIMARY KEY AUTOINCREMENT,
				name 		TEXT NOT NULL,
				artist 		TEXT NOT NULL,
				path 		TEXT NOT NULL UNIQUE,
				extension 	TEXT NOT NULL,
				file_size 	INTEGER,
				created 	DATETIME DEFAULT (datetime('now', 'localtime'))
			);",
			"CREATE TABLE IF NOT EXISTS queue (
				id			INTEGER PRIMARY KEY AUTOINCEMENT,
				media_id	INTEGER NOT NULL,
				is_playing	INTEGER NOT NULL,
				FOREIGN KEY (media_id) REFERENCES main(id),
				created 	DATETIME DEFAULT (datetime('now', 'localtime'))
			);",
			"CREATE UNIQUE INDEX IF NOT EXISTS path_index ON main(path);",
			"CREATE INDEX IF NOT EXISTS name_index ON main(name);",
			"CREATE INDEX IF NOT EXISTS author_index ON main(author);",
			// "CREATE INDEX IF NOT EXISTS source_index ON main(source);",
			"CREATE INDEX IF NOT EXISTS sources_index ON sources(path);"
		];

		let mut index = 0;
		for query in queries {
			index += 1;
			let response = connection.execute(query, ());

			if let Err(message) = response {
				println!("Error occured while initializing ({}): {:?}", index, message);
				return Err(());
			}
		}

		return Ok(());
	}

	Err(())
}

pub fn get_table<T: std::fmt::Debug + FromRow + CreateKey + GetQuery + Instanceable>()
	-> Result<Vec<T>, ErrorHandler>
{
	match connect() {
		Ok(connection) => {
			let mut list: Vec<T> = Vec::new();
			let instance = T::new();
			let query = instance.get_query(SqlQueries::Select);
			let mut statement = connection.prepare(&query)?;

			let iter = statement
				.query_map([], |row| {
					Ok(T::from_row(row).unwrap())
				})?;

			for record in iter.flatten() {
				list.push(record);
			}

			Ok(list)
		},
		Err(error) => Err(error),
	}
}

pub fn add_record<T: std::fmt::Debug + rusqlite::ToSql + GetQuery + ToSqlParams>(
	new_record: T
) {
	if let Ok(connection) = connect() {
		if connection.execute(
			&new_record.get_query(SqlQueries::Insert),
			new_record.to_sql_params().as_slice()
		).is_err() {
			println!("Failed to execute add_record: {:?}; {:?}", new_record, connection);
		};
	}
}

#[derive(std::fmt::Debug)]
struct ErrorBody {
	is_error: bool,
	message: Result<usize, rusqlite::Error>,
}

pub fn add_records<T: std::fmt::Display + std::fmt::Debug + rusqlite::ToSql + GetQuery + ToSqlParams>(
	new_records: Vec<T>
) {
	if let Ok(connection) = connect() {
		println!("Add records: {:?}", &new_records);
		let mut result: HashMap<usize, ErrorBody> = HashMap::new();

		for record in new_records {
			let key = result.len();
			let mut response = ErrorBody {
				is_error: false,
				message: Ok(0),
			};

			let ex_result = connection.execute(
				&record.get_query(SqlQueries::Insert),
				record.to_sql_params().as_slice()
			);

			if ex_result.is_err() {
				response.is_error = true;
				response.message = ex_result;
			};

			result.insert(key, response);
		}

		println!("Failure Hashmap: {:?}", result);
	}
}