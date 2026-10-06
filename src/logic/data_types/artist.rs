use crate::logic::data_types::{CreateKey, FromRow, GetQuery, Instanceable, SlintConvertable, SqlQueries, ToSqlParams};
use rusqlite::{Row, ToSql};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Artist {
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
			name: "".to_string(),
			year: 0,
			thumbnail: "".to_string(),
		}
	}
}

impl FromRow for Artist {
	fn from_row(row: &Row) -> Result<Self, Box<dyn std::error::Error>> {
		Ok(Self {
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

impl rusqlite::ToSql for Artist {
	fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
		self.to_sql()
	}
}

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
