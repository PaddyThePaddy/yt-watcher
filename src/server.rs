use std::{collections::HashMap, net::SocketAddr, str::FromStr, sync::Arc, time::Duration};

use crate::yt_api::{structs::*, *};
use chrono::{DateTime, Utc};
use icalendar::{Component, EventLike};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use warp::Filter;

#[derive(Debug, Serialize, Deserialize)]
pub struct ChannelIdResponse {
    channel_id: Option<String>,
    error: Option<String>,
}

const SAVE_FILE: &'static str = "save.json";
pub async fn server_start(
    addr: impl Into<SocketAddr> + Clone,
    api_key: &str,
    refresh_interval: u64,
) {
    let root_node = warp::path::end().map(|| "Root node");
    let server_data = Arc::new(RwLock::new(ServerData::new(api_key)));

    {
        match tokio::fs::read_to_string(SAVE_FILE).await {
            Ok(s) => {
                if let Err(e) = server_data.write().await.load_save_string(&s) {
                    log::error!("Load save file failed: {e}")
                }
            }
            Err(e) => log::error!("Load save file failed: {e}"),
        }
        server_data.write().await.check_upcoming_event().await;
    }

    let get_channel_id = warp::get()
        .and(warp::path("channel"))
        .and(warp::query::<HashMap<String, String>>())
        .then(|query: HashMap<String, String>| async move {
            let response_data = match query.get("q") {
                None => ChannelIdResponse {
                    channel_id: None,
                    error: Some(format!(
                        "Please provide channel name with \"q\" get parameter"
                    )),
                },
                Some(channel_name) => {
                    match get_channel_id_by_url(&format!(
                        "https://www.youtube.com/@{}",
                        channel_name.trim_start_matches("@")
                    ))
                    .await
                    {
                        Ok(id) => ChannelIdResponse {
                            channel_id: Some(id),
                            error: None,
                        },
                        Err(e) => ChannelIdResponse {
                            channel_id: None,
                            error: Some(format!("Failed to get channel id: {:?}", e)),
                        },
                    }
                }
            };
            serde_json::to_string(&response_data).unwrap()
        });

    let server_data_clone = server_data.clone();
    let get_data_endpoint = warp::get()
        .and(warp::path("data"))
        .and(warp::query::<HashMap<String, String>>())
        .then(move |query: HashMap<String, String>| {
            let server_data_clone2 = server_data_clone.clone();
            async move {
                let mut response: Vec<UpcomingEvent> = vec![];
                if let Some(query_str) = query.get("channels") {
                    let mut ids: Vec<String> = vec![];
                    for id in query_str.split(",") {
                        ids.push(try_youtube_id(id).await);
                    }
                    let new_channel_ids = {
                        server_data_clone2
                            .read()
                            .await
                            .filter_new_yt_channel_id(&ids)
                    };
                    if !new_channel_ids.is_empty() {
                        if let Err(e) = server_data_clone2
                            .write()
                            .await
                            .track_new_channel(&new_channel_ids)
                            .await
                        {
                            log::error!("Track new channel failed: {:?}", e);
                        };
                        match server_data_clone2.read().await.to_save_string() {
                            Ok(save_str) => {
                                if let Err(e) = tokio::fs::write(SAVE_FILE, save_str).await {
                                    log::error!("Write to save file failed: {}", e);
                                }
                            }
                            Err(e) => log::error!("Encode to save string failed: {}", e),
                        }
                    }
                    let events = { server_data_clone2.read().await.events.clone() };
                    response.extend(events.into_iter().filter(|e| match &e.source {
                        EventSource::YoutubeChannel(id) => ids.contains(id),
                    }));
                }
                serde_json::to_string(&response).unwrap()
            }
        });

    let server_data_clone = server_data.clone();
    let get_calendar_endpoint = warp::get()
        .and(warp::path("cal"))
        .and(warp::query::<HashMap<String, String>>())
        .then(move |query: HashMap<String, String>| {
            let server_data_clone2 = server_data_clone.clone();
            async move {
                let mut cal = icalendar::Calendar::new();
                cal.name("VT calendar");
                if let Some(query_str) = query.get("channels") {
                    let mut ids: Vec<String> = vec![];
                    for id in query_str.split(",") {
                        ids.push(try_youtube_id(id).await);
                    }
                    let new_channel_ids = {
                        server_data_clone2
                            .read()
                            .await
                            .filter_new_yt_channel_id(&ids)
                    };
                    if !new_channel_ids.is_empty() {
                        if let Err(e) = server_data_clone2
                            .write()
                            .await
                            .track_new_channel(&new_channel_ids)
                            .await
                        {
                            log::error!("Track new channel failed: {:?}", e);
                        };
                        match server_data_clone2.read().await.to_save_string() {
                            Ok(save_str) => {
                                if let Err(e) = tokio::fs::write(SAVE_FILE, save_str).await {
                                    log::error!("Write to save file failed: {}", e);
                                }
                            }
                            Err(e) => log::error!("Encode to save string failed: {}", e),
                        }
                    }
                    let events = { server_data_clone2.read().await.events.clone() };
                    cal.extend(
                        events
                            .into_iter()
                            .filter(|e: &UpcomingEvent| match &e.source {
                                EventSource::YoutubeChannel(id) => ids.contains(id),
                            })
                            .map(|e| e.to_ical_event()),
                    );
                }
                cal.done().to_string()
            }
        });

    let server_data_clone = server_data.clone();
    let _handle = tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60 * refresh_interval)).await;
            log::info!("Updating upcoming event");
            let mut data = server_data_clone.write().await;
            data.check_upcoming_event().await;
            match data.to_save_string() {
                Ok(save_str) => {
                    if let Err(e) = tokio::fs::write(SAVE_FILE, save_str).await {
                        log::error!("Write to save file failed: {}", e);
                    }
                }
                Err(e) => log::error!("Encode to save string failed: {}", e),
            }
        }
    });
    let routes = warp::get().and(
        root_node
            .or(get_channel_id)
            .or(get_data_endpoint)
            .or(get_calendar_endpoint),
    );
    warp::serve(routes).run(addr.into()).await;
}

