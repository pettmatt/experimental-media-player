pub mod state;
pub mod playlist;
pub mod queue_item;
pub mod timeline;
pub mod setting;
pub mod source;
pub mod artist;
pub mod track;

use rusqlite::{Row, ToSql};

pub trait Instanceable {
	fn new() -> Self;
}

pub trait FromRow {
	fn from_row(row: &Row) -> Result<Self, Box<dyn std::error::Error>> where Self: Sized;
}

pub trait CreateKey {
	fn create_key(&self) -> String;
}

pub trait GetQuery {
	fn get_query(&self, query: SqlQueries) -> String;
}

pub trait ToSqlParams {
	fn to_sql_params(&self) -> Vec<&dyn ToSql>;
}

pub trait Convertable {
	fn convert_to_string(&mut self);
}

pub trait SlintConvertable {
	fn create_image_from_path(&self) -> slint::Image;
}

pub enum SqlQueries {
	Insert,
	Select,
	Update,
	Delete
}
