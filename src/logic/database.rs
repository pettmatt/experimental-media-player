use std::collections::HashMap;
use rusqlite::{Connection, ErrorCode};
use crate::logic::database_types::{Convertable, CreateKey, FromRow, GetQuery, Instanceable, SqlQueries, ToSqlParams};

use super::custom::ErrorHandler;

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
	// TODO: Create more robust database structure:
	// main -> tracks
	// artist
	// source
	// queue (to restore previous session, could be renamed to "session" and store more data in it)
	// playlists
	// albums (created by structuring files in directories) or through *.ogg.m3u file
	// settings
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
