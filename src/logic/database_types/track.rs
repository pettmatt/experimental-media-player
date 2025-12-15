use crate::logic::database_types::{Instanceable, Convertable, CreateKey, FromRow, GetQuery, SqlQueries, ToSqlParams};
use rusqlite::{Row, ToSql};

#[derive(Debug, Clone, PartialEq)]
pub struct Track {
    pub id: i32,
    pub name: String,
    pub artist: String,
    pub path: String,
    pub extension: String,
    pub duration: i32,
    pub file_size: i32,
    pub playing: bool,
}

impl std::fmt::Display for Track {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}, {}, {}, {}, {}, {}, {}",
            self.name,
            self.artist,
            self.path,
            self.extension,
            self.duration,
            self.file_size,
            self.playing
        )
    }
}

impl Instanceable for Track {
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

impl FromRow for Track {
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

impl CreateKey for Track {
    fn create_key(&self) -> String {
        format!("{}", self.path)
    }
}

impl rusqlite::ToSql for Track {
    #[inline]
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        self.to_sql()
    }
}

impl GetQuery for Track {
    fn get_query(&self, query: SqlQueries) -> String {
        match query {
            SqlQueries::Insert => String::from(
                "
					INSERT INTO main (name, artist, path, extension, file_size, duration, playing)
					VALUES (?, ?, ?, ?, ?, ?, ?);
				",
            ),
            SqlQueries::Select => String::from("SELECT * FROM main;"),
        }
    }
}

impl ToSqlParams for Track {
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

impl Convertable for Track {
    fn convert_to_string(&mut self) {}
}
