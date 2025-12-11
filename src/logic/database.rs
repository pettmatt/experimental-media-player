use std::collections::HashMap;
use rusqlite::{Connection, ErrorCode, Row, ToSql};
use serde::{Deserialize, Serialize};
use super::custom::ErrorHandler;

#[derive(Debug, Clone, PartialEq)]
pub struct MediaFile {
	pub id: i32,
	pub name: String,
	pub artist: String,
	pub path: String,
	pub extension: String,
	pub duration: i32,
	pub file_size: i32,
	pub playing: bool,
}

#[derive(Debug, Clone)]
pub struct Source {
	pub origin: String,
	pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioEntry {
	pub id: i32,
	pub added_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
	pub id: i32,
	pub name: String,
	pub sources: Vec<String>,
	pub image_url: String,
	pub created_at: String,
	pub listened_at: String,
	pub audio_list: Vec<AudioEntry>,
	pub _sources_string: String,
	pub _audio_list_string: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueItem {
	pub media_id: i32,
}

impl std::fmt::Display for MediaFile {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}, {}, {}, {}, {}, {}, {}",
			self.name, self.artist, self.path, self.extension, self.duration, self.file_size, self.playing
		)
	}
}

impl std::fmt::Display for Source {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}, {}", self.origin, self.path)
	}
}

impl std::fmt::Display for Playlist {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f, "{}, {:?}, {}, {}, {}, {:?}", self.name, self.sources, self.image_url, 
			self.created_at, self.listened_at, self.audio_list
		)
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
			duration: 0,
			playing: false,
		}
	}
}

impl Instanceable for Source {
	fn new() -> Self {
		Self { origin: "".to_string(), path: "".to_string() }
	}
}

impl Instanceable for Playlist {
	fn new() -> Self {
		Self {
			id: 0,
			name: "".to_string(),
			sources: Vec::new(),
			image_url: "".to_string(),
			created_at: "".to_string(),
			listened_at: "".to_string(),
			audio_list: Vec::new(),
			_audio_list_string: String::new(),
			_sources_string: String::new()
		}
	}
}

impl Instanceable for QueueItem {
	fn new() -> Self {
		Self { media_id: 0 }
	}
}

trait FromRow {
	fn from_row(row: &Row) -> Result<Self, Box<dyn std::error::Error>> where Self: Sized;
}