#[derive(Debug, Serialize, Deserialize)]
struct YtChannelSave {
    custom_url: String,
    id: String,
    title: String,
    upload_playlist: String,
    first_video_after_all_stream: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum EventSource {
    YoutubeChannel(String),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpcomingEvent {
    schedule_date: DateTime<Utc>,
    thumbnail_url: Option<String>,
    title: String,
    description: String,
    target_url: String,
    on_going: bool,
    source: EventSource,
}

impl UpcomingEvent {
    fn to_ical_event(&self) -> icalendar::Event {
        let mut builder = icalendar::Event::new();
        if self.on_going {
            builder.summary(&format!("🔴{}", self.title));
        } else {
            builder.summary(&self.title);
        }
        builder.description(&format!("{}\n\n{}", self.target_url, self.description));
        builder.starts(self.schedule_date);
        builder.ends(self.schedule_date + chrono::Duration::hours(1));
        builder.url(&self.target_url);
        builder.done()
    }
}

#[derive(Debug)]
pub enum ConvertToUpcomingEventError {
    AlreadyDone(String),
    MissingInformation(String),
    DecodeError(String),
    Unknown(String),
}

impl TryFrom<&Video::Resource> for UpcomingEvent {
    type Error = ConvertToUpcomingEventError;
    fn try_from(value: &Video::Resource) -> Result<Self, Self::Error> {
        let start_time: DateTime<Utc> = if let Some(live_info) = &value.liveStreamingDetails {
            if let Some(actual_start_time) = &live_info.actualStartTime {
                chrono::DateTime::from_str(&actual_start_time)
                    .map_err(|e| ConvertToUpcomingEventError::DecodeError(format!("{}", e)))?
            } else if let Some(scheduled_start_time) = &live_info.scheduledStartTime {
                chrono::DateTime::from_str(&scheduled_start_time)
                    .map_err(|e| ConvertToUpcomingEventError::DecodeError(format!("{}", e)))?
            } else {
                return Err(ConvertToUpcomingEventError::MissingInformation(
                    "start time".to_string(),
                ));
            }
        } else {
            return Err(ConvertToUpcomingEventError::MissingInformation(
                "liveStreamingDetails".to_string(),
            ));
        };

        match &value.snippet {
            None => {
                return Err(ConvertToUpcomingEventError::MissingInformation(
                    "snippet".to_string(),
                ))
            }
            Some(snippet) => {
                let on_going;
                match snippet.liveBroadcastContent.as_str() {
                    "none" => {
                        return Err(ConvertToUpcomingEventError::AlreadyDone(value.id.clone()))
                    }
                    "live" => on_going = true,
                    "upcoming" => on_going = false,
                    _ => return Err(ConvertToUpcomingEventError::Unknown(value.id.clone())),
                }
                let thumbnail_url = if let Some(t) = snippet.thumbnails.get("default") {
                    t
                } else {
                    return Err(ConvertToUpcomingEventError::MissingInformation(
                        "Default thumbnail".to_string(),
                    ));
                };
                return Ok(UpcomingEvent {
                    schedule_date: start_time,
                    title: snippet.title.clone(),
                    description: snippet.description.clone(),
                    target_url: format!("https://www.youtube.com/watch?v={}", value.id),
                    on_going,
                    thumbnail_url: Some(thumbnail_url.url.clone()),
                    source: EventSource::YoutubeChannel(snippet.channelId.clone()),
                });
            }
        }
    }
}

pub struct ServerData {
    yt_channels: HashMap<String, YtChannelSave>,
    events: Vec<UpcomingEvent>,
    api_key: String,
}

impl ServerData {
    fn new(api_key: &str) -> Self {
        Self {
            yt_channels: HashMap::new(),
            events: vec![],
            api_key: api_key.to_string(),
        }
    }

    fn to_save_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self.yt_channels)
    }

