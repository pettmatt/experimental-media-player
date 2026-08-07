use crate::logic::data_types::{Convertable, CreateKey, FromRow, GetQuery, Instanceable, SqlQueries, ToSqlParams};
use rusqlite::{Row, ToSql};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Setting {
	pub name: String,
	pub value: String,
	pub default_value: String,
	pub created_at: String,
	pub updated_at: String,
}

impl Setting {
	pub fn is_numeric(value: &String) -> Option<i32> {
		let is_numeric = value.chars().all(|c|
			c.is_ascii_digit()) || (value.starts_with("-") && value[1..].chars().all(|c| c.is_ascii_digit())
		) ||
		(value.starts_with("+") && value[1..].chars().all(|c| c.is_ascii_digit()));

		if is_numeric {
			value.parse::<i32>().ok()
		} else {
			None
		}
	}
}

impl std::fmt::Display for Setting {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f, "{}, {}, {}, {}, {}", self.name, self.value,
			self.default_value, self.created_at, self.updated_at
		)
	}
}

impl Instanceable for Setting {
	fn new() -> Self {
		Self {
			name: "".to_string(),
			value: "".to_string(),
			default_value: "".to_string(),
			created_at: "".to_string(),
			updated_at: "".to_string(),
		}
	}
}

impl FromRow for Setting {
	fn from_row(row: &Row) -> Result<Self, Box<dyn std::error::Error>> {
		Ok(Self {
			name: row.get("name")?,
			value: row.get("value")?,
			default_value: row.get("default_value")?,
			created_at: row.get("created_at")?,
			updated_at: row.get("updated_at")?,
		})
	}
}

impl CreateKey for Setting {
	fn create_key(&self) -> String {
		String::from(&self.name)
	}
}

impl rusqlite::ToSql for Setting {
	fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
		self.to_sql()
	}
}

impl GetQuery for Setting {
	fn get_query(&self, query: SqlQueries) -> String {
		match query {
			SqlQueries::Insert => String::from("
				INSERT INTO settings (name, value, default_value)
				VALUES (?, ?, ?);
			"),
			SqlQueries::Select => String::from("SELECT * FROM settings;"),
			SqlQueries::Update => String::from("
				UPDATE settings
				SET
					name = (name),
					value = (value),
					default_value = (default_value),
				WHERE name = (name)
				VALUES (?, ?, ?);
			"),
			SqlQueries::Delete => String::from("
				DELETE FROM settings WHERE name = (name)
				VALUES (?);
			"),
		}
	}
}

impl ToSqlParams for Setting {
	fn to_sql_params(&self) -> Vec<&dyn ToSql> {
		vec![
			&self.name as &dyn ToSql,
			&self.value as &dyn ToSql,
			&self.default_value as &dyn ToSql,
		]
	}
}

impl Convertable for Setting {
	fn convert_to_string(&mut self) {}
}