impl FromRow for MediaFile {
	fn from_row(row: &Row) -> Result<Self, Box<dyn std::error::Error>> {
		let mut file = Self {
			id: row.get("id")?,
			name: row.get("name")?,
			artist: row.get("artist")?,
			path: row.get("path")?,
			extension: row.get("extension")?,
			file_size: row.get("file_size")?,
			duration: row.get("duration")?,
			playing: row.get("playing")?,
		};

		let mut path_array: Vec<&str> = file.path.split('"').collect();
		path_array.remove(0);
		path_array.remove(path_array.len() - 1);
		file.path = path_array.concat();

		Ok(file)
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

impl FromRow for Playlist {
	fn from_row(row: &Row) -> Result<Self, Box<dyn std::error::Error>> {
		let sources: String = row.get("sources")?;
		let audio_list: String = row.get("audio_list")?;

		Ok(Self {
			id: row.get("id")?,
			name: row.get("name")?,
			sources: serde_json::from_str(&sources).unwrap_or_default(),
			image_url: row.get("image_url")?,
			created_at: row.get("created_at")?,
			listened_at: row.get("listened_at")?,
			audio_list: serde_json::from_str(&audio_list).unwrap_or_default(),
			_sources_string: String::new(),
			_audio_list_string: String::new()
		})
	}
}

impl FromRow for QueueItem {
	fn from_row(row: &Row) -> Result<Self, Box<dyn std::error::Error>> {
		Ok(Self {
			media_id: row.get("media_id")?,
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

impl CreateKey for Playlist {
	fn create_key(&self) -> String {
		String::from(&self.name)
	}
}

impl CreateKey for QueueItem {
	fn create_key(&self) -> String {
		self.media_id.to_string()
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

impl rusqlite::ToSql for Playlist {
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

impl GetQuery for MediaFile {
	fn get_query(&self, query: SqlQueries) -> String {
		match query {
			SqlQueries::Insert => {
				String::from("
					INSERT INTO main (name, artist, path, extension, file_size, duration, playing)
					VALUES (?, ?, ?, ?, ?, ?, ?);
				")
			},
			SqlQueries::Select => String::from("SELECT * FROM main;"),
		}
	}
}

impl GetQuery for Playlist {
	fn get_query(&self, query: SqlQueries) -> String {
		match query {
			SqlQueries::Insert => String::from("
				INSERT INTO playlists (name, sources, image_url, audio_list, created_at, listened_at)
				VALUES (?, ?, ?, ?, ?, ?);
			"),
			SqlQueries::Select => String::from("SELECT * FROM playlists;"),
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
			&self.duration as &dyn ToSql,
			&self.playing as &dyn ToSql,
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

impl ToSqlParams for Playlist {
	fn to_sql_params(&self) -> Vec<&dyn ToSql> {
		vec![
			&self.name as &dyn ToSql,
			&self._sources_string as &dyn ToSql,
			&self.image_url as &dyn ToSql,
			&self.created_at as &dyn ToSql,
			&self.listened_at as &dyn ToSql,
			&self._audio_list_string as &dyn ToSql,
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

pub trait Convertable {
	fn convert_to_string(&mut self);
}

impl Convertable for Source {
	fn convert_to_string(&mut self) {}
}

impl Convertable for MediaFile {
	fn convert_to_string(&mut self) {}
}

impl Convertable for QueueItem {
	fn convert_to_string(&mut self) {}
}

impl Convertable for Playlist {
	fn convert_to_string(&mut self) {
		self._sources_string = serde_json::to_string(&self.sources)
			.unwrap_or(String::new());
		self._audio_list_string = serde_json::to_string(&self._audio_list_string)
			.unwrap_or(String::new());
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
			"CREATE TABLE IF NOT EXISTS main (
				id			INTEGER PRIMARY KEY AUTOINCREMENT,
				name 		TEXT NOT NULL,
				artist 		TEXT NOT NULL,
				path 		TEXT NOT NULL UNIQUE,
				extension 	TEXT NOT NULL,
				file_size 	INTEGER,
				duration	INTEGER,
				playing		INTEGER NOT NULL,
				created 	DATETIME DEFAULT (datetime('now', 'localtime'))
			);",
			"CREATE TABLE IF NOT EXISTS queue (
				id			INTEGER PRIMARY KEY AUTOINCEMENT,
				media_id	INTEGER NOT NULL,
				created 	DATETIME DEFAULT (datetime('now', 'localtime')),
				FOREIGN KEY (media_id) REFERENCES main(id)
			);",
			"CREATE TABLE IF NOT EXISTS playlist (
				id			INTEGER PRIMARY KEY AUTOINCEMENT,
				name		TEXT NOT NULL,
				sources		TEXT,
				image_url	TEXT,
				audio_list	TEXT,
				created_at 	DATETIME DEFAULT (datetime('now', 'localtime')),
				listened_at	DATETIME DEFAULT (datetime('now', 'localtime'))
			);",
			// "CREATE UNIQUE INDEX IF NOT EXISTS path_index ON main(path);",
			// "CREATE INDEX IF NOT EXISTS name_index ON main(name);",
			// "CREATE INDEX IF NOT EXISTS author_index ON main(author);",
			// "CREATE INDEX IF NOT EXISTS source_index ON main(source);",
			// "CREATE INDEX IF NOT EXISTS sources_index ON sources(path);"
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

			println!("(Database) - get_table: {:?}", &list);
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

pub fn add_records<T: std::fmt::Display + std::fmt::Debug + rusqlite::ToSql + GetQuery + ToSqlParams + Convertable>(
	new_records: Vec<T>
) {
	if let Ok(connection) = connect() {
		println!("Adding records: {:?}", &new_records);
		let mut result: HashMap<usize, ErrorBody> = HashMap::new();

		for mut record in new_records {
			record.convert_to_string();
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