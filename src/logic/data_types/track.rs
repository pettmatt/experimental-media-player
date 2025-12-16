use crate::logic::data_types::{Instanceable, Convertable, CreateKey, FromRow, GetQuery, SqlQueries, ToSqlParams};
use rusqlite::{Row, ToSql};

#[derive(Debug, Clone, PartialEq)]
pub struct Track {
    pub id: i32,
    pub name: String,
    pub artists: String,
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
            self.artists,
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
            artists: "".to_string(),
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
    	let artists_string: String = row.get("artists")?;
     	let artists_temp: Vec<&str> = artists_string.split(", ").collect();
      	let artists = artists_temp.into_iter().map(|s| s.to_string()).collect();

        let mut file = Self {
            id: row.get("id")?,
            name: row.get("name")?,
            artists: artists,
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
					INSERT INTO tracks (name, artists, path, extension, file_size, duration, playing)
					VALUES (?, ?, ?, ?, ?, ?, ?);
				",
            ),
            SqlQueries::Select => String::from("SELECT * FROM tracks;"),
 			SqlQueries::Update => String::from("
				UPDATE tracks
				SET
					name = (name),
					artists = (artists),
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
            &self.name as &dyn ToSql,
            &self.artists as &dyn ToSql,
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
