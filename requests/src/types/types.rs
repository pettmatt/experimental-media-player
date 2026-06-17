use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod YouTube {
    // (if needed)
    #[derive(Debug, Serialize, Deserialize)]
    pub enum PrivacyStatus {
        Public,
        Private,
        Unlisted,
        #[serde(other)]
        Unknown,
    }

    // (if needed)
    #[derive(Debug, Serialize, Deserialize)]
    pub enum PodcastStatus {
        #[serde(rename = "PODCAST_STATUS_UNSPECIFIED")]
        Unspecified,
        #[serde(rename = "PODCAST_STATUS_ACTIVE")]
        Active,
        #[serde(rename = "PODCAST_STATUS_INACTIVE")]
        Inactive,
        #[serde(other)]
        Unknown,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Thumbnail {
        pub url: String,
        pub width: u32,
        pub height: u32,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Snippet {
        pub published_at: String, // can be changed into chrono::DateTime for proper datetime handling
        pub channel_id: String,
        pub title: String,
        pub description: String,
        #[serde(rename = "thumbnails")]
        pub thumbnails: HashMap<String, Thumbnail>,
        pub channel_title: String,
        #[serde(rename = "defaultLanguage")]
        pub default_language: Option<String>,
        pub localized: Option<Localized>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Localized {
        pub title: String,
        pub description: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Status {
        #[serde(rename = "privacyStatus")]
        pub privacy_status: String, // can be changed to PrivacyStatus enum
        #[serde(rename = "podcastStatus")]
        pub podcast_status: Option<PodcastStatus>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct ContentDetails {
        #[serde(rename = "itemCount")]
        pub item_count: u32,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Player {
        #[serde(rename = "embedHtml")]
        pub embed_html: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Localization {
        pub title: String,
        pub description: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct SearchResultId {
        #[serde(rename = "kind")]
        pub kind: String,
        #[serde(rename = "videoId")]
        pub video_id: Option<String>,  // Optional
        #[serde(rename = "channelId")]
        pub channel_id: Option<String>,
        #[serde(rename = "playlistId")]
        pub playlist_id: Option<String>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Thumbnail {
        pub url: String,
        pub width: u32,
        pub height: u32,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct SearchResultSnippet {
        #[serde(rename = "publishedAt")]
        pub published_at: String,  // can be changed into chrono::DateTime for proper datetime handling
        #[serde(rename = "channelId")]
        pub channel_id: String,
        pub title: String,
        pub description: String,
        pub thumbnails: HashMap<String, Thumbnail>,  // defaults: "default", "medium"
        #[serde(rename = "channelTitle")]
        pub channel_title: String,
        #[serde(rename = "liveBroadcastContent")]
        pub live_broadcast_content: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct YouTubePlaylist {
        #[serde(rename = "kind")]
        pub kind: String,
        #[serde(rename = "etag")]
        pub etag: String,
        #[serde(rename = "id")]
        pub id: String,
        pub snippet: Snippet,
        pub status: Status,
        #[serde(rename = "contentDetails")]
        pub content_details: ContentDetails,
        pub player: Player,
        #[serde(rename = "localizations")]
        pub localizations: Option<HashMap<String, Localization>>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub struct YouTubeSearchResult {
        #[serde(rename = "kind")]
        pub kind: String,
        #[serde(rename = "etag")]
        pub etag: String,
        pub id: SearchResultId,
        pub snippet: SearchResultSnippet,
    }
}
