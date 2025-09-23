use custom::ErrorHandler;

mod audio;
mod source;
pub mod database;
pub mod queue;
pub mod ui_events;
mod custom {
	use thiserror::Error;

	#[derive(Error, Debug)]
	pub enum ErrorHandler {
		#[error("SQLite error: {0}")]
		Sqlite(#[from] rusqlite::Error),
		#[error("IO error: {0}")]
		Io(#[from] std::io::Error),
	}
}

// Note: Private modules shouldn't interact with other modules. They should be independent.

pub fn validate_sources() -> Result<Vec<database::MediaFile>, ErrorHandler> {
	let source_list = database::get_table::<database::Source>()?;
	source::validate_sources(source_list)
}
