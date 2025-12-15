use custom::ErrorHandler;
use crate::logic::database_types::{source::Source, track::Track};

pub mod database_types;
pub mod audio;
pub mod database;
pub mod queue;
pub mod source;
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

pub fn validate_sources() -> Result<Vec<Track>, ErrorHandler> {
    let source_list = database::get_table::<Source>()?;
    source::validate_sources(source_list)
}
