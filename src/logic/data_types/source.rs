use crate::logic::data_types::{Convertable, FromRow, GetQuery, SqlQueries, ToSqlParams};
use rusqlite::{Row, ToSql};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
	pub origin: String,
	pub path: String,
}

impl std::fmt::Display for Source {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}, {}", self.origin, self.path)
	}
}

impl Default for Source {
	fn default() -> Self {
		Self { origin: "".to_string(), path: "".to_string() }
	}
}

impl FromRow for Source {
	fn from_row(row: &Row) -> Result<Self, Box<dyn std::error::Error>> {
		Ok(Self {
			origin: row.get("origin")?,
			path: row.get("path")?,
		})
	}
}

// impl CreateKey for Source {
// 	fn create_key(&self) -> String {
// 		String::from(&self.path)
// 	}
// }

// impl rusqlite::ToSql for Source {
// 	fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
// 		self.to_sql()
// 	}
// }

impl GetQuery for Source {
	fn get_query(&self, query: SqlQueries) -> String {
		match query {
			SqlQueries::Insert => String::from("
				INSERT INTO sources (origin, path)
				VALUES (?, ?);
			"),
			SqlQueries::Select => String::from("SELECT * FROM sources;"),
			SqlQueries::Update => String::from("
				UPDATE sources
				SET
					origin = ?,
					path = ?
				WHERE path = ?;
			"),
			SqlQueries::Delete => String::from("
				DELETE FROM sources WHERE path = ?;
			"),
		}
	}
}

impl ToSqlParams for Source {
	fn to_sql_params(&self) -> Vec<&dyn ToSql> {
		vec![
			&self.origin as &dyn ToSql,
			&self.path as &dyn ToSql,
		]
	}
}

impl Convertable for Source {
	fn convert_to_string(&mut self) {}
}
