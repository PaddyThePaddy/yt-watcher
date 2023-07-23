use std::{
    collections::{HashMap, HashSet},
    net::SocketAddr,
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use crate::{
    yt_api::{structs::*, *},
    Compression,
};
use chrono::{DateTime, Utc};
use icalendar::{Alarm, Component, EventLike};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use warp::Filter;

#[derive(Debug, Serialize, Deserialize)]
pub struct ChannelInfoData {
    id: String,
    title: String,
    custom_url: String,
    thumbnail: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum ChannelInfoResponse {
    data(ChannelInfoData),
    error(String),
}

const CHANNELS_SAVE_FILE: &'static str = "channels.json";
const VIDEOS_SAVE_FILE: &'static str = "videos.txt";
pub async fn server_start(config: &crate::Config) {
    let http_socket = match SocketAddr::from_str(&config.socket) {
        Ok(s) => s,
        Err(e) => panic!("Invalid socket: {}", e),
    };
    let tls_info = config.tls.clone().map(|tls| {
        (
            match SocketAddr::from_str(&tls.socket) {
                Ok(s) => s,
                Err(e) => panic!("Invalid socket: {}", e),
            },
            tls.cert,
            tls.key,
        )
    });
    let server_data = Arc::new(RwLock::new(ServerData::new(
        &config.api_key,
        CHANNELS_SAVE_FILE.to_string(),
        VIDEOS_SAVE_FILE.to_string(),
        config.channel_expire_min,
    )));

    {
        server_data.write().await.restore().await;
        server_data.write().await.check_upcoming_event().await;
    }

    let server_data_clone = server_data.clone();
    let get_channel_info = warp::get()
        .and(warp::path("channel"))
        .and(warp::query::<HashMap<String, String>>())
        .then(move |query: HashMap<String, String>| {
            let server_data_clone2: Arc<RwLock<ServerData>> = server_data_clone.clone();
            async move {
                let response_data = match query.get("q") {
                    None => ChannelInfoResponse::error(format!(
                        "Please provide channel name with \"q\" get parameter"
                    )),
                    Some(url) => match get_channel_id_by_url(&format!(
                        "https://www.youtube.com/channel/{}",
                        try_youtube_id(url).await
                    ))
                    .await
                    {
                        Ok(id) => {
                            if {
                                !server_data_clone2
                                    .read()
                                    .await
                                    .yt_channels
                                    .contains_key(&id)
                            } {
                                if let Err(e) = {
                                    server_data_clone2
                                        .write()
                                        .await
                                        .track_new_channels(&[&id])
                                        .await
                                } {
                                    log::error!("Track new channel failed: {:?}", e);
                                    return serde_json::to_string(&ChannelInfoResponse::error(
                                        format!("Track new channel failed: {:?}", e),
                                    ))
                                    .unwrap();
                                }
                            }
                            let mut server_data = server_data_clone2.write().await;
                            server_data.touch_channel(&id);
                            let channel_save = server_data.yt_channels.get(&id).unwrap();
                            ChannelInfoResponse::data(ChannelInfoData {
                                id,
                                title: channel_save.title.clone(),
                                custom_url: channel_save.custom_url.clone(),
                                thumbnail: channel_save.thumbnail.clone(),
                            })
                        }
                        Err(e) => {
                            ChannelInfoResponse::error(format!("Failed to get channel id: {:?}", e))
                        }
                    },
                };
                serde_json::to_string(&response_data).unwrap()
            }
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
                            .track_new_channels(&new_channel_ids)
                            .await
                        {
                            log::error!("Track new channel failed: {:?}", e);
                        };
                    }

                    let events = {
                        let mut server_data = server_data_clone2.write().await;
                        for id in ids.iter() {
                            server_data.touch_channel(id);
                        }
                        server_data.events.clone()
                    };
                    response.extend(events.into_iter().filter(|e| match &e.source {
                        EventSource::YoutubeChannel(c) => ids.contains(&c.id),
                    }));
                }
                response.sort();
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
                let alarm_enabled = match query.get("alram") {
                    Some(v) => v.to_lowercase() == "true" || v.to_lowercase() == "yes",
                    None => false,
                };
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
                            .track_new_channels(&new_channel_ids)
                            .await
                        {
                            log::error!("Track new channel failed: {:?}", e);
                        };
                    }

                    let events = {
                        let mut server_data = server_data_clone2.write().await;
                        for id in ids.iter() {
                            server_data.touch_channel(id);
                        }
                        server_data.events.clone()
                    };
                    cal.extend(
                        events
                            .into_iter()
                            .filter(|e: &UpcomingEvent| match &e.source {
                                EventSource::YoutubeChannel(c) => ids.contains(&c.id),
                            })
                            .map(|e| e.to_ical_event(alarm_enabled)),
                    );
                }
                cal.done().to_string()
            }
        });

    let server_data_clone = server_data.clone();
    let video_refresh_interval = config.video_refresh_interval;
    let _handle = tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60 * video_refresh_interval)).await;
            log::info!("Updating upcoming event");
            let mut data = server_data_clone.write().await;
            data.check_upcoming_event().await;
        }
    });
    let server_data_clone = server_data.clone();
    let channel_refresh_interval = config.channel_refresh_interval;
    let _handle = tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60 * channel_refresh_interval)).await;
            log::info!("Updating channel info");
            let mut data = server_data_clone.write().await;
            data.update_channel_info().await;
        }
    });
    let routes = warp::get().and(
        warp::fs::dir("www")
            .or(get_channel_info)
            .or(get_data_endpoint)
            .or(get_calendar_endpoint),
    );
    if config.compression == Compression::none {
        if let Some((https_socket, cert, key)) = tls_info {
            futures::join!(
                warp::serve(routes.clone()).run(http_socket),
                warp::serve(routes)
                    .tls()
                    .cert_path(cert)
                    .key_path(key)
                    .run(https_socket),
            );
        } else {
            warp::serve(routes).run(http_socket).await;
        }
    } else {
        let routes = match config.compression {
            Compression::none => unimplemented!(),
            Compression::gzip => routes.with(warp::filters::compression::deflate()),
            Compression::dflate => routes.with(warp::filters::compression::deflate()),
            //Compression::brotli => routes.with(warp::filters::compression::brotli()),
        };
        if let Some((https_socket, cert, key)) = tls_info {
            futures::join!(
                warp::serve(routes.clone()).run(http_socket),
                warp::serve(routes)
                    .tls()
                    .cert_path(cert)
                    .key_path(key)
                    .run(https_socket),
            );
        } else {
            warp::serve(routes).run(http_socket).await;
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct YtChannelSave {
    custom_url: String,
    id: String,
    title: String,
    thumbnail: String,
    upload_playlist: String,
    last_time_used: DateTime<Utc>,
    first_video_after_all_stream: String,
}

#[derive(Debug, Serialize, Deserialize, Default)]
struct YtVideosSave {
    ids: HashSet<String>,
}

impl YtVideosSave {
    fn dump(&self, target: &mut Vec<String>) {
        for id in self.ids.iter() {
            if !target.contains(id) {
                target.push(id.clone());
            }
        }
    }

    fn set<'a>(&mut self, new_value: HashSet<String>) {
        self.ids = new_value;
    }

    fn push_checked(&mut self, new_value: String) {
        self.ids.insert(new_value);
    }

    fn remove(&mut self, value: &String) {
        self.ids.remove(value);
    }

    fn extend_from_str(&mut self, value: &str) {
        let mut new_value = value
            .split("\n")
            .filter(|s| !s.trim().is_empty())
            .map(|s| s.to_string())
            .collect();
        self.dump(&mut new_value);
        self.set(HashSet::from_iter(new_value));
    }
}

impl ToString for YtVideosSave {
    fn to_string(&self) -> String {
        let mut s = String::new();
        let mut iter = self.ids.iter();
        if let Some(v) = iter.next() {
            s += v;
        }
        for v in iter {
            s += "\n";
            s += v;
        }
        s
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ChannelBrief {
    id: String,
    thumbnail_url: String,
    title: String,
    custom_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum EventSource {
    YoutubeChannel(ChannelBrief),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct UpcomingEvent {
    start_date_time: DateTime<Utc>,
    start_timestamp_millis: i64,
    thumbnail_url: Option<String>,
    title: String,
    description: String,
    target_url: String,
    ongoing: bool,
    source: EventSource,
}

impl UpcomingEvent {
    fn to_ical_event(&self, alarm_enabled: bool) -> icalendar::Event {
        let mut builder = icalendar::Event::new();
        builder.starts(self.start_date_time);
        if self.ongoing {
            builder.summary(&format!("🔴{}", self.title));
            builder.ends(Utc::now() + chrono::Duration::hours(1));
        } else {
            builder.summary(&self.title);
            builder.ends(self.start_date_time + chrono::Duration::hours(1));
        }
        let mut description = format!("{}\n\n", self.target_url);
        match &self.source {
            EventSource::YoutubeChannel(c) => {
                description += &format!("{}\n{}\n\n", c.title, c.custom_url);
            }
        }
        description += &self.description;

        builder.url(&self.target_url);
        if alarm_enabled {
            builder.alarm(Alarm::display(&self.title, -chrono::Duration::minutes(5)));
        }
        builder.done()
    }
}

impl PartialOrd for UpcomingEvent {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.start_date_time.partial_cmp(&other.start_date_time)
    }
}

impl Ord for UpcomingEvent {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.start_date_time.cmp(&other.start_date_time)
    }
}
#[derive(Debug)]
pub enum ConvertToUpcomingEventError {
    AlreadyDone(String),
    MissingInformation(String),
    DecodeError(String),
    EventSourceNotFound(String),
    Unknown(String),
}

impl TryFrom<(&Video::Resource, &ServerData)> for UpcomingEvent {
    type Error = ConvertToUpcomingEventError;
    fn try_from(value: (&Video::Resource, &ServerData)) -> Result<Self, Self::Error> {
        let start_time: DateTime<Utc> = if let Some(live_info) = &value.0.liveStreamingDetails {
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

        match &value.0.snippet {
            None => {
                return Err(ConvertToUpcomingEventError::MissingInformation(
                    "snippet".to_string(),
                ))
            }
            Some(snippet) => {
                let on_going;
                match snippet.liveBroadcastContent.as_str() {
                    "none" => {
                        return Err(ConvertToUpcomingEventError::AlreadyDone(value.0.id.clone()))
                    }
                    "live" => on_going = true,
                    "upcoming" => on_going = false,
                    _ => return Err(ConvertToUpcomingEventError::Unknown(value.0.id.clone())),
                }
                let thumbnail_url = if let Some(t) = snippet.thumbnails.get("medium") {
                    t
                } else {
                    return Err(ConvertToUpcomingEventError::MissingInformation(
                        "Default thumbnail".to_string(),
                    ));
                };

                return Ok(UpcomingEvent {
                    start_date_time: start_time,
                    start_timestamp_millis: start_time.timestamp_millis(),
                    title: snippet.title.clone(),
                    description: snippet.description.clone(),
                    target_url: format!("https://www.youtube.com/watch?v={}", value.0.id),
                    ongoing: on_going,
                    thumbnail_url: Some(thumbnail_url.url.clone()),
                    source: EventSource::YoutubeChannel(
                        match value.1.yt_channels.get(&snippet.channelId) {
                            None => {
                                return Err(ConvertToUpcomingEventError::EventSourceNotFound(
                                    snippet.channelId.clone(),
                                ))
                            }
                            Some(c) => ChannelBrief {
                                id: c.id.clone(),
                                thumbnail_url: c.thumbnail.clone(),
                                title: c.title.clone(),
                                custom_url: c.custom_url.clone(),
                            },
                        },
                    ),
                });
            }
        }
    }
}

#[derive(Default)]
pub struct ServerData {
    yt_channels: HashMap<String, YtChannelSave>,
    yt_videos: YtVideosSave,
    events: Vec<UpcomingEvent>,
    api_key: String,
    channel_save_path: String,
    video_save_path: String,
    channel_expire_min: i64,
}

impl ServerData {
    fn new(
        api_key: &str,
        channel_save_path: String,
        video_save_path: String,
        channel_expire_min: i64,
    ) -> Self {
        Self {
            api_key: api_key.to_string(),
            channel_save_path,
            video_save_path,
            channel_expire_min,
            ..Default::default()
        }
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

        self.yt_videos.dump(&mut unchecked_video_ids);

        match get_all_video_items(
            unchecked_video_ids.as_slice(),
            &GetVideoParts::default().snippet().live_streaming_details(),
            &self.api_key,
        )
        .await
        {
            Err(e) => log::error!("Fail to get video items: {:?}", e),
            Ok(resp) => {
                for v in resp {
                    if let Some(snippet) = &v.snippet {
                        if !self.yt_channels.contains_key(&snippet.channelId) {
                            log::info!("Video {} does not belongs to any tracking channel", v.id);
                            self.yt_videos.remove(&v.id);
                            continue;
                        }
                        if snippet.liveBroadcastContent == "none" {
                            if !first_video_after_all_stream_map.contains_key(&snippet.channelId) {
                                first_video_after_all_stream_map
                                    .insert(snippet.channelId.clone(), v.id.clone());
                            }
                        } else {
                            first_video_after_all_stream_map.remove(&snippet.channelId);
                            self.yt_videos.push_checked(v.id.clone());
                        }
                    }
                    match UpcomingEvent::try_from((&v, &*self)) {
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
        self.save().await;
    }

    pub async fn track_new_channels(&mut self, ids: &[&str]) -> Result<(), YtApiError> {
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
                    video_ids.as_slice(),
                    &GetVideoParts::default().snippet().live_streaming_details(),
                    &self.api_key,
                )
                .await?;

                let mut max_thumbnail: (&str, u32) = ("", 0);
                for thumb in snippet.thumbnails.values() {
                    if thumb.width > max_thumbnail.1 && thumb.width <= 240 {
                        max_thumbnail = (thumb.url.as_str(), thumb.width);
                    }
                }
                self.yt_channels.insert(
                    c.id.clone(),
                    YtChannelSave {
                        custom_url: snippet.customUrl,
                        thumbnail: max_thumbnail.0.to_string(),
                        id: c.id.clone(),
                        title: snippet.title,
                        upload_playlist: content_detail.relatedPlaylists.uploads.clone(),
                        first_video_after_all_stream: String::new(),
                        last_time_used: Utc::now(),
                    },
                );

                let mut first_video_after_all_stream = None;
                for v in videos {
                    match UpcomingEvent::try_from((&v, &*self)) {
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
                            self.yt_videos.push_checked(v.id.clone());
                        }
                    }
                }

                if let Some(save) = self.yt_channels.get_mut(&c.id) {
                    save.first_video_after_all_stream =
                        first_video_after_all_stream.unwrap_or("".to_string());
                }
            }
        }
        self.save().await;
        Ok(())
    }

    fn filter_new_yt_channel_id<'a>(&self, channel_ids: &'a [String]) -> Vec<&'a str> {
        channel_ids
            .iter()
            .filter(|c| !self.yt_channels.contains_key(*c))
            .map(|s| s.as_str())
            .collect()
    }

    async fn save(&self) {
        match serde_json::to_string(&self.yt_channels) {
            Ok(save_string) => {
                if let Err(e) = tokio::fs::write(&self.channel_save_path, save_string).await {
                    log::error!(
                        "Write to save file {} failed: {}",
                        self.channel_save_path,
                        e
                    );
                }
            }
            Err(e) => log::error!("Serialize channel save failed: {}", e),
        }

        if let Err(e) = tokio::fs::write(&self.video_save_path, self.yt_videos.to_string()).await {
            log::error!("Write to save file {} failed: {}", self.video_save_path, e);
        }
    }

    async fn restore(&mut self) {
        match tokio::fs::read_to_string(&self.channel_save_path).await {
            Ok(save_string) => {
                match serde_json::from_str::<HashMap<String, YtChannelSave>>(&save_string) {
                    Ok(channel) => self.yt_channels = channel,
                    Err(e) => log::error!(
                        "Deserialize save file {} failed: {}",
                        self.channel_save_path,
                        e
                    ),
                }
            }
            Err(e) => {
                log::error!("Read save file {} failed: {}", self.channel_save_path, e);
                return;
            }
        }

        match tokio::fs::read_to_string(&self.video_save_path).await {
            Ok(save_string) => self.yt_videos.extend_from_str(&save_string),
            Err(e) => log::error!("Read save file {} failed: {}", self.video_save_path, e),
        }
    }

    pub async fn update_channel_info(&mut self) {
        let now = Utc::now();
        let channel_ids = self
            .yt_channels
            .keys()
            .map(|s| s.clone())
            .collect::<Vec<String>>();
        for id in channel_ids.iter() {
            if now
                - self
                    .yt_channels
                    .get(id)
                    .expect("Get channel data from server data failed")
                    .last_time_used
                > chrono::Duration::minutes(self.channel_expire_min)
            {
                log::info!(
                    "Channel {id} was not accessed in the past {} mins. Will delete it.",
                    self.channel_expire_min
                );
                self.yt_channels.remove(id);
            }
        }
        if self.yt_channels.is_empty() {
            return;
        }
        match get_all_channels(
            self.yt_channels
                .keys()
                .map(|s| s.as_str())
                .collect::<Vec<&str>>()
                .as_slice(),
            &GetChannelParts::default().snippet().content_details(),
            &self.api_key,
        )
        .await
        {
            Err(e) => log::error!("Update channel info failed: {e:?}"),
            Ok(channel_res) => {
                for res in channel_res {
                    if res.snippet.is_none() {
                        log::error!(
                            "The channel {} resource doesn't has snippet property",
                            res.id
                        );
                    } else if res.contentDetails.is_none() {
                        log::error!(
                            "The channel {} resource doesn't has contentDetails property",
                            res.id
                        );
                    } else {
                        let snippet = res.snippet.unwrap();
                        let content_details = res.contentDetails.unwrap();
                        let channel_save = self
                            .yt_channels
                            .get_mut(&res.id)
                            .expect("Get channel save by id failed");
                        channel_save.custom_url = snippet.customUrl;
                        if let Some(thumbnail) = snippet.thumbnails.get("medium") {
                            channel_save.thumbnail = thumbnail.url.clone();
                        } else {
                            log::error!("The channel {} doesn't has \"medium\" thumbnail", res.id);
                        }
                        channel_save.title = snippet.title;
                        channel_save.upload_playlist = content_details.relatedPlaylists.uploads;
                    }
                }
            }
        }

        self.save().await;
    }

    pub fn touch_channel(&mut self, id: &str) {
        if let Some(ch) = self.yt_channels.get_mut(id) {
            ch.last_time_used = Utc::now();
        }
    }
}
