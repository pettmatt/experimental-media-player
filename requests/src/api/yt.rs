use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use reqwest::Client;
use serde::Deserialize;

const SEARCH_ENDPOINT: &str = "https://www.googleapis.com/youtube/v3/search";
const PLAYLISTS_ENDPOINT: &str = "https://www.googleapis.com/youtube/v3/playlists";
const CHANNELS_ENDPOINT: &str = "https://www.googleapis.com/youtube/v3/channels";

const MUSIC_CATEGORY_ID: &str = "10";

pub enum SearchResults {
    Videos(Vec<MusicVideo>),
    Playlists(Vec<PlaylistSearchResult>),
    PlaylistsDetailed(Vec<Playlist>),
    Channels(Vec<ChannelSearchResult>),
    ChannelsDetailed(Vec<Channel>),
    Channel(Option<Channel>),
}

#[derive(Debug)]
pub enum YouTubeApiError {
    Http(reqwest::Error),
    Api { status: u16, message: String },
}

impl fmt::Display for YouTubeApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            YouTubeApiError::Http(e) => write!(f, "HTTP request failed: {e}"),
            YouTubeApiError::Api { status, message } => {
                write!(f, "YouTube API returned {status}: {message}")
            }
        }
    }
}

impl Error for YouTubeApiError {}

impl From<reqwest::Error> for YouTubeApiError {
    fn from(e: reqwest::Error) -> Self {
        YouTubeApiError::Http(e)
    }
}

