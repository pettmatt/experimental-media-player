mod api;
mod stream;
use std::{collections::HashMap, fmt::Debug, path::PathBuf, process::{ExitCode, Termination}};
use stream::extract::Extractor;
use api::yt::YouTubeClient;
use crate::api::yt::SearchResults;

pub struct LibResult<T, E>(pub std::result::Result<T, E>);
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

impl<T: Termination, E: Debug> Termination for LibResult<T, E> {
    fn report(self) -> ExitCode {
        match self.0 {
            Ok(val) => val.report(),
            Err(err) => {
                eprintln!("Error: {:?}", err);
                ExitCode::FAILURE
            }
        }
    }
}

pub async fn get_source(url: String) -> Result<PathBuf> {
	let result = match Extractor::new().await {
		Ok(inst) => inst.resolve_audio_url(
			String::from(url)
		).await,
		Err(e) => {
        	eprintln!("Creating Extractor instance failed: {e:?}");
         	Err(e)
    	}
	};

    let path = result?;
    Ok(path)
}

pub enum Fetch {
	Search, Playlists, Channels, ChannelDetails, PlaylistDetails
}

pub async fn search(
	search_term: String, tags: Vec<String>, fetch: Fetch
) -> Result<SearchResults> {
	let api_key = std::env::var("YOUTUBE_API_KEY")?;
    let client = YouTubeClient::new(api_key);

    let mut extra = HashMap::new();
    extra.insert("order".to_string(), "viewCount".to_string());
    extra.insert("relevanceLanguage".to_string(), "en".to_string());
    let formatted = search_term.as_str();

    let result: SearchResults = match fetch {
    	Fetch::Search => SearchResults::Videos(
     		client.search_music(formatted, Some(extra)).await?
     	),
     	Fetch::Playlists => SearchResults::Playlists(
      		client.search_playlists(formatted, None).await?
      	),
      	Fetch::Channels => SearchResults::Channels(
        	client.search_channels(formatted, Some(extra)).await?
        ),
	    Fetch::PlaylistDetails => SearchResults::PlaylistsDetailed(
			client.search_playlists_detailed(formatted, None).await?
		),
	    Fetch::ChannelDetails => {
			if search_term.chars().nth(0) == "@".chars().next() {
				SearchResults::Channel(
					client.get_channel_by_handle(formatted).await?
				)
			} else {
				SearchResults::ChannelsDetailed(
					client.search_channels_detailed(formatted, None).await?
				)
			}
		},
    };

    // for item in results {
    //     println!("{} — {} (https://youtu.be/{})", video.title, video.channel_title, video.video_id);
    // }

    Ok(result)
}
