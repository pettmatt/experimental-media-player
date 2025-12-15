use crate::logic::database_types::{Instanceable, Convertable, CreateKey, FromRow, GetQuery, SqlQueries, ToSqlParams};
use rusqlite::{Row, ToSql};
use serde::{Deserialize, Serialize};

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

impl std::fmt::Display for Playlist {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f, "{}, {:?}, {}, {}, {}, {:?}", self.name, self.sources, self.image_url,
			self.created_at, self.listened_at, self.audio_list
		)
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

impl CreateKey for Playlist {
	fn create_key(&self) -> String {
		String::from(&self.name)
	}
}

impl rusqlite::ToSql for Playlist {
	fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
		self.to_sql()
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

impl Convertable for Playlist {
	fn convert_to_string(&mut self) {
		self._sources_string = serde_json::to_string(&self.sources)
			.unwrap_or(String::new());
		self._audio_list_string = serde_json::to_string(&self._audio_list_string)
			.unwrap_or(String::new());
	}
}