#[derive(Debug, Clone)]
pub struct MusicVideo {
    pub video_id: String,
    pub title: String,
    pub channel_title: String,
    pub description: String,
    pub published_at: String,
    pub thumbnail_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PlaylistSearchResult {
    pub playlist_id: String,
    pub title: String,
    pub channel_title: String,
    pub description: String,
    pub published_at: String,
    pub thumbnail_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Playlist {
    pub playlist_id: String,
    pub title: String,
    pub channel_title: String,
    pub description: String,
    pub published_at: String,
    pub thumbnail_url: Option<String>,
    pub item_count: Option<u32>,
    pub privacy_status: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ChannelSearchResult {
    pub channel_id: String,
    pub title: String,
    pub description: String,
    pub published_at: String,
    pub thumbnail_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Channel {
    pub channel_id: String,
    pub title: String,
    pub description: String,
    pub custom_url: Option<String>,
    pub country: Option<String>,
    pub published_at: String,
    pub thumbnail_url: Option<String>,
    pub subscriber_count: Option<u64>,
    pub hidden_subscriber_count: Option<bool>,
    pub video_count: Option<u64>,
    pub view_count: Option<u64>,
    pub uploads_playlist_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SearchListResponse {
    items: Vec<SearchResultItem>,
    #[serde(rename = "nextPageToken")]
    next_page_token: Option<String>,
    error: Option<ApiErrorBody>,
}

#[derive(Debug, Deserialize)]
struct ApiErrorBody {
    code: u16,
    message: String,
}

#[derive(Debug, Deserialize)]
struct SearchResultItem {
    id: SearchResultId,
    snippet: SearchResultSnippet,
}

#[derive(Debug, Deserialize)]
struct SearchResultId {
	#[serde(rename = "videoId")]
    video_id: Option<String>,
    #[serde(rename = "playlistId")]
    playlist_id: Option<String>,
    #[serde(rename = "channelId")]
    channel_id: Option<String>,
}

// #[derive(Debug, Deserialize)]
// struct SearchResultId {
// 	title: String,
//     description: String,
//     #[serde(rename = "channelTitle")]
//     channel_title: String,
//     #[serde(rename = "publishedAt")]
//     published_at: String,
//     thumbnails: Thumbnails,
// }

#[derive(Debug, Deserialize)]
struct SearchResultSnippet {
    title: String,
    description: String,
    #[serde(rename = "channelTitle")]
    channel_title: String,
    #[serde(rename = "publishedAt")]
    published_at: String,
    thumbnails: Thumbnails,
}

#[derive(Debug, Deserialize)]
struct Thumbnails {
    default: Option<Thumbnail>,
    medium: Option<Thumbnail>,
    high: Option<Thumbnail>,
}

#[derive(Debug, Deserialize)]
struct Thumbnail {
    url: String,
}

#[derive(Debug, Deserialize)]
struct PlaylistListResponse {
    items: Vec<PlaylistResource>,
    error: Option<ApiErrorBody>,
}

#[derive(Debug, Deserialize)]
struct PlaylistResource {
    id: String,
    snippet: PlaylistSnippet,
    #[serde(rename = "contentDetails")]
    content_details: Option<PlaylistContentDetails>,
    status: Option<PlaylistStatus>,
}

#[derive(Debug, Deserialize)]
struct PlaylistSnippet {
    title: String,
    description: String,
    #[serde(rename = "channelTitle")]
    channel_title: String,
    #[serde(rename = "publishedAt")]
    published_at: String,
    thumbnails: Thumbnails,
}

#[derive(Debug, Deserialize)]
struct PlaylistContentDetails {
    #[serde(rename = "itemCount")]
    item_count: u32,
}

#[derive(Debug, Deserialize)]
struct PlaylistStatus {
    #[serde(rename = "privacyStatus")]
    privacy_status: String,
}

#[derive(Debug, Deserialize)]
struct ChannelListResponse {
    items: Vec<ChannelResource>,
    error: Option<ApiErrorBody>,
}

#[derive(Debug, Deserialize)]
struct ChannelResource {
    id: String,
    snippet: ChannelSnippet,
    statistics: Option<ChannelStatistics>,
    #[serde(rename = "contentDetails")]
    content_details: Option<ChannelContentDetails>,
}

#[derive(Debug, Deserialize)]
struct ChannelSnippet {
    title: String,
    description: String,
    #[serde(rename = "customUrl")]
    custom_url: Option<String>,
    #[serde(rename = "publishedAt")]
    published_at: String,
    thumbnails: Thumbnails,
    country: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ChannelStatistics {
    #[serde(rename = "viewCount")]
    view_count: Option<String>,
    #[serde(rename = "subscriberCount")]
    subscriber_count: Option<String>,
    #[serde(rename = "hiddenSubscriberCount")]
    hidden_subscriber_count: Option<bool>,
    #[serde(rename = "videoCount")]
    video_count: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ChannelContentDetails {
    #[serde(rename = "relatedPlaylists")]
    related_playlists: Option<RelatedPlaylists>,
}

#[derive(Debug, Deserialize)]
struct RelatedPlaylists {
    uploads: Option<String>,
}

pub struct YouTubeClient {
    api_key: String,
    http: Client,
}

impl YouTubeClient {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            http: Client::new(),
        }
    }

    pub async fn search_music(
        &self,
        query: &str,
        extra_params: Option<HashMap<String, String>>,
    ) -> Result<Vec<MusicVideo>, YouTubeApiError> {
        let mut params: HashMap<String, String> = HashMap::new();
        params.insert("part".into(), "snippet".into());
        params.insert("q".into(), query.into());
        params.insert("type".into(), "video".into());
        params.insert("videoCategoryId".into(), MUSIC_CATEGORY_ID.into());
        params.insert("maxResults".into(), "25".into());
        params.insert("key".into(), self.api_key.clone());

        if let Some(extra) = extra_params {
            for (k, v) in extra {
                params.insert(k, v);
            }
        }

        let response = self.http
            .get(SEARCH_ENDPOINT)
            .query(&params)
            .send()
            .await?;

        let status = response.status();
        let body: SearchListResponse = response.json().await?;

        if let Some(err) = body.error {
            return Err(YouTubeApiError::Api {
                status: err.code,
                message: err.message,
            });
        }
        if !status.is_success() {
            return Err(YouTubeApiError::Api {
                status: status.as_u16(),
                message: "request failed with no error body".into(),
            });
        }

        let videos = body.items
            .into_iter()
            .filter_map(|item| {
                let video_id = item.id.video_id?;
                let thumb = item
                    .snippet
                    .thumbnails
                    .high
                    .or(item.snippet.thumbnails.medium)
                    .or(item.snippet.thumbnails.default)
                    .map(|t| t.url);

                Some(MusicVideo {
                    video_id,
                    title: item.snippet.title,
                    channel_title: item.snippet.channel_title,
                    description: item.snippet.description,
                    published_at: item.snippet.published_at,
                    thumbnail_url: thumb,
                })
            })
            .collect();

        // `next_page_token` is available on `body` if we want to add pagination
        // later (pass it through as `extra_params["pageToken"]` on the next call).
        // let _ = body.next_page_token;

        Ok(videos)
    }

    pub async fn search_playlists(
        &self,
        query: &str,
        extra_params: Option<HashMap<String, String>>,
    ) -> Result<Vec<PlaylistSearchResult>, YouTubeApiError> {
        let mut params: HashMap<String, String> = HashMap::new();
        params.insert("part".into(), "snippet".into());
        params.insert("q".into(), query.into());
        params.insert("type".into(), "playlist".into());
        params.insert("maxResults".into(), "25".into());
        params.insert("key".into(), self.api_key.clone());

        if let Some(extra) = extra_params {
            for (k, v) in extra {
                params.insert(k, v);
            }
        }

        let response = self.http
            .get(SEARCH_ENDPOINT)
            .query(&params)
            .send()
            .await?;

        let status = response.status();
        let body: SearchListResponse = response.json().await?;

        if let Some(err) = body.error {
            return Err(YouTubeApiError::Api {
                status: err.code,
                message: err.message,
            });
        }
        if !status.is_success() {
            return Err(YouTubeApiError::Api {
                status: status.as_u16(),
                message: "request failed with no error body".into(),
            });
        }

        let playlists = body.items
            .into_iter()
            .filter_map(|item| {
                let thumb = item
                    .snippet
                    .thumbnails
                    .high
                    .or(item.snippet.thumbnails.medium)
                    .or(item.snippet.thumbnails.default)
                    .map(|t| t.url);

                item.id.playlist_id.map(|playlist_id| PlaylistSearchResult {
                    playlist_id,
                    title: item.snippet.title,
                    channel_title: item.snippet.channel_title,
                    description: item.snippet.description,
                    published_at: item.snippet.published_at,
                    thumbnail_url: thumb,
                })
            })
            .collect();

        Ok(playlists)
    }

    pub async fn get_playlists(
        &self,
        playlist_ids: &[String],
        extra_params: Option<HashMap<String, String>>,
    ) -> Result<Vec<Playlist>, YouTubeApiError> {
        let mut params: HashMap<String, String> = HashMap::new();
        params.insert("part".into(), "snippet,contentDetails,status".into());
        params.insert("id".into(), playlist_ids.join(","));
        params.insert("maxResults".into(), "50".into());
        params.insert("key".into(), self.api_key.clone());

        if let Some(extra) = extra_params {
            for (k, v) in extra {
                params.insert(k, v);
            }
        }

        let response = self
            .http
            .get(PLAYLISTS_ENDPOINT)
            .query(&params)
            .send()
            .await?;

        let status = response.status();
        let body: PlaylistListResponse = response.json().await?;

        if let Some(err) = body.error {
            return Err(YouTubeApiError::Api {
                status: err.code,
                message: err.message,
            });
        }
        if !status.is_success() {
            return Err(YouTubeApiError::Api {
                status: status.as_u16(),
                message: "request failed with no error body".into(),
            });
        }

        let playlists = body.items
            .into_iter()
            .map(|item| {
                let thumb = item
                    .snippet
                    .thumbnails
                    .high
                    .or(item.snippet.thumbnails.medium)
                    .or(item.snippet.thumbnails.default)
                    .map(|t| t.url);

                Playlist {
                    playlist_id: item.id,
                    title: item.snippet.title,
                    channel_title: item.snippet.channel_title,
                    description: item.snippet.description,
                    published_at: item.snippet.published_at,
                    thumbnail_url: thumb,
                    item_count: item.content_details.map(|c| c.item_count),
                    privacy_status: item.status.map(|s| s.privacy_status),
                }
            })
            .collect();

        Ok(playlists)
    }

    pub async fn search_playlists_detailed(
        &self,
        query: &str,
        search_extra_params: Option<HashMap<String, String>>,
    ) -> Result<Vec<Playlist>, YouTubeApiError> {
        let found = self.search_playlists(query, search_extra_params).await?;
        if found.is_empty() {
            return Ok(Vec::new());
        }

        let ids: Vec<String> = found.into_iter().map(|p| p.playlist_id).collect();
        self.get_playlists(&ids, None).await
    }

    pub async fn search_channels(
        &self,
        query: &str,
        extra_params: Option<HashMap<String, String>>,
    ) -> Result<Vec<ChannelSearchResult>, YouTubeApiError> {
        let mut params: HashMap<String, String> = HashMap::new();
        params.insert("part".into(), "snippet".into());
        params.insert("q".into(), query.into());
        params.insert("type".into(), "channel".into());
        params.insert("maxResults".into(), "25".into());
        params.insert("key".into(), self.api_key.clone());

        if let Some(extra) = extra_params {
            for (k, v) in extra {
                params.insert(k, v);
            }
        }

        let response = self.http
            .get(SEARCH_ENDPOINT)
            .query(&params)
            .send()
            .await?;

        let status = response.status();
        let body: SearchListResponse = response.json().await?;

        if let Some(err) = body.error {
            return Err(YouTubeApiError::Api {
                status: err.code,
                message: err.message,
            });
        }
        if !status.is_success() {
            return Err(YouTubeApiError::Api {
                status: status.as_u16(),
                message: "request failed with no error body".into(),
            });
        }

        let channels = body.items
            .into_iter()
            .filter_map(|item| {
                let thumb = item.snippet.thumbnails.high
                    .or(item.snippet.thumbnails.medium)
                    .or(item.snippet.thumbnails.default)
                    .map(|t| t.url);

                item.id.channel_id.map(|channel_id| ChannelSearchResult {
                    channel_id,
                    title: item.snippet.title,
                    description: item.snippet.description,
                    published_at: item.snippet.published_at,
                    thumbnail_url: thumb,
                })
            })
            .collect();

        Ok(channels)
    }

    pub async fn get_channels(
        &self,
        channel_ids: &[String],
        extra_params: Option<HashMap<String, String>>,
    ) -> Result<Vec<Channel>, YouTubeApiError> {
        let mut params: HashMap<String, String> = HashMap::new();
        params.insert("id".into(), channel_ids.join(","));
        self.fetch_channels(params, extra_params).await
    }

    pub async fn get_channel_by_handle(
        &self,
        handle: &str,
    ) -> Result<Option<Channel>, YouTubeApiError> {
        let mut params: HashMap<String, String> = HashMap::new();
        params.insert("forHandle".into(), handle.into());
        let mut results = self.fetch_channels(params, None).await?;
        Ok(if results.is_empty() {
            None
        } else {
            Some(results.remove(0))
        })
    }

    async fn fetch_channels(
        &self,
        mut params: HashMap<String, String>,
        extra_params: Option<HashMap<String, String>>,
    ) -> Result<Vec<Channel>, YouTubeApiError> {
        params.insert(
            "part".into(),
            "snippet,statistics,contentDetails".into(),
        );
        params.insert("maxResults".into(), "50".into());
        params.insert("key".into(), self.api_key.clone());

        if let Some(extra) = extra_params {
            for (k, v) in extra {
                params.insert(k, v);
            }
        }

        let response = self.http
            .get(CHANNELS_ENDPOINT)
            .query(&params)
            .send()
            .await?;

        let status = response.status();
        let body: ChannelListResponse = response.json().await?;

        if let Some(err) = body.error {
            return Err(YouTubeApiError::Api {
                status: err.code,
                message: err.message,
            });
        }
        if !status.is_success() {
            return Err(YouTubeApiError::Api {
                status: status.as_u16(),
                message: "request failed with no error body".into(),
            });
        }

        let channels = body.items
            .into_iter()
            .map(|item| {
                let thumb = item.snippet.thumbnails.high
                    .or(item.snippet.thumbnails.medium)
                    .or(item.snippet.thumbnails.default)
                    .map(|t| t.url);

                let stats = item.statistics;
                let uploads_playlist_id = item
                    .content_details
                    .and_then(|c| c.related_playlists)
                    .and_then(|r| r.uploads);

                Channel {
                    channel_id: item.id,
                    title: item.snippet.title,
                    description: item.snippet.description,
                    custom_url: item.snippet.custom_url,
                    country: item.snippet.country,
                    published_at: item.snippet.published_at,
                    thumbnail_url: thumb,
                    subscriber_count: stats
                        .as_ref()
                        .and_then(|s| s.subscriber_count.as_ref())
                        .and_then(|s| s.parse().ok()),
                    hidden_subscriber_count: stats
                        .as_ref()
                        .and_then(|s| s.hidden_subscriber_count),
                    video_count: stats
                        .as_ref()
                        .and_then(|s| s.video_count.as_ref())
                        .and_then(|s| s.parse().ok()),
                    view_count: stats
                        .as_ref()
                        .and_then(|s| s.view_count.as_ref())
                        .and_then(|s| s.parse().ok()),
                    uploads_playlist_id,
                }
            })
            .collect();

        Ok(channels)
    }

    pub async fn search_channels_detailed(
        &self,
        query: &str,
        search_extra_params: Option<HashMap<String, String>>,
    ) -> Result<Vec<Channel>, YouTubeApiError> {
        let found = self.search_channels(query, search_extra_params).await?;
        if found.is_empty() {
            return Ok(Vec::new());
        }

        let ids: Vec<String> = found.into_iter().map(|c| c.channel_id).collect();
        self.get_channels(&ids, None).await
    }
}


// --- Example usage ---

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let api_key = std::env::var("YOUTUBE_API_KEY")?;
    let client = YouTubeClient::new(api_key);

    let mut extra = HashMap::new();
    extra.insert("order".to_string(), "viewCount".to_string());
    extra.insert("relevanceLanguage".to_string(), "en".to_string());

    let results = client.search_music("lofi hip hop", Some(extra)).await?;
    for video in results {
        println!("{} — {} (https://youtu.be/{})", video.title, video.channel_title, video.video_id);
    }

    // Playlist search — lightweight (snippet only), 1 quota unit:
    let playlists = client.search_playlists("lofi hip hop", None).await?;
    for p in &playlists {
        println!("{} — {} ({})", p.title, p.channel_title, p.playlist_id);
    }

    // Playlist search with full detail (item count, privacy) — 2 quota units:
    let detailed = client.search_playlists_detailed("lofi hip hop", None).await?;
    for p in &detailed {
        println!("{} — {:?} items, {:?}", p.title, p.item_count, p.privacy_status);
    }

    // Channel search, with detail (subscriber count, uploads playlist):
    let channels = client.search_channels_detailed("lofi girl", None).await?;
    for c in &channels {
        println!("{} — {:?} subs, uploads: {:?}", c.title, c.subscriber_count, c.uploads_playlist_id);
    }

    // Exact handle lookup — no search needed, 1 quota unit:
    if let Some(c) = client.get_channel_by_handle("@LofiGirl").await? {
        println!("Found: {} ({})", c.title, c.channel_id);
    }

    Ok(())
}
