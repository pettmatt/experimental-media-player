use std::{collections::HashMap, fmt::Error, fs, path::{Path, PathBuf}};
use native_dialog::DialogBuilder;
use super::{custom::ErrorHandler, database::{MediaFile, Source}};

pub fn new_local_source() -> Option<PathBuf> {
	let path: Option<PathBuf> = DialogBuilder::file()
		.set_location("~")
		.open_single_dir()
		.show()
		.unwrap();

	path
}

pub fn read_source(source: PathBuf) -> Result<HashMap<String, MediaFile>, Error> {
	let mut hashmap: HashMap<String, MediaFile> = HashMap::new();
	let path = source.as_path();

	let entries = fs::read_dir(path).expect("Couldn't read directory from path");

	for entry in entries {
		let entry = entry.expect("Couldn't get entry");
		let entry_path = entry.path();

		if entry_path.is_dir() {
			let nested_files = read_source(entry_path.clone());

			if let Ok(nf) = nested_files {
				hashmap.extend(nf)
			}
		} else {
			let file_name = entry.file_name().to_string_lossy().to_string();
			println!("Entry_path {:?}", &entry_path);
			let metadata = fs::metadata(&entry_path)
				.expect("Couldn't get metadata");
			let file_size = metadata.len();

			let file_extension = Path::new(&entry_path)
				.extension()
				.and_then(|ext| ext.to_str())
				.unwrap_or("unknown_extension");

			let mime_type = match file_extension {
				"mp3" => Some("audio/mpeg"),
				"mp4" => Some("video/mp4"),
				"webm" => Some("video/webm"),
				_ => None
			};

			if mime_type.is_some() {
				let name_array: Vec<&str> = file_name.split(".").collect();
				let audio_name = name_array[0];
				let artist = String::from("unknown");
				let key = format!("{}.{}", audio_name, artist);
				let path = format!("{:?}", entry_path);

				hashmap.entry(key).or_insert(MediaFile {
					artist,
					name: file_name,
					extension: file_extension.to_string(),
					path,
					file_size,
				});
			} else {
				println!("Unknown mime_type: {:?}", mime_type);
				println!("Unhandled file_extension: {:?}", file_extension);
			}
		}
	}

	Ok(hashmap)
}

pub fn validate_sources(source_hashmap: HashMap<String, Source>) -> Result<HashMap<String, MediaFile>, ErrorHandler> {
	// Fetch sources, if fetching is done without issues, the sources are valid.
	let mut file_hashmap: HashMap<String, MediaFile> = HashMap::new();

	for hash_item in source_hashmap {
		if hash_item.1.origin == "local" {
			let source: Source = hash_item.1;
			let path = PathBuf::from(source.path);
			let files: HashMap<String, MediaFile> = read_source(path)
				.expect("Couldn't validate some media files");

			file_hashmap.extend(files);
		} else {
			// Later on we can add logic to validate other than local sources.
			// At that point probably better to switch if-statement to match.
			println!("Not a local source: {:?}", hash_item);
		}
	}

	Ok(file_hashmap)
}