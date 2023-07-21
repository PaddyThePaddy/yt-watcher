#![allow(dead_code)]
pub mod structs;
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::{self, StatusCode};
use structs::*;

#[derive(Debug, PartialEq, Eq)]
pub enum YtApiError {
    RequestFailed(Option<StatusCode>),
    DeserializeFailed(String),
    InvalidParameter,
    NotFound,
}

static CHANNEL_ID_PATTERNS: [(Lazy<Regex>, usize); 4] = [
    (
        Lazy::new(|| {
            Regex::new(r#"<link rel="canonical" href="https://www\.youtube\.com/channel/(.+?)">"#)
                .unwrap()
        }),
        1,
    ),
    (
        Lazy::new(|| {
            Regex::new(
                r#"<meta property="og:url" content="https://www\.youtube\.com/channel/(.+?)">"#,
            )
            .unwrap()
        }),
        1,
    ),
    (
        Lazy::new(|| Regex::new(r#"<meta itemprop="identifier" content="(.+?)">"#).unwrap()),
        1,
    ),
    (
        Lazy::new(|| {
            Regex::new(r#"<link rel="alternate" type="application/rss\+xml" title="RSS" href="https://www\.youtube\.com/feeds/videos\.xml\?channel_id=(.+?)">"#).unwrap()
        }),
        1,
    ),
];

pub async fn get_channel_id_by_url(url: &str) -> Result<String, YtApiError> {
    let channel_page_src = reqwest::get(url)
        .await
        .map_err(|e| YtApiError::RequestFailed(e.status()))?
        .error_for_status()
        .map_err(|e| YtApiError::RequestFailed(e.status()))?
        .text()
        .await
        .map_err(|e| YtApiError::DeserializeFailed(format!("{}", e.without_url())))?;

    for (pattern, grp) in CHANNEL_ID_PATTERNS.iter() {
        if let Some(cap) = pattern.captures(&channel_page_src) {
            if let Some(id) = cap.get(*grp) {
                return Ok(id.as_str().to_string());
            }
        }
    }

    return Err(YtApiError::NotFound);
}

pub async fn try_youtube_id(query: &str) -> String {
    if let Ok(id) = get_channel_id_by_url(&format!(
        "https://www.youtube.com/@{}",
        query
            .trim_start_matches("https://www.youtube.com/@")
            .trim_start_matches("@")
    ))
    .await
    {
        return id;
    }

    return query
        .trim_start_matches("https://www.youtube.com/channel/")
        .to_string();
}

pub struct PagedResponse<T> {
    pub next_page_token: Option<String>,
    pub prev_page_token: Option<String>,
    pub value: Vec<T>,
}

#[derive(Default)]
pub struct GetChannelParts {
    _audit_details: bool,
    _branding_settings: bool,
    _content_details: bool,
    _content_owner_details: bool,
    _id: bool,
    _localizations: bool,
    _snippet: bool,
    _statistics: bool,
    _status: bool,
    _topic_details: bool,
}

impl GetChannelParts {
    pub fn build(&self) -> String {
        let mut parts = vec![];
        if self._audit_details {
            parts.push("auditDetails".to_string());
        }
        if self._branding_settings {
            parts.push("brandingSettings".to_string());
        }
        if self._content_details {
            parts.push("contentDetails".to_string());
        }
        if self._content_owner_details {
            parts.push("contentOwnerDetails".to_string());
        }
        if self._id {
            parts.push("id".to_string());
        }
        if self._localizations {
            parts.push("localizations".to_string());
        }
        if self._snippet {
            parts.push("snippet".to_string());
        }
        if self._statistics {
            parts.push("statistics".to_string());
        }
        if self._status {
            parts.push("status".to_string());
        }
        if self._topic_details {
            parts.push("topicDetails".to_string());
        }
        parts.join(",")
    }

    pub fn audit_details(mut self) -> Self {
        self._audit_details = true;
        self
    }
    pub fn branding_settings(mut self) -> Self {
        self._branding_settings = true;
        self
    }
    pub fn content_details(mut self) -> Self {
        self._content_details = true;
        self
    }
    pub fn content_owner_details(mut self) -> Self {
        self._branding_settings = true;
        self
    }
    pub fn id(mut self) -> Self {
        self._id = true;
        self
    }
    pub fn localizations(mut self) -> Self {
        self._localizations = true;
        self
    }
    pub fn snippet(mut self) -> Self {
        self._snippet = true;
        self
    }
    pub fn statistics(mut self) -> Self {
        self._statistics = true;
        self
    }
    pub fn status(mut self) -> Self {
        self._status = true;
        self
    }
    pub fn topic_details(mut self) -> Self {
        self._topic_details = true;
        self
    }
    fn is_none(&self) -> bool {
        !self._audit_details
            && !self._branding_settings
            && !self._content_details
            && !self._content_owner_details
            && !self._id
            && !self._localizations
            && !self._snippet
            && !self._statistics
            && !self._status
            && !self._topic_details
    }
}

pub async fn get_all_channels(
    ids: &[&str],
    parts: &GetChannelParts,
    key: &str,
) -> Result<Vec<Channel::Resource>, YtApiError> {
    let mut result = vec![];
    let mut page_token = None;
    loop {
        let current_page = get_channels(ids, parts, page_token, key).await?;
        page_token = current_page.next_page_token;
        result.extend(current_page.value.into_iter());
        if page_token.is_none() {
            break;
        }
    }
    Ok(result)
}

pub async fn get_channels(
    ids: &[&str],
    parts: &GetChannelParts,
    page_token: Option<String>,
    key: &str,
) -> Result<PagedResponse<Channel::Resource>, YtApiError> {
    log::info!("Getting {} channels info, 1 quota used", ids.len());
    log::debug!("Channel IDs: {:?}", ids);
    if ids.is_empty() {
        return Err(YtApiError::InvalidParameter);
    }
    let mut url = format!(
        "https://www.googleapis.com/youtube/v3/channels?key={}&id={}&maxResults=50",
        key,
        ids.join(","),
    );
    if !parts.is_none() {
        url += "&part=";
        url += &parts.build();
    }
    if let Some(token) = page_token {
        url += "&pageToken=";
        url += &token;
    }
    reqwest::get(url)
        .await
        .map_err(|e| YtApiError::RequestFailed(e.status()))?
        .error_for_status()
        .map_err(|e| YtApiError::RequestFailed(e.status()))?
        .json::<MultipleItemsResponse<Channel::Resource>>()
        .await
        .map_err(|e| YtApiError::DeserializeFailed(format!("{}", e.without_url())))
        .map(|resp| PagedResponse {
            next_page_token: resp.nextPageToken,
            prev_page_token: resp.prevPageToken,
            value: resp.items,
        })
}

#[derive(Default)]
pub struct GetPlaylistItemParts {
    pub _content_details: bool,
    pub _id: bool,
    pub _snippet: bool,
    pub _status: bool,
}

impl GetPlaylistItemParts {
    pub fn build(&self) -> String {
        let mut parts = vec![];
        if self._content_details {
            parts.push("contentDetails".to_string());
        }
        if self._id {
            parts.push("id".to_string());
        }
        if self._snippet {
            parts.push("snippet".to_string());
        }
        if self._status {
            parts.push("status".to_string());
        }
        parts.join(",")
    }

    pub fn content_details(mut self) -> Self {
        self._content_details = true;
        self
    }
    pub fn id(mut self) -> Self {
        self._id = true;
        self
    }
    pub fn snippet(mut self) -> Self {
        self._snippet = true;
        self
    }
    pub fn status(mut self) -> Self {
        self._status = true;
        self
    }
    fn is_none(&self) -> bool {
        !self._content_details && !self._id && !self._snippet && !self._status
    }
}

pub async fn get_all_playlist_items(
    playlist_item_id: &str,
    parts: &GetPlaylistItemParts,
    key: &str,
) -> Result<Vec<PlayListItem::Resource>, YtApiError> {
    let mut result = vec![];
    let mut page_token = None;
    loop {
        let current_page = get_playlist_items(playlist_item_id, parts, page_token, key).await?;
        page_token = current_page.next_page_token;
        result.extend(current_page.value.into_iter());
        if page_token.is_none() {
            break;
        }
    }
    Ok(result)
}

pub async fn get_playlist_items(
    playlist_item_id: &str,
    parts: &GetPlaylistItemParts,
    page_token: Option<String>,
    api_key: &str,
) -> Result<PagedResponse<PlayListItem::Resource>, YtApiError> {
    log::info!("Getting playlist item, 1 quota used");
    log::debug!("Playlist ID: {}", playlist_item_id);
    let mut url = format!(
        "https://www.googleapis.com/youtube/v3/playlistItems?key={}&playlistId={}&maxResults=50",
        api_key, playlist_item_id,
    );
    if !parts.is_none() {
        url += "&part=";
        url += &parts.build();
    }
    if let Some(token) = page_token {
        url += "&pageToken=";
        url += &token;
    }
    reqwest::get(url)
        .await
        .map_err(|e| YtApiError::RequestFailed(e.status()))?
        .error_for_status()
        .map_err(|e| YtApiError::RequestFailed(e.status()))?
        .json::<MultipleItemsResponse<PlayListItem::Resource>>()
        .await
        .map_err(|e| YtApiError::DeserializeFailed(format!("{}", e.without_url())))
        .map(|resp| PagedResponse {
            next_page_token: resp.nextPageToken,
            prev_page_token: resp.prevPageToken,
            value: resp.items,
        })
}

#[derive(Default)]
pub struct GetVideoParts {
    _content_details: bool,
    _file_details: bool,
    _id: bool,
    _live_streaming_details: bool,
    _localizations: bool,
    _player: bool,
    _processing_details: bool,
    _recording_details: bool,
    _snippet: bool,
    _statistics: bool,
    _status: bool,
    _suggestions: bool,
    _topic_details: bool,
}

impl GetVideoParts {
    pub fn build(&self) -> String {
        let mut parts = vec![];
        if self._content_details {
            parts.push("contentDetails".to_string());
        }
        if self._file_details {
            parts.push("fileDetails".to_string());
        }
        if self._id {
            parts.push("id".to_string());
        }
        if self._live_streaming_details {
            parts.push("liveStreamingDetails".to_string());
        }
        if self._localizations {
            parts.push("localizations".to_string());
        }
        if self._player {
            parts.push("player".to_string());
        }
        if self._processing_details {
            parts.push("processingDetails".to_string());
        }
        if self._recording_details {
            parts.push("recordingDetails".to_string());
        }
        if self._snippet {
            parts.push("snippet".to_string());
        }
        if self._statistics {
            parts.push("statistics".to_string());
        }
        if self._status {
            parts.push("status".to_string());
        }
        if self._suggestions {
            parts.push("suggestions".to_string());
        }
        if self._topic_details {
            parts.push("topicDetails".to_string());
        }
        parts.join(",")
    }

    pub fn content_details(mut self) -> Self {
        self._content_details = true;
        self
    }
    pub fn file_details(mut self) -> Self {
        self._file_details = true;
        self
    }
    pub fn id(mut self) -> Self {
        self._id = true;
        self
    }
    pub fn live_streaming_details(mut self) -> Self {
        self._live_streaming_details = true;
        self
    }
    pub fn localizations(mut self) -> Self {
        self._localizations = true;
        self
    }
    pub fn player(mut self) -> Self {
        self._player = true;
        self
    }
    pub fn processing_details(mut self) -> Self {
        self._processing_details = true;
        self
    }
    pub fn recording_details(mut self) -> Self {
        self._recording_details = true;
        self
    }
    pub fn snippet(mut self) -> Self {
        self._snippet = true;
        self
    }
    pub fn statistics(mut self) -> Self {
        self._statistics = true;
        self
    }
    pub fn status(mut self) -> Self {
        self._status = true;
        self
    }
    pub fn suggestions(mut self) -> Self {
        self._suggestions = true;
        self
    }
    pub fn topic_details(mut self) -> Self {
        self._topic_details = true;
        self
    }
    fn is_none(&self) -> bool {
        !self._content_details
            && !self._file_details
            && !self._id
            && !self._live_streaming_details
            && !self._localizations
            && !self._player
            && !self._processing_details
            && !self._recording_details
            && !self._snippet
            && !self._statistics
            && !self._status
            && !self._suggestions
            && !self._topic_details
    }
}

pub async fn get_all_video_items(
    video_ids: &[String],
    parts: &GetVideoParts,
    key: &str,
) -> Result<Vec<Video::Resource>, YtApiError> {
    let mut result = vec![];
    let mut page_token = None;
    loop {
        let current_page = get_video(video_ids, parts, page_token, key).await?;
        page_token = current_page.next_page_token;
        result.extend(current_page.value.into_iter());
        if page_token.is_none() {
            break;
        }
    }
    Ok(result)
}

pub async fn get_video(
    video_ids: &[String],
    parts: &GetVideoParts,
    page_token: Option<String>,
    api_key: &str,
) -> Result<PagedResponse<Video::Resource>, YtApiError> {
    if video_ids.is_empty() {
        return Err(YtApiError::InvalidParameter);
    }
    log::info!("Getting {} videos info, 1 quota used", video_ids.len());
    log::debug!("Video IDs: {:?}", video_ids);
    let mut url = format!(
        "https://www.googleapis.com/youtube/v3/videos?key={}&id={}&maxResults=50",
        api_key,
        video_ids.join(","),
    );
    if !parts.is_none() {
        url += "&part=";
        url += &parts.build();
    }
    if let Some(token) = page_token {
        url += "&pageToken=";
        url += &token;
    }
    reqwest::get(url)
        .await
        .map_err(|e| YtApiError::RequestFailed(e.status()))?
        .error_for_status()
        .map_err(|e| YtApiError::RequestFailed(e.status()))?
        .json::<MultipleItemsResponse<Video::Resource>>()
        .await
        .map_err(|e| YtApiError::DeserializeFailed(format!("{}", e.without_url())))
        .map(|resp| PagedResponse {
            next_page_token: resp.nextPageToken,
            prev_page_token: resp.prevPageToken,
            value: resp.items,
        })
}

static VIDEO_ID_PATTERN: Lazy<Regex> =
    Lazy::new(|| regex::Regex::new("<yt:videoId>(.+?)</yt:videoId>").unwrap());

pub async fn get_video_list_through_rss(channel_id: &str) -> Result<Vec<String>, YtApiError> {
    let feed_body = reqwest::get(format!(
        "https://www.youtube.com/feeds/videos.xml?channel_id={}",
        channel_id
    ))
    .await
    .map_err(|e| YtApiError::RequestFailed(e.status()))?
    .error_for_status()
    .map_err(|e| YtApiError::RequestFailed(e.status()))?
    .text()
    .await
    .map_err(|e| YtApiError::DeserializeFailed(format!("{}", e.without_url())))?;
    Ok(VIDEO_ID_PATTERN
        .captures_iter(&feed_body)
        .map(|cap| cap.get(1).unwrap().as_str().to_string())
        .collect())
}

#[cfg(test)]
mod test {
    use super::*;
    const CUSTOM_URL: &'static str = "GawrGura";
    const CHANNEL_ID_TEST_URL: &'static str = "https://www.youtube.com/@GawrGura";
    const CHANNEL_ID_TEST_ID: &'static str = "UCoSrY_IQQVpmIRZ9Xf-y93g";
    #[test]
    fn get_channel_id_test() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        assert_eq!(
            runtime.block_on(get_channel_id_by_url(CHANNEL_ID_TEST_URL)),
            Ok(CHANNEL_ID_TEST_ID.to_string())
        )
    }

    #[test]
    fn test_try_youtube_id() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        assert_eq!(
            runtime.block_on(try_youtube_id(CHANNEL_ID_TEST_URL)),
            CHANNEL_ID_TEST_ID.to_string()
        );
        assert_eq!(
            runtime.block_on(try_youtube_id(CUSTOM_URL)),
            CHANNEL_ID_TEST_ID.to_string()
        );
        assert_eq!(
            runtime.block_on(try_youtube_id(&format!("@{}", CUSTOM_URL))),
            CHANNEL_ID_TEST_ID.to_string()
        );
        assert_eq!(
            runtime.block_on(try_youtube_id(CHANNEL_ID_TEST_ID)),
            CHANNEL_ID_TEST_ID.to_string()
        );
        assert_eq!(
            runtime.block_on(try_youtube_id(&format!(
                "https://www.youtube.com/channel/{}",
                CHANNEL_ID_TEST_ID
            ))),
            CHANNEL_ID_TEST_ID.to_string()
        );
    }

    #[test]
    fn test_all_channel_id_patterns() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        runtime.block_on(async {
            let channel_page_src = reqwest::get(CHANNEL_ID_TEST_URL)
                .await
                .expect("Get channel page source failed")
                .error_for_status()
                .expect("Get channel page source failed")
                .text()
                .await
                .expect("Decode response body failed");

            for (pattern, grp) in CHANNEL_ID_PATTERNS.iter() {
                assert_eq!(
                    pattern
                        .captures(&channel_page_src)
                        .expect(&format!("Capture with pattern {} failed", pattern.as_str()))
                        .get(*grp)
                        .expect(&format!(
                            "Get group {} from capture result of pattern {} failed",
                            *grp,
                            pattern.as_str()
                        ))
                        .as_str(),
                    CHANNEL_ID_TEST_ID
                )
            }
        });
    }
}
