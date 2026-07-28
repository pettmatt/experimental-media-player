use std::fs;
use std::os::unix::fs::PermissionsExt;
use yt_dlp::model::playlist::Playlist;
// use yt_dlp::model::AudioQuality;
use yt_dlp::Downloader;
use yt_dlp::client::deps::Libraries;
use std::path::PathBuf;

pub struct Extractor {
	downloader: Downloader,
}

impl Extractor {
	pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
		let libraries_dir = PathBuf::from("libs");
		let output_dir = PathBuf::from("output");
		tokio::fs::create_dir_all(&libraries_dir).await?;
		tokio::fs::create_dir_all(&output_dir).await?;

		let yt = libraries_dir.join("yt-dlp");
		let ffmpeg = libraries_dir.join("ffmpeg");

		// if !yt.exists() || !yt.is_file() {
		// 	let _ = std::fs::File::create("libs/yt-dlp");
		// }
		// if !ffmpeg.exists() || !ffmpeg.is_file() {
		// 	let _ = std::fs::File::create("libs/ffmpeg");
		// }

		let libraries = Libraries::new(yt.clone(), ffmpeg.clone());
		for bin in [libraries_dir.join("yt-dlp"), libraries_dir.join("ffmpeg")] {
			let mut permissions = tokio::fs::metadata(&bin).await?.permissions();
			permissions.set_mode(0o755);
			tokio::fs::set_permissions(&bin, permissions).await?;
		}

		let downloader = Downloader::with_new_binaries(libraries_dir, "output")
    		.await?
			.with_timeout(std::time::Duration::from_secs(20))
			.build()
			.await?;

		if !fs::metadata(&yt).is_ok() {
			return Err("yt-dlp binary not found in libs/".into());
		}
		if !fs::metadata(&ffmpeg).is_ok() {
			return Err("ffmpeg binary not found in libs/".into());
		}

		Ok(Self {
			downloader: downloader
		})
	}

	pub async fn search_results(&self, search_term: String, result_amount: usize) -> Result<Playlist, Box<dyn std::error::Error>> {
		let youtube = self.downloader.youtube_extractor();
		let results = youtube.search(&search_term, result_amount).await?;

		Ok(results)
	}

	pub async fn resolve_audio_url(&self, url: String) -> Result<PathBuf, Box<dyn std::error::Error>> {
		let video = self.downloader.fetch_video_infos(url).await?;
		let video_path = self.downloader.download_audio_stream(&video, "audio.mp3").await?;

		Ok(video_path)
	}
}
