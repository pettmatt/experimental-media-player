pub mod ui {
    use std::collections::HashMap;
    use crate::{logic::managment::source::{read_source, read_sources}, AppWindow};

    use super::managment::source;

	pub fn handle_events(app: &AppWindow) {
		// Media elements bottom panel.
		app.on_media_change(move |index: i32| audio_control_events::handle_media_change(index));
		app.on_media_start(move |start: bool| audio_control_events::handle_media_start(start));
		app.on_media_loop(move |create_loop: bool| audio_control_events::handle_media_loop(create_loop));
		app.on_media_mix(move || audio_control_events::handle_media_mix());

		// Settings
		app.on_new_local_source(move || {
			let source = source::new_local_source();

			match source {
				Some(source) => {
					println!("Directory fetched correctly {:?}", source);
					let files = read_source(source).expect("Couldn't fetch all files");
					println!("files read correctly {:?}", files);
					// Add new source to sources.
					// Read through the source on new source added.
					// Check if there is neat way to do this, or do I need to manually call the function here.
					// let list: HashMap<String, Vec<String>> = read_source(result);
				},
				None => println!("Didn't receive a path. Result should be None: {:?}", source)
			}
		});
	}

	mod audio_control_events {
		pub fn handle_media_change(index: i32) {
			let move_to_next_audio = index > 0;
		}

		pub fn handle_media_start(start: bool) {}

		pub fn handle_media_loop(create_loop: bool) {}

		pub fn handle_media_mix() {}
	}
}

pub mod managment {
	pub mod source {
		use std::{fmt::Error, fs, path::{Path, PathBuf}};
		use native_dialog::DialogBuilder;

		#[derive(Debug)]
		pub struct MediaFile {
			name: String,
			author: String,
			path: String,
			extension: String,
			file_size: u64,
		}

		pub fn new_local_source() -> Option<PathBuf> {
			let path: Option<PathBuf> = DialogBuilder::file()
				.set_location("~")
				.open_single_dir()
				.show()
				.unwrap();
			
			// let path: PathBuf = match path {
			// 	Some(path) => path,
			// 	None => return None,
			// };

			path
		}

		pub fn read_source(source: PathBuf) -> Result<Vec<MediaFile>, Error> {
			let mut files: Vec<MediaFile> = Vec::new();
			let path = source.as_path();

			let entries = fs::read_dir(&path).expect("Couldn't read directory from path");

			for entry in entries {
				let entry = entry.expect("Couldn't get entry");
				let entry_path = entry.path();

				if entry_path.is_dir() {
					let nested_files = read_source(entry_path.clone());

					if let Ok(nf) = nested_files {
						files.extend(nf)
					}
				} else {
					let file_name = entry.file_name().to_string_lossy().to_string();
					println!("Entry_path {:?}", &entry_path);
					let metadata = fs::metadata(&entry_path).expect("Couldn't get metadata");
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

					if mime_type != None {
						files.push(
							MediaFile {
								name: file_name,
								author: String::from("None"),
								extension: String::from(file_extension),
								path: String::from(""),
								file_size,
							}
						)
					}
				}
			}

			Ok(files)
		}

		pub fn read_sources() {}

		fn update_index() {}
	}

	mod audio {

	}

	mod database {

	}
}
