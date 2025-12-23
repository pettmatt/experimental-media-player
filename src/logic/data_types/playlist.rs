use crate::logic::data_types::{source::Source, Convertable, CreateKey, FromRow, GetQuery, Instanceable, SqlQueries, ToSqlParams};
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
	pub artist: Option<String>,
	pub list_type: String,
	pub image_url: String,
	pub created_at: String,
	pub listened_at: String,
	pub tracks: Option<Vec<AudioEntry>>,
	pub sources: Option<Vec<Source>>,
}

impl std::fmt::Display for Playlist {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f, "{}, {}, {}, {}, {:?}", self.name, self.image_url,
			self.created_at, self.listened_at, self.tracks
		)
	}
}

impl Instanceable for Playlist {
	fn new() -> Self {
		Self {
			id: 0,
			name: "".to_string(),
			list_type: "".to_string(),
			artist: None,
			image_url: "".to_string(),
			created_at: "".to_string(),
			listened_at: "".to_string(),
			tracks: None,
			sources: None,
		}
	}
}

impl FromRow for Playlist {
	fn from_row(row: &Row) -> Result<Self, Box<dyn std::error::Error>> {
		// TODO: Tracks and sources should be joined through the sql query
		Ok(Self {
			id: row.get("id")?,
			list_type: row.get("list_type")?,
			name: row.get("name")?,
			artist: row.get("artist")?,
			image_url: row.get("image_url")?,
			created_at: row.get("created_at")?,
			listened_at: row.get("listened_at")?,
			tracks: None,
			sources: None,
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
				INSERT INTO playlists (name, list_type, image_url, artist)
				VALUES (?, ?, ?, ?);
			"),
			SqlQueries::Select => String::from("SELECT * FROM playlists;"),
			SqlQueries::Update => String::from("
				UPDATE playlists
				SET
					name = (name),
					list = (list_type),
					image_url = (image_url),
					artist = (artist),
				WHERE name = (name)
				VALUES (?, ?, ?, ?);
			"),
			SqlQueries::Delete => String::from("
				DELETE FROM playlists WHERE name = (name)
				VALUES (?);
			"),
		}
	}
}

impl ToSqlParams for Playlist {
	fn to_sql_params(&self) -> Vec<&dyn ToSql> {
		vec![
			&self.name as &dyn ToSql,
			&self.list_type as &dyn ToSql,
			&self.image_url as &dyn ToSql,
			&self.artist as &dyn ToSql
		]
	}
}
