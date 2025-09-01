pub mod ui {
    use crate::{logic::managment::{database, source::{read_source, Source}}, AppWindow};
    use super::managment::source::new_local_source;

	pub fn handle_events(app: &AppWindow) {
		// Media elements bottom panel.
		app.on_media_change(move |index: i32| audio_control_events::handle_media_change(index));
		app.on_media_start(move |start: bool| audio_control_events::handle_media_start(start));
		app.on_media_loop(move |create_loop: bool| audio_control_events::handle_media_loop(create_loop));
		app.on_media_mix(audio_control_events::handle_media_mix);

		// Settings
		app.on_new_local_source(move || {
			let source = new_local_source();

			match source {
				Some(source) => {
					{
						let path_string = source.clone().to_str().unwrap().to_string();
						database::add_record(Source {
							origin: String::from("local"),
							path: path_string
						});
					}

					println!("Directory fetched correctly {:?}", source);
					let records = read_source(source).expect("Couldn't fetch all files");
					println!("files read correctly {:?}", records);
					database::add_records(records);
				},
				None => println!("Didn't receive a path. Result should be None: {:?}", source)
			}
		});
	}

	mod audio_control_events {
		pub fn handle_media_change(index: i32) {
			let move_to_next_audio = index > 0;
		}

		pub fn handle_media_start(start: bool) {}

		pub fn handle_media_loop(create_loop: bool) {}

		pub fn handle_media_mix() {}
	}
}

pub mod managment {
	pub mod source {
		use std::{collections::HashMap, fmt::Error, fs, path::{Path, PathBuf}};
		use native_dialog::DialogBuilder;
		use super::database::{self, CError};

		pub trait Instanceable {
			fn new() -> Self;
		}

		#[derive(Debug, Clone)]
		pub struct MediaFile {
			pub name: String,
			pub artist: String,
			pub path: String,
			pub extension: String,
			pub file_size: u64,
		}

		impl Instanceable for MediaFile {
			fn new() -> Self {
				Self {
					name: "".to_string(),
					artist: "".to_string(),
					path: "".to_string(),
					extension: "".to_string(),
					file_size: 0,
				}
			}
		}

		#[derive(Debug, Clone)]
		pub struct Source {
			pub origin: String,
			pub path: String,
		}

		impl Instanceable for Source {
			fn new() -> Self {
				Self { origin: "".to_string(), path: "".to_string() }
			}
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

		pub fn new_local_source() -> Option<PathBuf> {
			let path: Option<PathBuf> = DialogBuilder::file()
				.set_location("~")
				.open_single_dir()
				.show()
				.unwrap();

			path
		}

		pub fn read_source(source: PathBuf) -> Result<HashMap<String, MediaFile>, Error> {
			let mut hashmap: HashMap<String, MediaFile> = HashMap::new();
			let path = source.as_path();

			let entries = fs::read_dir(path).expect("Couldn't read directory from path");

			for entry in entries {
				let entry = entry.expect("Couldn't get entry");
				let entry_path = entry.path();

				if entry_path.is_dir() {
					let nested_files = read_source(entry_path.clone());

					if let Ok(nf) = nested_files {
						hashmap.extend(nf)
					}
				} else {
					let file_name = entry.file_name().to_string_lossy().to_string();
					println!("Entry_path {:?}", &entry_path);
					let metadata = fs::metadata(&entry_path)
						.expect("Couldn't get metadata");
					let file_size = metadata.len();

					let file_extension = Path::new(&entry_path)
						.extension()
						.and_then(|ext| ext.to_str())
						.unwrap_or("unknown_extension");

					let mime_type = match file_extension {
						"mp3" => Some("audio/mpeg"),
						"mp4" => Some("video/mp4"),
						"webm" => Some("video/webm"),
						_ => None
					};

					if mime_type.is_some() {
						let name_array: Vec<&str> = file_name.split(".").collect();
						let audio_name = name_array[0];
						let artist = String::from("unknown");
						let key = format!("{}.{}", audio_name, artist);
						let path = format!("{:?}", entry_path);

						hashmap.entry(key).or_insert(MediaFile {
							artist,
							name: file_name,
							extension: file_extension.to_string(),
							path,
							file_size,
						});
					} else {
						println!("Unknown mime_type: {:?}", mime_type);
						println!("Unhandled file_extension: {:?}", file_extension);
					}
				}
			}

			Ok(hashmap)
		}

		pub fn read_sources() {}