    fn load_save_string(&mut self, save: &str) -> Result<(), serde_json::Error> {
        let channels: HashMap<String, YtChannelSave> = serde_json::from_str(save)?;
        channels.into_iter().for_each(|(channel_id, channel)| {
            self.yt_channels.insert(channel_id, channel);
        });

        Ok(())
    }

    pub async fn check_upcoming_event(&mut self) {
        let mut events = vec![];
        let mut unchecked_video_ids = vec![];
        let mut first_video_after_all_stream_map: HashMap<String, String> = HashMap::new();
        for c in self.yt_channels.values() {
            //match get_playlist_items(
            //    &c.upload_playlist,
            //    &GetPlaylistItemParts::default().content_details(),
            //    None,
            //    &self.api_key,
            //)
            //.await
            //{
            //    Err(e) => log::error!("Fail to get playlist item of channel {}: {:?}", c.id, e),
            //    Ok(resp) => {
            //        for v in resp.value {
            //            if v.id == c.first_video_after_all_stream {
            //                break;
            //            }
            //            unchecked_video_ids.push(v.contentDetails.expect("The response of get playlist request doesn't has the contentDetail field. Does youtube api updated?").videoId);
            //        }
            //    }
            //}
            match get_video_list_through_rss(&c.id).await {
                Err(e) => log::error!(
                    "Fail to get video list through rss of channel {}: {:?}",
                    c.id,
                    e
                ),
                Ok(resp) => {
                    for v in resp {
                        if v == c.first_video_after_all_stream {
                            break;
                        }
                        unchecked_video_ids.push(v);
                    }
                }
            }
        }
        match get_all_video_items(
            unchecked_video_ids
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<&str>>()
                .as_slice(),
            &GetVideoParts::default().snippet().live_streaming_details(),
            &self.api_key,
        )
        .await
        {
            Err(e) => log::error!("Fail to get video items: {:?}", e),
            Ok(resp) => {
                for v in resp {
                    if let Some(snippet) = &v.snippet {
                        if snippet.liveBroadcastContent == "none" {
                            if !first_video_after_all_stream_map.contains_key(&snippet.channelId) {
                                first_video_after_all_stream_map
                                    .insert(snippet.channelId.clone(), v.id.clone());
                            }
                        } else {
                            first_video_after_all_stream_map.remove(&snippet.channelId);
                        }
                    }
                    match UpcomingEvent::try_from(&v) {
                        Ok(e) => events.push(e),
                        Err(e) => match e {
                            ConvertToUpcomingEventError::AlreadyDone(_) => {}
                            ConvertToUpcomingEventError::MissingInformation(msg) => {
                                if msg != "liveStreamingDetails" {
                                    log::error!("Convert video {} to event failed: MissingInformation(\"{}\")", v.id, msg);
                                }
                            }
                            _ => {
                                log::error!("Convert video {} to event failed: {:?}", v.id, e)
                            }
                        },
                    }
                }
            }
        }

        first_video_after_all_stream_map
            .into_iter()
            .for_each(|(channel_id, video_id)| {
                if let Some(c) = self.yt_channels.get_mut(&channel_id) {
                    c.first_video_after_all_stream = video_id;
                }
            });
        self.events = events;
    }

