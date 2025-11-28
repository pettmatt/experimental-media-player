use super::{
    custom::ErrorHandler,
    database::{MediaFile, Source},
};
use lofty::file::AudioFile;
use native_dialog::DialogBuilder;
use std::{
    fmt::Error,
    fs,
    path::{Path, PathBuf},
};

pub fn new_local_source() -> Option<PathBuf> {
    let path: Option<PathBuf> = DialogBuilder::file()
        .set_location("~")
        .open_single_dir()
        .show()
        .unwrap();

    path
}

pub fn read_source(source: PathBuf) -> Result<Vec<MediaFile>, Error> {
    let mut list: Vec<MediaFile> = Vec::new();
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

            let mime_type = match file_extension {
                "ogg" => Some("audio/ogg"),
                "mp3" => Some("audio/mpeg"),
                "mp4" => Some("video/mp4"),
                "webm" => Some("video/webm"),
                _ => None,
            };

            let duration = match file_extension {
                "ogg" => read_audio_file(Path::new(&entry_path)),
                "mp3" => read_audio_file(Path::new(&entry_path)),
                _ => None,
            };

            if mime_type.is_some() {
                // let name_array: Vec<&str> = file_name.split(".").collect();
                // let audio_name = name_array[0];
                let artist = String::from("unknown");
                let path = format!("{:?}", entry_path);
                let id = list.len() as i32;

                if let Some(d) = duration {
                    list.push(MediaFile {
                        id,
                        artist,
                        name: file_name,
                        extension: file_extension.to_string(),
                        path,
                        file_size,
                        duration: d.as_secs_f32() as i32,
                        playing: false,
                    });
                }
            } else {
                println!("Unknown mime_type: {:?}", mime_type);
                println!("Unhandled file_extension: {:?}", file_extension);
            }
        }
    }

    Ok(list)
}

pub fn validate_sources(source_list: Vec<Source>) -> Result<Vec<MediaFile>, ErrorHandler> {
    // Fetch sources, if fetching is done without issues, the sources are valid.
    let mut file_list: Vec<MediaFile> = Vec::new();

    for source in source_list {
        if source.origin == "local" {
            let path = PathBuf::from(source.path);
            let files: Vec<MediaFile> =
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

fn read_audio_file(path: &Path) -> Option<core::time::Duration> {
    if let Ok(file) = lofty::read_from_path(path) {
        let duration = file.properties().duration();
        return Some(duration);
    }

    None
}