		pub fn validate_sources() -> Result<HashMap<String, MediaFile>, CError> {
			// Fetch sources.
			let source_hashmap = database::get_table::<Source>();
			let mut file_hashmap: HashMap<String, MediaFile> = HashMap::new();

			match source_hashmap {
				Ok(sources) => {
					println!("Table fetched correctly: {:?}", sources);

					for hash_item in sources {
						if hash_item.1.origin == "local" {
							let source: Source = hash_item.1;
							let path = PathBuf::from(source.path);
							let files: HashMap<String, MediaFile> = read_source(path)
								.expect("Couldn't validate some media files");

							file_hashmap.extend(files);
						} else {
							println!("Not a local source {:?}", hash_item);
						}
					}
				},
				Err(error) => {
					println!("Get_local_files() error message: {}", error);
					return Err(error);
				}
			}

			Ok(file_hashmap)
		}

		fn update_index() {}
	}

	mod audio {

	}

	pub mod database {
		use thiserror::Error;
    	use std::collections::HashMap;
		use rusqlite::{ffi::Error, Connection, ErrorCode, Row, ToSql};
    	use super::source::{Instanceable, MediaFile, Source};

		pub struct SourceIndex {
			id: i32,
			path: String
		}

		pub trait FromRow {
			fn from_row(row: &Row) -> Result<Self, Box<dyn std::error::Error>> where Self: Sized;
		}

		impl FromRow for MediaFile {
			fn from_row(row: &Row) -> Result<Self, Box<dyn std::error::Error>> {
				Ok(MediaFile {
					name: row.get(0)?,
					artist: row.get(1)?,
					extension: row.get(2)?,
					path: row.get(3)?,
					file_size: row.get(4)?,
				})
			}
		}

		impl FromRow for Source {
			fn from_row(row: &Row) -> Result<Self, Box<dyn std::error::Error>> {
				Ok(Source {
					origin: row.get("origin")?,
					path: row.get("path")?,
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

		pub trait CreateKey {
			fn create_key(&self) -> String;
		}

		impl CreateKey for MediaFile {
			fn create_key(&self) -> String {
				format!("{}.{}", self.artist, self.name)
			}
		}

		impl CreateKey for Source {
			fn create_key(&self) -> String {
				String::from(&self.path)
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

		pub enum SqlQueries {
			Insert,
			Select
		}

		pub trait GetQuery {
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
							INSERT INTO main (name, author, path, extension, file_size, source)
							VALUES (?, ?, ?, ?, ?, ?);
						")
					},
					SqlQueries::Select => String::from("SELECT * FROM main;"),
				}
			}
		}

		pub trait ToSqlParams {
			fn to_sql_params(&self) -> Vec<&dyn ToSql>;
		}

		impl ToSqlParams for MediaFile {
			fn to_sql_params(&self) -> Vec<&dyn ToSql> {
				vec![
					&self.artist as &dyn ToSql,
					&self.name as &dyn ToSql,
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

		#[derive(Error, Debug)]
		pub enum CError {
			#[error("SQLite error: {0}")]
			Sqlite(#[from] rusqlite::Error),
			#[error("IO error: {0}")]
			Io(#[from] std::io::Error),
		}

		fn connect() -> Result<Connection, CError> {
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
								return Err(CError::Io(message));
							}
						}
					}

					println!("Unhandled SQLite error occured: {}", error);
					Err(CError::Sqlite(error))
				}
			}
		}

		pub fn initialize_tables() -> Result<(), ()> {
			if let Ok(connection) = connect() {
				let queries = [
					"PRAGMA foreign_keys = ON;",
					"CREATE TABLE IF NOT EXISTS sources (
						id		INTEGER PRIMARY KEY AUTOINCREMENT,
						origin 	TEXT NOT NULL,
						path 	TEXT NOT NULL UNIQUE,
						created_on DATETIME DEFAULT (datetime('now', 'localtime'))
					);",
					"CREATE TABLE IF NOT EXISTS main (
						name 	TEXT NOT NULL,
						author 	TEXT NOT NULL,
						path 	TEXT NOT NULL,
						extension TEXT NOT NULL,
						file_size INTEGER,
						created_on DATETIME DEFAULT (datetime('now', 'localtime'))
					);",
					// source 	INTEGER,
					// FOREIGN KEY (source) REFERENCES sources(id)
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
			-> Result<HashMap<String, T>, CError>
		{
			match connect() {
				Ok(connection) => {
					let mut hashmap: HashMap<String, T> = HashMap::new();
					let instance = T::new();
					let query = instance.get_query(SqlQueries::Select);
					let mut statement = connection.prepare(&query)?;
	
					let iter = statement
						.query_map([], |row| {
							Ok(T::from_row(row).unwrap())
						})?;

					for record in iter.flatten() {
						let key = record.create_key();
						hashmap.insert(key, record);
					}
	
					Ok(hashmap)
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
			new_records: HashMap<String, T>
		) {
			if let Ok(connection) = connect() {
				println!("Add records: {:?}", &new_records);
				let mut result: HashMap<usize, ErrorBody> = HashMap::new();

				for hash in new_records {
					let record = hash.1;
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
	}
}