    pub async fn track_new_channel(&mut self, ids: &[&str]) -> Result<(), YtApiError> {
        let channels = get_all_channels(
            ids,
            &GetChannelParts::default().snippet().content_details(),
            &self.api_key,
        )
        .await?;
        for c in channels {
            if c.contentDetails.is_none() {
                log::error!("The response of get channel {} request doesn't has the contentDetail field. Does youtube api updated?", c.id);
            } else if c.snippet.is_none() {
                log::error!("The response of get channel {} request doesn't has the snippet field. Does youtube api updated?", c.id);
            } else {
                let snippet = c.snippet.expect("The response of get channel request doesn't has the snippet field. Does youtube api updated?");
                let content_detail = c.contentDetails.expect("The response of get channel request doesn't has the contentDetail field. Does youtube api updated?");
                log::info!(
                    "Tracking new channel: {} ({}, {})",
                    c.id,
                    snippet.title,
                    snippet.customUrl
                );
                //let list_item = get_playlist_items(
                //    &content_detail.relatedPlaylists.uploads,
                //    &GetPlaylistItemParts::default().content_details(),
                //    None,
                //    &self.api_key,
                //)
                //.await?;
                //let video_ids:Vec<String> = list_item.value.iter().map(|item| item.contentDetails.as_ref().expect("The response of get playlist request doesn't has the contentDetail field. Does youtube api updated?").videoId.clone()).collect();
                let video_ids = get_video_list_through_rss(&c.id).await?;
                let videos = get_all_video_items(
                    video_ids
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<&str>>()
                        .as_slice(),
                    &GetVideoParts::default().snippet().live_streaming_details(),
                    &self.api_key,
                )
                .await?;
                let mut first_video_after_all_stream = None;
                for v in videos {
                    match UpcomingEvent::try_from(&v) {
                        Ok(e) => self.events.push(e),
                        Err(e) => match e {
                            ConvertToUpcomingEventError::AlreadyDone(_) => {}
                            ConvertToUpcomingEventError::MissingInformation(msg) => {
                                if msg != "liveStreamingDetails" {
                                    log::error!("Convert video {} to event failed: MissingInformation(\"{}\")", v.id, msg);
                                }
                            }
                            _ => {
                                log::error!("Convert video {} to event failed: {:?}", v.id, e)
                            }
                        },
                    }
                    if let Some(snippet) = v.snippet {
                        if snippet.liveBroadcastContent == "none" {
                            if first_video_after_all_stream.is_none() {
                                first_video_after_all_stream = Some(v.id);
                            }
                        } else {
                            first_video_after_all_stream = None;
                        }
                    }
                }
                self.yt_channels.insert(
                    c.id.clone(),
                    YtChannelSave {
                        custom_url: snippet.customUrl,
                        id: c.id.clone(),
                        title: snippet.title,
                        upload_playlist: content_detail.relatedPlaylists.uploads.clone(),
                        first_video_after_all_stream: first_video_after_all_stream
                            .unwrap_or("".to_string()),
                    },
                );
            }
        }
        Ok(())
    }

    fn filter_new_yt_channel_id<'a>(&self, channel_ids: &'a [String]) -> Vec<&'a str> {
        channel_ids
            .iter()
            .filter(|c| !self.yt_channels.contains_key(*c))
            .map(|s| s.as_str())
            .collect()
    }
}
