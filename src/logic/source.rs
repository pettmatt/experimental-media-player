use crate::logic::{data_types::{playlist::Playlist, source::Source, track::Track}, database};
use super::{
    custom::ErrorHandler,
};
use lofty::{self, file::TaggedFileExt, file::AudioFile, probe::Probe, tag::Accessor};
use native_dialog::DialogBuilder;
use std::{
    borrow::Cow, fmt::Error, fs, ops::Not, path::{Path, PathBuf}
};

pub fn new_local_source() -> Option<PathBuf> {
    let path: Option<PathBuf> = DialogBuilder::file()
        .set_location("~")
        .open_single_dir()
        .show()
        .unwrap();

    path
}

pub fn read_source(source: PathBuf) -> Result<Vec<Track>, Error> {
    let mut list: Vec<Track> = Vec::new();
    let path = source.as_path();

    let entries = fs::read_dir(path).expect("Couldn't read directory from path");

    for entry in entries {
        let entry = entry.expect("Couldn't get entry");
        let entry_path = entry.path();

        if entry_path.is_dir() {
            let nested_files = read_source(entry_path.clone());

            if let Ok(nf) = nested_files {
                list.extend(nf)
            }
        } else {
            let file_name = entry.file_name().to_string_lossy().to_string();
            println!("Entry_path {:?}", &entry_path);
            let metadata = fs::metadata(&entry_path).expect("Couldn't get metadata");
            let file_size = metadata.len() as i32;

            let file_extension = Path::new(&entry_path)
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("unknown_extension");

            // let mime_type = match file_extension {
            //     "ogg" => Some("audio/ogg"),
            //     "mp3" => Some("audio/mpeg"),
            //     "mp4" => Some("video/mp4"),
            //     "webm" => Some("video/webm"),
            //     _ => None,
            // };

            let duration = match file_extension {
                "ogg" => get_duration(Path::new(&entry_path)),
                "vorbis" => get_duration(Path::new(&entry_path)),
                "mp3" => get_duration(Path::new(&entry_path)),
                "flac" => get_duration(Path::new(&entry_path)),
                "aac" => get_duration(Path::new(&entry_path)),
                "aiff" => get_duration(Path::new(&entry_path)),
                _ => None,
            };

            let path = entry_path.to_string_lossy().to_string();
            let id = list.len() as i32;
            let mut artist = "unknown".to_string();
            let mut title = "".to_string();
            let mut genre = "".to_string();
            let mut year = 0;

            // BUG: Unknown format creates issues
            let file_tag = Probe::open(&path).unwrap().read().unwrap();
            if let Some(tag) = file_tag.primary_tag() {
            	let default = Cow::Borrowed("???");
            	artist = tag.artist().unwrap_or(default.clone()).to_string();
             	title = tag.title().unwrap_or(default.clone()).to_string();
              	genre = tag.genre().unwrap_or(default.clone()).to_string();

              	if let Some(album_name) = tag.album() {
               		let playlists = database::get_table::<Playlist>();

                 	if let Ok(playlists) = playlists {
                 		let album_exists: bool = playlists.iter().any(|pl| &pl.name == &album_name);
                   		if album_exists.not() {
                     		let album = Playlist {
                       			id: 0,
                          		name: album_name.to_string(),
                            	list_type: "album".to_string(),
                            	artist: Some(artist.clone()),
                             	sources: Vec::new(),
                              	image_url: "".to_string(),
                               	created_at: "".to_string(),
                                listened_at: "".to_string(),
                                tracks: Vec::new()
                            };

                   			if let Err(()) = database::add_record::<Playlist>(album) {
                   				println!("Failed to create new album \"{album_name}\".");
                      		}
                     	}
                  	}
                 	// Todo: Create new album playlist if album is found
               	}

               	if let Some(y) = tag.year() {
                	year = y;
                }
            }

            if let Some(d) = duration {
                list.push(Track {
                    id,
                    artist,
                    title,
                    genre,
                    year,
                    extension: file_extension.to_string(),
                    path,
                    file_size,
                    duration: d.as_secs_f32() as i32,
                    playing: false,
                });
            }
        }
    }

    Ok(list)
}

pub fn validate_sources(source_list: Vec<Source>) -> Result<Vec<Track>, ErrorHandler> {
    // Fetch sources, if fetching is done without issues, the sources are valid.
    let mut file_list: Vec<Track> = Vec::new();

    for source in source_list {
        if source.origin == "local" {
            let path = PathBuf::from(source.path);
            let files: Vec<Track> =
                read_source(path).expect("Couldn't validate some media files");

            file_list.extend(files);
        } else {
            // Later on we can add logic to validate other than local sources.
            // At that point probably better to switch if-statement to match.
            println!("Not a local source: {:?}", source);
        }
    }

    Ok(file_list)
}

fn get_duration(path: &Path) -> Option<core::time::Duration> {
    if let Ok(file) = lofty::read_from_path(path) {
        let duration = file.properties().duration();
        return Some(duration);
    }

    None
}
