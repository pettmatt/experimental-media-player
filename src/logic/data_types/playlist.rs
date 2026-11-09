use crate::{logic::{data_types::{FromRow, GetQuery, SlintConvertable, SqlQueries, ToSlint, ToSqlParams, source::Source, state::State}, slint::convert_to_slint_model}, slint_generatedAppWindow};
use rusqlite::{Row, ToSql};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioEntry {
	pub id: i32,
	// pub path: String,
	pub added_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
	pub id: i32,
	pub name: String,
	pub artist: Option<String>,
	pub list_type: String,
	pub thumbnail: String,
	pub created_at: String,
	pub listened_at: String,
	pub tracks: Option<Vec<AudioEntry>>,
	pub sources: Option<Vec<Source>>,
}

impl std::fmt::Display for Playlist {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f, "{}, {}, {}, {}, {:?}", self.name, self.thumbnail,
			self.created_at, self.listened_at, self.tracks
		)
	}
}

impl Default for Playlist {
	fn default() -> Self {
		Self {
			id: 0,
			name: "".to_string(),
			list_type: "".to_string(),
			artist: None,
			thumbnail: "".to_string(),
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
			thumbnail: row.get("thumbnail")?,
			created_at: row.get("created_at")?,
			listened_at: row.get("listened_at")?,
			tracks: None,
			sources: None,
		})
	}
}

// impl CreateKey for Playlist {
// 	fn create_key(&self) -> String {
// 		String::from(&self.name)
// 	}
// }

// impl rusqlite::ToSql for Playlist {
// 	fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
// 		self.to_sql()
// 	}
// }

impl GetQuery for Playlist {
	fn get_query(&self, query: SqlQueries) -> String {
		match query {
			SqlQueries::Insert => String::from("
				INSERT INTO playlists (name, list_type, thumbnail, artist)
				VALUES (?, ?, ?, ?);
			"),
			SqlQueries::Select => String::from("SELECT * FROM playlists;"),
			SqlQueries::Update => String::from("
				UPDATE playlists
				SET
					name = ?,
					list = ?,
					thumbnail = ?,
					artist = ?
				WHERE name = ?;
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
			&self.thumbnail as &dyn ToSql,
			&self.artist as &dyn ToSql
		]
	}
}

enum PrivateQueries {
	GetTracks,
	GetArtists,
	GetSources,
}

impl Playlist {
	fn get_private_query(&self, private_query: PrivateQueries) -> String {
		match private_query {
			PrivateQueries::GetTracks => String::from("
				SELECT t.* FROM playlist_tracks pt
	            JOIN tracks t ON t.id = pt.track_id
	            WHERE pt.playlist_id = ?1
			"),
			PrivateQueries::GetArtists => String::from("
				SELECT pa.artist_name FROM playlist_artists pa
             	WHERE pa.playlist_id = ?1
            "),
			PrivateQueries::GetSources => String::from("
				SELECT s.* FROM playlist_sources ps
	            JOIN sources s ON s.id = ps.source_id
	            WHERE ps.playlist_id = ?1
			"),
		}
	}
}

impl SlintConvertable for Playlist {
	fn create_image_from_path(&self) -> slint::Image {
		if let Ok(loaded_image) = slint::Image::load_from_path(
			std::path::Path::new(self.thumbnail.as_str())
		) {
	  		loaded_image
	 	} else {
      		slint::Image::default()
       	}
	}
}

impl ToSlint for Playlist {
	type Output = slint_generatedAppWindow::SlintPlaylist;
	type Overrides = Option<State>;

	fn to_slint(&self, _overrides: Self::Overrides) -> Self::Output {
		let mut sources = Vec::new();
		let mut tracks = Vec::new();
		let mut artist = String::from("");
		let thumbnail = self.create_image_from_path();

		if let Some(s) = &self.sources {
		    sources = s.clone();
		}

		if let Some(t) = &self.tracks {
		    tracks = t.clone();
		}

		if let Some(a) = &self.artist {
			artist = a.clone();
		}

		// if let Some(state) = overrides {
		// 	let converted_tracks = tracks.iter().map(|entry| {
		// 		let o_track = state.find_source_by_id(entry.id);
		// 		if let Some((_, track)) = o_track {
		// 			track.borrow().to_slint(None)
		// 		} else {
		// 			let mut t = Track::to_slint_empty();
		// 			t.title = "Not found".to_string().into();
		// 			t
		// 		}
		// 	});
		// }

		Self::Output {
		    id: self.id,
		    name: slint::SharedString::from(self.name.clone()),
		    artist: slint::SharedString::from(artist),
		    list_type: slint::SharedString::from(self.list_type.clone()),
		    thumbnail: thumbnail,
		    created_at: slint::SharedString::from(self.created_at.clone()),
		    listened_at: slint::SharedString::from(self.listened_at.clone()),
		    sources: convert_to_slint_model(sources),
		    tracks: convert_to_slint_model(tracks),
		}
	}

	fn to_slint_empty() -> Self::Output {
		let result = slint::Image::load_from_path(&std::path::Path::new(""));
		Self::Output {
			id: i32::MAX,
			name: slint::SharedString::from("Nameless playlist"),
			artist: slint::SharedString::from(""),
			list_type: slint::SharedString::from("Playlist"),
			thumbnail: result.unwrap(),
			created_at: slint::SharedString::from(""),
			listened_at: slint::SharedString::from(""),
			sources: convert_to_slint_model(Vec::<Source>::new()),
			tracks: convert_to_slint_model(Vec::<AudioEntry>::new()),
        }
	}
}
