use crate::logic::data_types::{
    Convertable, CreateKey, FromRow, GetQuery, Instanceable, SqlQueries, ToSqlParams,
};
use rusqlite::{Connection, ErrorCode};
use std::collections::HashMap;

use super::custom::ErrorHandler;

fn connect() -> Result<Connection, ErrorHandler> {
    let db_path = String::from("./database/main.db");
    let result: Result<Connection, rusqlite::Error> = Connection::open(&db_path);

    match result {
        Ok(connect) => Ok(connect),
        Err(error) => {
	        println!("Error occured: {}", error);
	        println!("Trying to solve the problem by creating the db file.");

			let coverable_errors =
				error.sqlite_error_code() == Some(ErrorCode::NotFound) ||
				error.sqlite_error_code() == Some(ErrorCode::CannotOpen);

	        if coverable_errors {
	            let mut builder = std::fs::DirBuilder::new();
	            builder.recursive(true).create("./database").unwrap();

	            match std::fs::File::create(db_path) {
	                Ok(file) => {
	                    println!("Created sqlite db file: {:?}", file);
	                    return self::connect();
	                }
	                Err(message) => {
	                    return Err(ErrorHandler::Io(message));
	                }
	            }
	        }

	        println!("Unhandled SQLite error occured: {}", error);
	        Err(ErrorHandler::Sqlite(error))
        },
    }
}

pub fn initialize_tables() -> Result<(), ()> {
    if let Ok(connection) = connect() {
        let queries = [
            "PRAGMA foreign_keys = ON;",
            "CREATE TABLE IF NOT EXISTS playlists (
				id			INTEGER PRIMARY KEY AUTOINCREMENT,
				list_type	TEXT NOT NULL,
				name		TEXT NOT NULL,
				image_url	TEXT,
				tracks		TEXT,
				created_at 	DATETIME DEFAULT (datetime('now', 'localtime')),
				listened_at	DATETIME DEFAULT (datetime('now', 'localtime')),
				track_id	INTEGER,
				FOREIGN KEY(track_id) REFERENCES tracks(id) ON DELETE SET NULL
			);",
		   "CREATE TABLE IF NOT EXISTS sources (
				id			INTEGER PRIMARY KEY AUTOINCREMENT,
				origin 		TEXT NOT NULL,
				path 		TEXT NOT NULL UNIQUE,
				created 	DATETIME DEFAULT (datetime('now', 'localtime'))
			);",
        	"CREATE TABLE IF NOT EXISTS session (
				id			INTEGER PRIMARY KEY AUTOINCREMENT,
				track_id	INTEGER NOT NULL,
				created 	DATETIME DEFAULT (datetime('now', 'localtime'))
			);",
            "CREATE TABLE IF NOT EXISTS settings (
				id			INTEGER PRIMARY KEY AUTOINCREMENT,
				name		TEXT NOT NULL,
				value		TEXT,
				default_value	TEXT,
				updated_at 	DATETIME DEFAULT (datetime('now', 'localtime')),
				created_at 	DATETIME DEFAULT (datetime('now', 'localtime'))
			);",
            "CREATE TABLE IF NOT EXISTS tracks (
				id			INTEGER PRIMARY KEY AUTOINCREMENT,
				title 		TEXT NOT NULL,
				artist	 	TEXT NOT NULL,
				path 		TEXT NOT NULL UNIQUE,
				genre		TEXT,
				year		INTEGER,
				extension 	TEXT NOT NULL,
				file_size 	INTEGER,
				duration	INTEGER,
				playing		INTEGER NOT NULL,
				created 	DATETIME DEFAULT (datetime('now', 'localtime')),
				source_id	INTEGER,
				playlist_id	INTEGER,
				session_id	INTEGER,
				FOREIGN KEY(source_id) REFERENCES sources(id) ON DELETE SET NULL,
				FOREIGN KEY(playlist_id) REFERENCES playlists(id) ON DELETE SET NULL,
				FOREIGN KEY(session_id) REFERENCES session(id) ON DELETE SET NULL
			);",
			"CREATE playlist_tracks (
				playlist_id INTEGER NOT NULL,
				track_id INTEGER NOT NULL,
				PRIMARY KEY (playlist_id, track_id),
				FOREIGN KEY (playlist_id) REFERENCES playlist(id) ON DELETE CASCADE,
				FOREIGN KEY (track_id) REFERENCES tracks(id) ON DELETE CASCADE,
			);",
			"CREATE playlist_sources (
				playlist_id INTEGER NOT NULL,
				source_id INTEGER NOT NULL,
				PRIMARY KEY (playlist_id, source_id),
				FOREIGN KEY (playlist_id) REFERENCES playlist(id) ON DELETE CASCADE,
				FOREIGN KEY (source_id) REFERENCES sources(id) ON DELETE CASCADE,
			);",
        ];

        let mut index = 0;
        for query in queries {
            index += 1;
            let response = connection.execute(query, ());

            if let Err(message) = response {
                println!(
                    "Error occured while initializing ({}): {:?}",
                    index, message
                );
                return Err(());
            }
        }

        return Ok(());
    }

    Err(())
}

pub fn get_table<T: std::fmt::Debug + FromRow + CreateKey + GetQuery + Instanceable>()
-> Result<Vec<T>, ErrorHandler> {
    match connect() {
        Ok(connection) => {
            let mut list: Vec<T> = Vec::new();
            let instance = T::new();
            let query = instance.get_query(SqlQueries::Select);
            let mut statement = connection.prepare(&query)?;

            let iter = statement.query_map([], |row| Ok(T::from_row(row).unwrap()))?;

            for record in iter.flatten() {
                list.push(record);
            }

            println!("(Database) - get_table: {:?}", &list);
            Ok(list)
        }
        Err(error) => Err(error),
    }
}

pub fn add_record<T: std::fmt::Debug + rusqlite::ToSql + GetQuery + ToSqlParams>(new_record: T)
-> Result<(), ()> {
    if let Ok(connection) = connect() {
        if let Err(error) = connection.execute(
            &new_record.get_query(SqlQueries::Insert),
            new_record.to_sql_params().as_slice(),
        ) {
            println!(
                "Failed to execute add_record: {:?}; {:?}",
                new_record, connection
            );
            println!("Raw error: {:?}", error);
            return Err(());
        };

        return Ok(());
    }

    Err(())
}

#[derive(std::fmt::Debug)]
struct ErrorBody {
    is_error: bool,
    message: Result<usize, rusqlite::Error>,
}

pub fn add_records<
    T: std::fmt::Display + std::fmt::Debug + rusqlite::ToSql + GetQuery + ToSqlParams + Convertable,
>(new_records: Vec<T>) -> Result<(), ()> {
    if let Ok(connection) = connect() {
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
                record.to_sql_params().as_slice(),
            );

            if ex_result.is_err() {
                response.is_error = true;
                response.message = ex_result;
            };

            result.insert(key, response);
        }

        println!("Failure Hashmap: {:?}", result);
        return Ok(());
    }

    Err(())
}

pub fn update_records<T: std::fmt::Debug + rusqlite::ToSql + GetQuery + ToSqlParams>(
    new_records: Vec<T>,
) -> Result<(), ()> {
    if let Ok(connection) = connect() {
    	return Ok(());
    }

    Err(())
}

pub fn delete_records<T: std::fmt::Debug + rusqlite::ToSql + GetQuery + ToSqlParams>(
    record_ids: Vec<i32>,
) -> Result<(), ()> {
    if let Ok(connection) = connect() {
    	return Ok(());
    }

    Err(())
}
