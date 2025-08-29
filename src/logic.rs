pub mod ui {
    use std::borrow::Cow;

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
						database::add_record(Cow::from("sources"), Source {
							origin: String::from("local"),
							path: path_string
						});
					}

					println!("Directory fetched correctly {:?}", source);
					let files = read_source(source).expect("Couldn't fetch all files");
					println!("files read correctly {:?}", files);
					// Add new source to sources.
					// Read through the source on new source added.
					// Check if there is neat way to do this, or do I need to manually call the function here.
					// let list: HashMap<String, Vec<String>> = read_source(result);
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
		use std::{borrow::Cow, collections::HashMap, fmt::Error, fs, path::{Path, PathBuf}};
		use native_dialog::DialogBuilder;
		use super::database;

		#[derive(Debug)]
		pub struct MediaFile {
			pub name: String,
			pub author: String,
			pub path: String,
			pub extension: String,
			pub file_size: u64,
		}

		#[derive(Debug)]
		pub struct Source {
			pub origin: String,
			pub path: String,
		}

		impl std::fmt::Display for MediaFile {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				write!(f, "({}, {}, {}, {}, {})", self.name, self.author, self.path, self.extension, self.file_size)
			}
		}

		impl std::fmt::Display for Source {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				write!(f, "({}, {})", self.origin, self.path)
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
					let metadata = fs::metadata(&entry_path).expect("Couldn't get metadata");
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

					if mime_type != None {
						let author = String::from("None");
						let key = format!("{}.{}", file_name, author);

						hashmap.entry(key).or_insert(MediaFile {
							author,
							name: file_name,
							extension: String::from(file_extension),
							path: String::from(""),
							file_size,
						});
					}
				}
			}

			Ok(hashmap)
		}

		pub fn read_sources() {}

		pub fn get_local_files() -> HashMap<String, MediaFile> {
			// Fetch sources.
			let source_hashmap = database::get_table::<Source>(Cow::from("sources"));
			let mut file_hashmap: HashMap<String, MediaFile> = HashMap::new();

			match source_hashmap {
				Ok(sources) => {
					println!("Table fetched correctly {:?}", sources);
					// Add new source to sources.
					// Read through the source on new source added.
					// Check if there is neat way to do this, or do I need to manually call the function here.
					// let files = read_source(source).expect("Couldn't fetch all files");
					for hash_item in sources {
						if hash_item.0 == "local" {
							let source: Source = hash_item.1;
							let path = PathBuf::from(source.path);
							let files: HashMap<String, MediaFile> = read_source(path)
								.expect("Couldn't fetch all files");

							file_hashmap.extend(files);
						} else {
							println!("Not a local source {:?}", hash_item);
						}
					}

				},
				Err(error) => {
					println!("Detailed error message: {}", error);
				}
			}

			file_hashmap
		}

		fn update_index() {}
	}

	mod audio {

	}

	pub mod database {
    	use std::{borrow::Cow, collections::HashMap};
		use sqlite::Row;
		use thiserror::Error;
    	use super::source::{MediaFile, Source};

		pub trait FromRow {
			fn from_row(row: Row) -> Result<Self, Box<dyn std::error::Error>> where Self: Sized;
		}

		impl FromRow for MediaFile {
			fn from_row(row: Row) -> Result<Self, Box<dyn std::error::Error>> {
				Ok(MediaFile {
					name: row.read(0)?,
					author: row.read(1)?,
					extension: row.read(2)?,
					path: row.read(3)?,
					file_size: row.read(4)?,
				})
			}
		}

		impl FromRow for Source {
			fn from_row(row: Row) -> Result<Self, Box<dyn std::error::Error>> {
				Ok(Source {
					origin: row.read(0)?,
					path: row.read(1)?,
				})
			}
		}

		pub trait CreateKey {
			fn create_key(&self) -> String;
		}

		impl CreateKey for MediaFile {
			fn create_key(&self) -> String {
				format!("{}.{}", self.author, self.name)
			}
		}

		impl CreateKey for Source {
			fn create_key(&self) -> String {
				format!("{}", self.path)
			}
		}

		#[derive(Error, Debug)]
		pub enum CError {
			#[error("SQLite error: {0}")]
			Sqlite(#[from] sqlite::Error),
			#[error("IO error: {0}")]
			Io(#[from] std::io::Error),
		}

		fn connect() -> Result<sqlite::Connection, CError> {
			let db_path = String::from("./database/main.db");
			let result = sqlite::open(&db_path);

			match result {
				Ok(connect) => Ok(connect),
				Err(error) => {
					println!("Error occured: {}", error);
					println!("Trying to solve the problem by creating it.");

					if error.code == Some(14) {
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

					Err(CError::Sqlite(error))
				}
			}
		}

		pub fn initialize_tables() -> Result<(), ()> {
			if let Ok(connection) = connect() {
				let query = String::from("
					PRAGMA foreign_keys = ON;
					CREATE TABLE IF NOT EXISTS main (
						name 	TEXT NOT NULL,
						author 	TEXT NOT NULL,
						path 	TEXT NOT NULL,
						extension TEXT NOT NULL,
						file_size INTEGER,
						source 	INTEGER
						created_on DATETIME DEFAULT (datetime('now', 'localtime'))
					);
					CREATE TABLE IF NOT EXISTS sources (
						id		INTEGER PRIMARY KEY AUTOINCREMENT,
						origin 	TEXT NOT NULL,
						path 	TEXT NOT NULL UNIQUE,
						created_on DATETIME DEFAULT (datetime('now', 'localtime'))
					);
					CREATE INDEX IF NOT EXISTS name_index ON main(name);
					CREATE INDEX IF NOT EXISTS author_index ON main(author);
					CREATE INDEX IF NOT EXISTS source_index ON main(source);
				");

				let response = connection.execute(query);

				match response {
					Ok(_) => return Ok(()),
					Err(error) => println!("Error occured: {}", error),
				}
			}

			Err(())
		}

		pub fn get_table<T: FromRow + CreateKey>(table_name: Cow<'_, str>)
			-> Result<HashMap<String, T>, CError>
		{
			match connect() {
				Ok(connection) => {
					let mut hashmap: HashMap<String, T> = HashMap::new();
					let query = format!("SELECT * FROM {}", table_name.as_ref());
					let statement = connection.prepare(query);
	
					for row in statement
						.unwrap()
						.into_iter()
						.bind((1, 50))
						.unwrap()
						.map(|row| row.unwrap()) 
					{
						if let Ok(value) = T::from_row(row) {
							let key = value.create_key();
							hashmap.insert(key, value);
						}
					}
	
					return Ok(hashmap)
				},
				Err(error) => Err(error),
			}
		}

		pub fn add_record<T: std::fmt::Display>(table_name: Cow<'_, str>, new_record: T) {
			if let Ok(connection) = connect() {
				let query: String = format!(
					"INSERT INTO {} (origin, path) VALUES {}",
					table_name.clone().into_owned(),
					new_record
				);

				if connection.execute(&query).is_err() {
					println!("Failed to execute add_record: {}", query);
				};
			}
		}

		pub fn add_records<T: std::fmt::Display>(table_name: Cow<'_, str>, new_records: HashMap<String, T>) {
			if let Ok(connection) = connect() {
				let mut result: HashMap<usize, bool> = HashMap::new();

				for hash in new_records {
					let record = hash.1;
					let query: String = format!(
						"INSERT INTO {} (name, author, path, extension, file_size, source) VALUES {}",
						table_name.clone().into_owned(),
						record
					);

					let key = result.len();
					let mut value = true;
					
					if connection.execute(query).is_err() {
						value = false;
					};

					result.insert(key, value);
				}
			}
		}
	}
}
