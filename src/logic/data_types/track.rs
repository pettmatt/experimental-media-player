use crate::logic::data_types::{Convertable, CreateKey, FromRow, GetQuery, Instanceable, SqlQueries, ToSqlParams};
use rusqlite::{Row, ToSql};

#[derive(Debug, Clone, PartialEq)]
pub struct Track {
    pub id: i32,
    pub title: String,
    pub artist: String,
    pub path: String,
    pub genre: String,
    pub year: u32,
    pub extension: String,
    pub duration: i32,
    pub file_size: i32,
    pub playing: bool,
}

impl std::fmt::Display for Track {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}, {}, {}, {}, {}, {}, {}, {}, {}",
            self.title,
            self.artist,
            self.path,
            self.genre,
            self.year,
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
            title: "".to_string(),
            artist: "".to_string(),
            path: "".to_string(),
            genre: "".to_string(),
            year: 0,
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
            title: row.get("title")?,
            artist: row.get("artist")?,
            path: row.get("path")?,
            genre: row.get("genre")?,
            year: row.get("year")?,
            extension: row.get("extension")?,
            file_size: row.get("file_size")?,
            duration: row.get("duration")?,
            playing: row.get("playing")?,
        };

        // Used to remove unnecessary '"' from string. Could be added to State as get_path() method.
        // TODO: Remove if commenting out doesn't break anything.
        // let mut path_array: Vec<&str> = file.path.split('"').collect();
        // println!("PATH ARRAY: {:?}", path_array);
        // path_array.remove(0);
        // path_array.remove(path_array.len() - 1);
        // file.path = path_array.concat();

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
            SqlQueries::Insert => String::from("
				INSERT INTO tracks (title, artist, path, extension, file_size, duration, playing)
				VALUES (?, ?, ?, ?, ?, ?, ?);
			",),
            SqlQueries::Select => String::from("SELECT * FROM tracks;"),
 			SqlQueries::Update => String::from("
				UPDATE tracks
				SET
					title = (title),
					artist = (artist),
					path = (path),
					extension = (extension),
					file_size = (file_size),
					duration = (duration),
					playing = (playing),
				WHERE id = (id)
				VALUES (?, ?, ?, ?, ?, ?, ?, ?);
    		"),
			SqlQueries::Delete => String::from("
				DELETE FROM tracks WHERE id = (id)
				VALUES (?);
			"),
        }
    }
}

impl ToSqlParams for Track {
    fn to_sql_params(&self) -> Vec<&dyn ToSql> {
        vec![
            &self.title as &dyn ToSql,
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
