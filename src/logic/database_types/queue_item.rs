use crate::logic::database_types::{Instanceable, Convertable, CreateKey, FromRow, GetQuery, SqlQueries, ToSqlParams};
use rusqlite::{Row, ToSql};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueItem {
	pub media_id: i32,
}

impl Instanceable for QueueItem {
	fn new() -> Self {
		Self { media_id: 0 }
	}
}

impl FromRow for QueueItem {
	fn from_row(row: &Row) -> Result<Self, Box<dyn std::error::Error>> {
		Ok(Self {
			media_id: row.get("media_id")?,
		})
	}
}

impl CreateKey for QueueItem {
	fn create_key(&self) -> String {
		self.media_id.to_string()
	}
}

impl rusqlite::ToSql for QueueItem {
	fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
		self.to_sql()
	}
}

impl GetQuery for QueueItem {
	fn get_query(&self, query: SqlQueries) -> String {
		match query {
			SqlQueries::Insert => String::from("
				INSERT INTO session (media_id)
				VALUES (?);
			"),
			SqlQueries::Select => String::from("SELECT * FROM queue;"),
			SqlQueries::Update => String::from(""),
			SqlQueries::Delete => String::from("
				DELETE FROM session WHERE id = (id)
				VALUES (?);
			"),
		}
	}
}

impl ToSqlParams for QueueItem {
	fn to_sql_params(&self) -> Vec<&dyn ToSql> {
		vec![
			&self.media_id as &dyn ToSql,
		]
	}
}

impl Convertable for QueueItem {
	fn convert_to_string(&mut self) {}
}
