use crate::logic::data_types::{Convertable, CreateKey, FromRow, GetQuery, Instanceable, SqlQueries, ToSqlParams};
use rusqlite::{Row, ToSql};
use serde::{Deserialize, Serialize};
use requests::api::yt::{MusicVideo};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Track {
    pub id: i32,
    pub title: String,
    pub artist: String,
    pub path: String,
    pub genre: String,
    pub year: String,
    pub extension: String,
    pub duration: i32,
    pub str_duration: String,
    pub thumbnail: String,
    pub file_size: i32,
    pub playing: bool,
}

impl From<MusicVideo> for Track {
	fn from(v: MusicVideo) -> Self {
		let mut thumbnail = "".to_string();
		if let Some(url) = v.thumbnail_url {
			thumbnail = url
		}

		Track {
			id: 0,
			title: v.title,
			artist: v.channel_title,
			path: format!("https://www.youtube.com/watch?v={}", v.video_id),
			genre: "".to_string(),
			year: v.published_at,
			thumbnail: thumbnail,
			extension: "online".to_string(),
			duration: 0, // Should be updated when the video is fetched
			str_duration: "".to_string(),
			file_size: 0,
			playing: false,
		}
	}
}

impl std::fmt::Display for Track {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    	let thumbnail = if self.thumbnail == "" {
     		"no-thumbnail".to_string()
     	} else {
      		self.thumbnail.clone()
      	};

        write!(
            f,
            "{}, {}, {}, {:?}, {}, {}, {}, {}, {}, {}",
            self.title,
            self.artist,
            self.path,
            self.genre,
            self.year,
            thumbnail,
            self.extension,
            self.duration,
            self.file_size,
            self.playing,
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
            year: "".to_string(),
            thumbnail: "".to_string(),
            extension: "".to_string(),
            file_size: 0,
            duration: 0,
            str_duration: 0.to_string(),
            playing: false,
        }
    }
}

impl FromRow for Track {
    fn from_row(row: &Row) -> Result<Self, Box<dyn std::error::Error>> {
    	let year = row.get("year").unwrap_or_else(|_| "".to_string());
     	let genre = row.get("genre").unwrap_or_else(|_| "".to_string());
      	let str_duration = row.get("duration").unwrap_or_else(|_| "".to_string());

        let file = Self {
            id: row.get("id")?,
            title: row.get("title")?,
            artist: row.get("artist")?,
            path: row.get("path")?,
            genre: genre,
            year: year,
            thumbnail: row.get("thumbnail")?,
            extension: row.get("extension")?,
            file_size: row.get("file_size")?,
            duration: row.get("duration")?,
            str_duration: str_duration,
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
				INSERT INTO tracks (title, artist, path, genre, year, thumbnail, extension, file_size, duration, playing)
				VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?);
			"),
            SqlQueries::Select => String::from("SELECT * FROM tracks;"),
 			SqlQueries::Update => String::from("
				UPDATE tracks
				SET
					title = (title),
					artist = (artist),
					path = (path),
					genre = (genre),
					year = (year),
					thumbnail = (thumbnail),
					extension = (extension),
					file_size = (file_size),
					duration = (duration),
					playing = (playing),
				WHERE id = (id)
				VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?);
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
            &self.genre as &dyn ToSql,
            &self.year as &dyn ToSql,
            &self.thumbnail as &dyn ToSql,
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
