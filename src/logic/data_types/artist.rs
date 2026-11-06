use std::{cell::RefCell, rc::Rc};

use crate::{logic::data_types::{CreateKey, FromRow, GetQuery, Instanceable, SlintConvertable, SqlQueries, ToSlint, ToSqlParams, state::State, track::Track}, slint_generatedAppWindow};
use rusqlite::{Row, ToSql};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Artist {
	pub id: i32,
	pub name: String,
	pub year: i32,
	pub thumbnail: String
}

impl SlintConvertable for Artist {
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

impl Instanceable for Artist {
	fn new() -> Self {
		Self {
			id: -1,
			name: "".to_string(),
			year: 0,
			thumbnail: "".to_string(),
		}
	}
}

impl FromRow for Artist {
	fn from_row(row: &Row) -> Result<Self, Box<dyn std::error::Error>> {
		Ok(Self {
			id: row.get("id")?,
			name: row.get("name")?,
			year: row.get("year")?,
			thumbnail: row.get("thumbnail")?,
		})
	}
}

impl CreateKey for Artist {
	fn create_key(&self) -> String {
		String::from(&self.name)
	}
}

// impl rusqlite::ToSql for Artist {
// 	fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
// 		self.to_sql()
// 	}
// }

impl GetQuery for Artist {
	fn get_query(&self, query: SqlQueries) -> String {
		match query {
			SqlQueries::Insert => String::from("
				INSERT INTO artists (name, year, thumbnail)
				VALUES (?, ?, ?);
			"),
			SqlQueries::Select => String::from("SELECT * FROM artists;"),
			SqlQueries::Update => String::from("
				UPDATE artists
				SET
					name = (name),
					year = (year),
					thumbnail = (thumbnail),
				WHERE name = (name)
				VALUES (?, ?, ?);
			"),
			SqlQueries::Delete => String::from("
				DELETE FROM artists WHERE name = (name)
				VALUES (?);
			"),
		}
	}
}

impl ToSqlParams for Artist {
	fn to_sql_params(&self) -> Vec<&dyn ToSql> {
		vec![
			&self.name as &dyn ToSql,
			&self.year as &dyn ToSql,
			&self.thumbnail as &dyn ToSql,
		]
	}
}

impl ToSlint for Artist {
	type Output = slint_generatedAppWindow::SlintArtist;
	type Overrides = Vec<Rc<RefCell<Track>>>; // List of tracks usually state.index

	fn to_slint(&self, overrides: Self::Overrides) -> Self::Output {
		let genres: Vec<String> = overrides
			.iter()
			.filter(|t| t.borrow().artist.contains(&self.name))
			.map(|t| t.borrow().genres.clone())
			.flatten()
			.collect();
		let converted_genres: slint::VecModel<slint::SharedString> = genres
			.iter().map(|g| slint::SharedString::from(g.as_str())).collect();
		let genre_model: slint::ModelRc<slint::SharedString> = slint::ModelRc::new(
			slint::VecModel::from(converted_genres)
		);

		Self::Output {
			id: self.id,
            name: self.name.clone().into(),
            genres: genre_model,
            year: self.year,
		}
	}

	fn to_slint_empty() -> Self::Output {
		Self::Output {
            id: i32::MAX,
            name: slint::SharedString::from(""),
            genres: slint::ModelRc::default(),
            year: 0,
        }
	}
}
