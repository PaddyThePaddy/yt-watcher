use std::{
    collections::{HashMap, HashSet},
    net::SocketAddr,
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use crate::{
    sync,
    tw_api::{structs::*, *},
    yt_api::{structs::*, *},
    TwAppKey,
};
use chrono::{DateTime, Timelike, Utc};
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
pub enum YtChannelInfoResponse {
    data(ChannelInfoData),
    error(String),
}

const CHANNELS_SAVE_FILE: &str = "channels.json";
const VIDEOS_SAVE_FILE: &str = "videos.txt";
pub async fn server_start(config: &crate::Config) {
    let http_socket = match SocketAddr::from_str(&config.socket) {
        Ok(s) => s,
        Err(e) => panic!("Invalid socket: {}", e),
    };
    let server_data = Arc::new(RwLock::new(
        ServerData::new(
            &config.api_key,
            CHANNELS_SAVE_FILE.to_string(),
            VIDEOS_SAVE_FILE.to_string(),
            config.channel_expire_min,
            &config.twitch_key,
        )
        .await,
    ));

    {
        server_data.write().await.restore().await;
        server_data.write().await.check_upcoming_event(false).await;
    }

    let server_data_clone = server_data.clone();
    let get_yt_channel_info = warp::get()
        .and(warp::path("yt-ch"))
        .and(warp::query::<HashMap<String, String>>())
        .then(move |query: HashMap<String, String>| {
            let server_data_clone2: Arc<RwLock<ServerData>> = server_data_clone.clone();
            async move {
                let response_data = match query.get("q") {
                    None => YtChannelInfoResponse::error(
                        "Please provide channel name with \"q\" get parameter".to_string(),
                    ),
                    Some(url) => {
                        if !validate_custom_url(
                            url.trim_start_matches("https://www.youtube.com/@")
                                .trim_start_matches("https://www.youtube.com/channel/")
                                .trim_start_matches("https://youtube.com/@")
                                .trim_start_matches("https://youtube.com/channel/")
                                .trim_start_matches('@')
                                .trim_end_matches("/featured"),
                        ) {
                            return serde_json::to_string(&YtChannelInfoResponse::error(
                                "The query string does not like a youtube user".to_string(),
                            ))
                            .unwrap();
                        }
                        match get_channel_id_by_url(&format!(
                            "https://www.youtube.com/channel/{}",
                            try_youtube_id(url).await
                        ))
                        .await
                        {
                            Ok(id) => {
                                if !server_data_clone2
                                    .read()
                                    .await
                                    .yt_channels
                                    .contains_key(&id)
                                {
                                    if let Err(e) = {
                                        server_data_clone2
                                            .write()
                                            .await
                                            .track_new_yt_channels(&[&id])
                                            .await
                                    } {
                                        log::error!("Track new youtube channel failed: {:?}", e);
                                        return serde_json::to_string(
                                            &YtChannelInfoResponse::error(format!(
                                                "Track new youtube channel failed: {:?}",
                                                e
                                            )),
                                        )
                                        .unwrap();
                                    }
                                }
                                let mut server_data = server_data_clone2.write().await;
                                server_data.touch_yt_channel(&id);
                                let channel_save = server_data.yt_channels.get(&id).unwrap();
                                YtChannelInfoResponse::data(ChannelInfoData {
                                    id,
                                    title: channel_save.title.clone(),
                                    custom_url: channel_save.custom_url.clone(),
                                    thumbnail: channel_save.thumbnail.clone(),
                                })
                            }
                            Err(e) => YtChannelInfoResponse::error(format!(
                                "Failed to get channel id: {:?}",
                                e
                            )),
                        }
                    }
                };
                serde_json::to_string(&response_data).unwrap()
            }
        });

    let server_data_clone = server_data.clone();
    let get_tw_channel_info = warp::get()
        .and(warp::path("tw-ch"))
        .and(warp::query::<HashMap<String, String>>())
        .then(move |query: HashMap<String, String>| {
            let server_data_clone2: Arc<RwLock<ServerData>> = server_data_clone.clone();
            async move {
                let query_string = match query.get("q") {
                    Some(s) => s,
                    None => {
                        return serde_json::to_string(&YtChannelInfoResponse::error(
                            "Please provide channel name with \"q\" get parameter".to_string(),
                        ))
                        .unwrap();
                    }
                };
                let query_string = query_string
                    .trim_start_matches("https://www.twitch.tv/")
                    .to_string();
                if !validate_user_login(&query_string) {
                    return serde_json::to_string(&YtChannelInfoResponse::error(
                        "The query string does not like a twitter user login".to_string(),
                    ))
                    .unwrap();
                }
                let search_result =
                    if let Some(client) = &mut server_data_clone2.write().await.tw_client {
                        match client
                            .get_user_info(&[UserIdentity::Login(query_string)])
                            .await
                        {
                            Ok(r) => r,
                            Err(e) => {
                                log::error!("Search channel failed: {e}");
                                return serde_json::to_string(&YtChannelInfoResponse::error(
                                    format!("Search channel failed: {e}",),
                                ))
                                .unwrap();
                            }
                        }
                    } else {
                        log::error!("Twitch client is not initialized");
                        return serde_json::to_string(&YtChannelInfoResponse::error(
                            "Twitch client is not initialized".to_string(),
                        ))
                        .unwrap();
                    };
                if let Some(c) = search_result.first() {
                    let mut server_data = server_data_clone2.write().await;
                    if !server_data.tw_channels.contains_key(&c.login) {
                        server_data.tw_channels.insert(
                            c.login.clone(),
                            TwChannelSave {
                                id: c.id.clone(),
                                login: c.login.clone(),
                                profile_img: c.profile_image_url.clone(),
                                name: c.display_name.clone(),
                                last_time_used: Utc::now(),
                            },
                        );
                        server_data.check_tw_upcoming_event(None).await;
                        server_data.save().await;
                    }
                    server_data.touch_tw_channel(&c.login);
                    let channel_save = server_data.tw_channels.get(&c.login).unwrap();
                    serde_json::to_string(&YtChannelInfoResponse::data(ChannelInfoData {
                        id: channel_save.id.clone(),
                        custom_url: channel_save.login.clone(),
                        title: channel_save.name.clone(),
                        thumbnail: channel_save.profile_img.clone(),
                    }))
                    .unwrap()
                } else {
                    log::error!("Search channel failed: Not found");
                    serde_json::to_string(&YtChannelInfoResponse::error(
                        "Search channel failed: Not found".to_string(),
                    ))
                    .unwrap()
                }
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
                let mut yt_channel_ids: Vec<String> = vec![];
                let mut tw_channel_logins: Vec<String> = vec![];
                if let Some(query_str) = query.get("yt-ch") {
                    for id in query_str.split(',') {
                        yt_channel_ids.push(try_youtube_id(id).await);
                    }
                }
                if let Some(query_str) = query.get("tw-ch") {
                    tw_channel_logins.extend(query_str.split(',').map(|s| s.to_string()));
                }
                if let Some(sync_key) = query.get("key") {
                    let key = uuid::Uuid::from_str(sync_key).unwrap_or_default();
                    if let Some(ch) = sync::get_yt_channel(&key).await {
                        for id in ch.iter() {
                            yt_channel_ids.push(try_youtube_id(id).await);
                        }
                    }

                    if let Some(ch) = sync::get_tw_channel(&key).await {
                        tw_channel_logins.extend(ch.iter().cloned());
                    }
                }
                let new_yt_channel_ids = {
                    server_data_clone2
                        .read()
                        .await
                        .filter_new_yt_channel_id(&yt_channel_ids)
                };
                if !new_yt_channel_ids.is_empty() {
                    if let Err(e) = server_data_clone2
                        .write()
                        .await
                        .track_new_yt_channels(&new_yt_channel_ids)
                        .await
                    {
                        log::error!("Track new youtube channel failed: {:?}", e);
                    };
                }

                let new_tw_channel_logins = {
                    server_data_clone2
                        .read()
                        .await
                        .filter_new_tw_channel_login(&tw_channel_logins)
                };
                if !new_tw_channel_logins.is_empty() {
                    server_data_clone2
                        .write()
                        .await
                        .track_new_tw_channels(&new_tw_channel_logins)
                        .await;
                }
                let events = {
                    let mut server_data = server_data_clone2.write().await;
                    for id in yt_channel_ids.iter() {
                        server_data.touch_yt_channel(id);
                    }
                    for login in tw_channel_logins.iter() {
                        server_data.touch_tw_channel(login);
                    }
                    server_data.events.clone()
                };
                response.extend(events.into_iter().filter(|e| match &e.source {
                    EventSource::YoutubeChannel(c) => yt_channel_ids.contains(&c.id),
                    EventSource::TwitchChannel(c) => tw_channel_logins.contains(&c.login),
                }));
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
                let alarm_enabled = match query.get("alarm") {
                    Some(v) => v.to_lowercase() == "true" || v.to_lowercase() == "yes",
                    None => false,
                };
                cal.name("Stream Calendar");
                let mut yt_channel_ids: Vec<String> = vec![];
                let mut tw_channel_logins = vec![];
                if let Some(query_str) = query.get("yt-ch") {
                    for id in query_str.split(',') {
                        yt_channel_ids.push(try_youtube_id(id).await);
                    }
                }
                if let Some(query_str) = query.get("tw-ch") {
                    tw_channel_logins.extend(query_str.split(',').map(|s| s.to_string()));
                }
                if let Some(sync_key) = query.get("key") {
                    let key = uuid::Uuid::from_str(sync_key).unwrap_or_default();
                    if let Some(ch) = sync::get_yt_channel(&key).await {
                        for id in ch.iter() {
                            yt_channel_ids.push(try_youtube_id(id).await);
                        }
                    }

                    if let Some(ch) = sync::get_tw_channel(&key).await {
                        tw_channel_logins.extend(ch.iter().cloned());
                    }
                }
                let new_yt_channel_ids = {
                    server_data_clone2
                        .read()
                        .await
                        .filter_new_yt_channel_id(&yt_channel_ids)
                };
                if !new_yt_channel_ids.is_empty() {
                    if let Err(e) = server_data_clone2
                        .write()
                        .await
                        .track_new_yt_channels(&new_yt_channel_ids)
                        .await
                    {
                        log::error!("Track new youtube channel failed: {:?}", e);
                    };
                }

                let new_tw_channel_logins = {
                    server_data_clone2
                        .read()
                        .await
                        .filter_new_tw_channel_login(&tw_channel_logins)
                };
                if !new_tw_channel_logins.is_empty() {
                    server_data_clone2
                        .write()
                        .await
                        .track_new_tw_channels(&new_tw_channel_logins)
                        .await;
                }
                let events = {
                    let mut server_data = server_data_clone2.write().await;
                    for id in yt_channel_ids.iter() {
                        server_data.touch_yt_channel(id);
                    }
                    for login in tw_channel_logins.iter() {
                        server_data.touch_tw_channel(login);
                    }
                    server_data.events.clone()
                };
                cal.extend(
                    events
                        .into_iter()
                        .filter(|e: &UpcomingEvent| match &e.source {
                            EventSource::YoutubeChannel(c) => yt_channel_ids.contains(&c.id),
                            EventSource::TwitchChannel(c) => tw_channel_logins.contains(&c.login),
                        })
                        .map(|e: UpcomingEvent| e.to_ical_event(alarm_enabled)),
                );
                cal.done().to_string()
            }
        });

    let sync_key_endpoint = warp::get().and(warp::path("sync")).and(
        warp::path("new")
            .then(|| async {
                serde_json::to_string(&HashMap::from([("key", sync::new_key().await)]))
                    .unwrap_or_default()
            })
            .or(warp::path("push")
                .and(warp::query::<HashMap<String, String>>())
                .then(|query: HashMap<String, String>| async move {
                    if let Some(key) = query.get("key") {
                        let key = uuid::Uuid::from_str(key).unwrap_or_default();
                        if let Some(yt_ch) = query.get("yt-ch") {
                            if sync::set_yt_channels(
                                &key,
                                yt_ch.split(',').filter(|s| !s.is_empty()),
                            )
                            .await
                            .is_err()
                            {
                                return serde_json::to_string(&HashMap::from([(
                                    "result".to_string(),
                                    "failed",
                                )]))
                                .unwrap_or_default();
                            }
                        }
                        if let Some(tw_ch) = query.get("tw-ch") {
                            if sync::set_tw_channels(
                                &key,
                                tw_ch.split(',').filter(|s| !s.is_empty()),
                            )
                            .await
                            .is_err()
                            {
                                return serde_json::to_string(&HashMap::from([(
                                    "result".to_string(),
                                    "failed",
                                )]))
                                .unwrap_or_default();
                            }
                        }
                        serde_json::to_string(&HashMap::from([("result".to_string(), "Ok")]))
                            .unwrap_or_default()
                    } else {
                        serde_json::to_string(&HashMap::from([(
                            "result",
                            "error: No key specified",
                        )]))
                        .unwrap_or_default()
                    }
                }))
            .or(warp::path("pull")
                .and(warp::query::<HashMap<String, String>>())
                .then(|query: HashMap<String, String>| async move {
                    if let Some(key) = query.get("key") {
                        let mut response: HashMap<&str, HashSet<String>> = HashMap::new();
                        let key = uuid::Uuid::from_str(key).unwrap_or_default();
                        if let Some(yt_ch) = sync::get_yt_channel(&key).await {
                            response.insert("yt_ch", yt_ch);
                        }
                        if let Some(tw_ch) = sync::get_tw_channel(&key).await {
                            response.insert("tw_ch", tw_ch);
                        }
                        serde_json::to_string(&response).unwrap_or_default()
                    } else {
                        serde_json::to_string(&HashMap::from([("error", "No key specified")]))
                            .unwrap_or_default()
                    }
                })),
    );

    let server_data_clone = server_data.clone();
    let notice_yt_video_endpoint = warp::get()
        .and(warp::path("notice-yt-video"))
        .and(warp::query::<HashMap<String, String>>())
        .then(move |query: HashMap<String, String>| {
            let server_data_clone2 = server_data_clone.clone();
            async move {
                if let Some(id_list) = query.get("id") {
                    for id in id_list.split(',') {
                        server_data_clone2
                            .write()
                            .await
                            .yt_videos
                            .push_checked(id.to_string());
                    }
                    serde_json::to_string(&HashMap::from([("result", "Ok")])).unwrap_or_default()
                } else {
                    serde_json::to_string(&HashMap::from([(
                        "result",
                        "no 'id' parameter is provided",
                    )]))
                    .unwrap_or_default()
                }
            }
        });

    let server_data_clone = server_data.clone();
    let video_refresh_interval = config.video_refresh_interval;
    let video_refresh_delay = config.video_refresh_delay.unwrap_or(60);
    let use_youtube_api_per_hour = config.use_youtube_api_per_hour as u64;
    let _handle = tokio::spawn(async move {
        loop {
            if video_refresh_interval > 1 && video_refresh_interval <= 60 {
                let now = Utc::now();
                let minutes =
                    (video_refresh_interval - 1) - now.minute() as u64 % video_refresh_interval;
                let seconds = 60 - now.second() as u64;
                tokio::time::sleep(Duration::from_secs(
                    (minutes * 60 + seconds + video_refresh_delay) % (video_refresh_interval * 60),
                ))
                .await;
            } else {
                tokio::time::sleep(Duration::from_secs(60 * video_refresh_interval)).await;
            }
            log::info!("Updating upcoming event");
            let mut data = server_data_clone.write().await;
            let now = Utc::now();
            if use_youtube_api_per_hour != 0
                && (now.minute() as u64 % (60 / use_youtube_api_per_hour))
                    + if now.second() == 0 { 0 } else { 1 }
                    < video_refresh_interval
            {
                data.check_upcoming_event(true).await;
            } else {
                data.check_upcoming_event(false).await;
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
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
    warp::serve(
        warp::get().and(
            get_yt_channel_info
                .or(get_data_endpoint)
                .or(get_calendar_endpoint)
                .or(get_tw_channel_info)
                .or(notice_yt_video_endpoint)
                .or(sync_key_endpoint),
        ),
    )
    .run(http_socket)
    .await;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ChannelSave {
    yt: HashMap<String, YtChannelSave>,
    tw: HashMap<String, TwChannelSave>,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
struct TwChannelSave {
    id: String,
    login: String,
    profile_img: String,
    name: String,
    last_time_used: DateTime<Utc>,
}

impl From<crate::tw_api::structs::UserInformation> for TwChannelSave {
    fn from(value: crate::tw_api::structs::UserInformation) -> Self {
        Self {
            id: value.id,
            login: value.login,
            profile_img: value.profile_image_url,
            name: value.display_name,
            last_time_used: Utc::now(),
        }
    }
}

impl PartialEq for TwChannelSave {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for TwChannelSave {}

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

    fn set(&mut self, new_value: HashSet<String>) {
        self.ids = new_value;
    }

    fn push_checked(&mut self, new_value: String) {
        self.ids.insert(new_value);
    }

    //fn remove(&mut self, value: &String) {
    //    self.ids.remove(value);
    //}

    fn extend_from_str(&mut self, value: &str) {
        let mut new_value = value
            .split('\n')
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
pub struct YtChannelBrief {
    id: String,
    thumbnail_url: String,
    title: String,
    custom_url: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct TwChannelBrief {
    id: String,
    thumbnail_url: String,
    title: String,
    login: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum EventSource {
    YoutubeChannel(YtChannelBrief),
    TwitchChannel(TwChannelBrief),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UpcomingEvent {
    start_date_time: DateTime<Utc>,
    start_timestamp_millis: i64,
    thumbnail_url: Option<String>,
    title: String,
    description: String,
    target_url: String,
    ongoing: bool,
    source: EventSource,
    uid: String,
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
            EventSource::TwitchChannel(c) => {
                description += &format!("{}({})\n\n", c.title, c.login);
            }
        }
        description += &self.description;
        builder.description(&description);
        builder.url(&self.target_url);
        if alarm_enabled {
            builder.alarm(Alarm::display(&self.title, -chrono::Duration::minutes(5)));
        }
        builder.uid(&self.uid);
        builder.done()
    }
}

impl PartialEq for UpcomingEvent {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid
    }
}

impl Eq for UpcomingEvent {}

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
                chrono::DateTime::from_str(actual_start_time)
                    .map_err(|e| ConvertToUpcomingEventError::DecodeError(format!("{}", e)))?
            } else if let Some(scheduled_start_time) = &live_info.scheduledStartTime {
                chrono::DateTime::from_str(scheduled_start_time)
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
            None => Err(ConvertToUpcomingEventError::MissingInformation(
                "snippet".to_string(),
            )),
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
                            Some(c) => YtChannelBrief {
                                id: c.id.clone(),
                                thumbnail_url: c.thumbnail.clone(),
                                title: c.title.clone(),
                                custom_url: c.custom_url.clone(),
                            },
                        },
                    ),
                    uid: format!("{}@yt@yt-watcher", value.0.id),
                });
            }
        }
    }
}

impl From<(StreamInformation, String)> for UpcomingEvent {
    fn from(value: (StreamInformation, String)) -> Self {
        Self {
            start_timestamp_millis: value.0.started_at.timestamp_millis(),
            start_date_time: value.0.started_at,
            thumbnail_url: Some(process_thumbnail_url(&value.0.thumbnail_url, 320, 180)),
            title: value.0.title,
            description: value.0.game_name,
            target_url: format!("https://www.twitch.tv/{}", &value.0.user_login),
            ongoing: true,
            uid: format!("{}@twitch@yt-watcher", &value.0.user_login),
            source: EventSource::TwitchChannel(TwChannelBrief {
                id: value.0.user_id,
                title: value.0.user_name,
                login: value.0.user_login,
                thumbnail_url: value.1,
            }),
        }
    }
}

#[derive(Default)]
pub struct ServerData {
    yt_channels: HashMap<String, YtChannelSave>,
    yt_videos: YtVideosSave,
    tw_channels: HashMap<String, TwChannelSave>,
    tw_client: Option<TwApiClient>,
    events: Vec<UpcomingEvent>,
    api_key: String,
    channel_save_path: String,
    video_save_path: String,
    channel_expire_min: i64,
}

impl ServerData {
    async fn new(
        api_key: &str,
        channel_save_path: String,
        video_save_path: String,
        channel_expire_min: i64,
        twitch_app_key: &Option<TwAppKey>,
    ) -> Self {
        if let Some(tw_key) = twitch_app_key {
            Self {
                api_key: api_key.to_string(),
                channel_save_path,
                video_save_path,
                channel_expire_min,
                tw_client: Some(
                    TwApiClient::new(tw_key.client_id.clone(), tw_key.client_secret.clone())
                        .await
                        .expect("Incorrect Twitch app key"),
                ),
                ..Default::default()
            }
        } else {
            Self {
                api_key: api_key.to_string(),
                channel_save_path,
                video_save_path,
                channel_expire_min,
                tw_client: None,
                ..Default::default()
            }
        }
    }

    pub async fn check_upcoming_event(&mut self, use_youtube_channel_api: bool) {
        let mut events = vec![];
        let mut unchecked_video_ids = vec![];
        let mut first_video_after_all_stream_map: HashMap<String, String> = HashMap::new();
        self.yt_videos.dump(&mut unchecked_video_ids);
        for c in self.yt_channels.values() {
            if use_youtube_channel_api {
                match get_playlist_items(
                    &c.upload_playlist,
                    &GetPlaylistItemParts::default().content_details(),
                    None,
                    &self.api_key,
                )
                .await
                {
                    Err(e) => log::error!("Fail to get playlist item of channel {}: {:?}", c.id, e),
                    Ok(resp) => {
                        for v in resp.value {
                            if let Some(content_detail) = v.contentDetails {
                                if content_detail.videoId == c.first_video_after_all_stream {
                                    break;
                                }
                                if !unchecked_video_ids.contains(&content_detail.videoId) {
                                    unchecked_video_ids.push(content_detail.videoId);
                                }
                            } else {
                                log::error!(
                                    "Playlist item {} does not have contentDetail field",
                                    v.id
                                );
                            }
                        }
                    }
                }
            } else {
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
                            if !unchecked_video_ids.contains(&v) {
                                unchecked_video_ids.push(v);
                            }
                        }
                    }
                }
            }
        }

        match get_all_video_items(
            unchecked_video_ids.as_slice(),
            &GetVideoParts::default().snippet().live_streaming_details(),
            &self.api_key,
        )
        .await
        {
            Err(e) => log::error!("Fail to get video items: {:?}", e),
            Ok(resp) => {
                for v in resp.iter() {
                    if let Some(snippet) = &v.snippet {
                        if !self.yt_channels.contains_key(&snippet.channelId) {
                            log::debug!("Video {} does not belongs to any tracking channel", v.id);
                            continue;
                        }
                        if snippet.liveBroadcastContent == "none" {
                            if !first_video_after_all_stream_map.contains_key(&snippet.channelId) {
                                log::debug!(
                                    "Channel {}({}) does not has first video, insert {}({})",
                                    snippet.channelTitle,
                                    snippet.channelId,
                                    snippet.title,
                                    v.id
                                );
                                first_video_after_all_stream_map
                                    .insert(snippet.channelId.clone(), v.id.clone());
                            } else {
                                log::debug!(
                                    "Channel {}({}) alread has first video. Skippet video {}({})",
                                    snippet.channelTitle,
                                    snippet.channelId,
                                    snippet.title,
                                    v.id
                                );
                            }
                        } else {
                            first_video_after_all_stream_map.remove(&snippet.channelId);
                            log::debug!("Remove first video of channel {}({}). Because found new live stream {}({})", snippet.channelTitle, snippet.channelId, snippet.title, v.id);
                        }
                    }
                    match UpcomingEvent::try_from((v, &*self)) {
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
                self.yt_videos.set(
                    resp.iter()
                        .filter(|r| {
                            if let Some(s) = &r.snippet {
                                s.liveBroadcastContent != "none"
                            } else {
                                false
                            }
                        })
                        .map(|v| v.id.clone())
                        .collect(),
                )
            }
        }

        first_video_after_all_stream_map
            .into_iter()
            .for_each(|(channel_id, video_id)| {
                if let Some(c) = self.yt_channels.get_mut(&channel_id) {
                    c.first_video_after_all_stream = video_id;
                }
            });
        self.check_tw_upcoming_event(Some(&mut events)).await;
        self.events = events;
        self.save().await;
    }
    pub async fn check_tw_upcoming_event(&mut self, events: Option<&mut Vec<UpcomingEvent>>) {
        let mut events_vec = vec![];
        let is_none = events.is_none();
        let event_ref = events.unwrap_or(&mut events_vec);
        let channel_ids = self
            .tw_channels
            .values()
            .map(|c| UserIdentity::Id(c.id.clone()))
            .collect::<Vec<UserIdentity>>();
        match &mut self.tw_client {
            None => log::error!("Twitch client is not initialized"),
            Some(client) => match client.get_stream_info(&channel_ids).await {
                Err(e) => log::error!("Get stream info of channel {channel_ids:?} failed: {e}"),
                Ok(streams) => event_ref.extend(streams.into_iter().map(|s| {
                    let profile_url: String = self
                        .tw_channels
                        .get(&s.user_login)
                        .unwrap()
                        .profile_img
                        .clone();
                    (s, profile_url).into()
                })),
            },
        }

        if is_none {
            for e in events_vec.into_iter() {
                if let Some(n) = self.events.iter().position(|event| *event == e) {
                    self.events.remove(n);
                }
                self.events.push(e);
            }
        }
    }
    pub async fn track_new_yt_channels(&mut self, ids: &[&str]) -> Result<(), YtApiError> {
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

    pub async fn track_new_tw_channels(&mut self, logins: &[String]) {
        match &mut self.tw_client {
            Some(client) => match client
                .get_user_info(
                    &logins
                        .iter()
                        .map(|s| UserIdentity::Login(s.clone()))
                        .collect::<Vec<UserIdentity>>(),
                )
                .await
            {
                Ok(channels) => {
                    self.tw_channels.extend(channels.iter().map(|c| {
                        log::info!("Tracking new twitch channel: {}", &c.login);
                        (c.login.clone(), c.clone().into())
                    }));
                    match client
                        .get_stream_info(
                            channels
                                .iter()
                                .map(|c| UserIdentity::Login(c.login.clone()))
                                .collect::<Vec<UserIdentity>>()
                                .as_slice(),
                        )
                        .await
                    {
                        Ok(streams) => self.events.extend(streams.into_iter().map(|s| {
                            (
                                s.clone(),
                                self.tw_channels
                                    .get(&s.user_login)
                                    .unwrap()
                                    .profile_img
                                    .clone(),
                            )
                                .into()
                        })),
                        Err(e) => log::error!(
                            "Get stream info of channel {:?} failed: {e}",
                            channels
                                .iter()
                                .map(|c| c.login.clone())
                                .collect::<Vec<String>>()
                        ),
                    }
                }
                Err(e) => log::error!("Get user info failed: {e}"),
            },
            None => log::error!("Twitch client is not initialized"),
        }
        self.check_tw_upcoming_event(None).await;
        self.save().await;
    }

    fn filter_new_yt_channel_id<'a>(&self, channel_ids: &'a [String]) -> Vec<&'a str> {
        channel_ids
            .iter()
            .filter(|c| !self.yt_channels.contains_key(*c))
            .map(|s| s.as_str())
            .collect()
    }

    fn filter_new_tw_channel_login(&self, channel_logins: &[String]) -> Vec<String> {
        channel_logins
            .iter()
            .filter(|l| !self.tw_channels.contains_key(*l))
            .cloned()
            .collect()
    }

    async fn save(&self) {
        match serde_json::to_string(&ChannelSave {
            yt: self.yt_channels.clone(),
            tw: self.tw_channels.clone(),
        }) {
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
            Ok(save_string) => match serde_json::from_str::<ChannelSave>(&save_string) {
                Ok(save) => {
                    self.yt_channels = save.yt;
                    self.tw_channels = save.tw;
                }
                Err(e) => log::error!(
                    "Deserialize save file {} failed: {}",
                    self.channel_save_path,
                    e
                ),
            },
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
        let channel_ids = self.yt_channels.keys().cloned().collect::<Vec<String>>();
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
        let tw_channel_logins = self.tw_channels.keys().cloned().collect::<Vec<String>>();
        for login in tw_channel_logins.iter() {
            if now
                - self
                    .tw_channels
                    .get(login)
                    .expect("Get twitch channel data from server data failed")
                    .last_time_used
                > chrono::Duration::minutes(self.channel_expire_min)
            {
                log::info!(
                    "Twitch channel {login} was not accessed in the past {} mins. Will delete it.",
                    self.channel_expire_min
                );
                self.tw_channels.remove(login);
            }
        }
        if self.yt_channels.is_empty() && self.tw_channels.is_empty() {
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

        if let Some(client) = &mut self.tw_client {
            match client
                .get_user_info(
                    &self
                        .tw_channels
                        .keys()
                        .map(|s| UserIdentity::Login(s.clone()))
                        .collect::<Vec<UserIdentity>>(),
                )
                .await
            {
                Err(e) => log::error!("Get user info failed: {e}"),
                Ok(users) => {
                    for user in users.iter() {
                        let channel_save = self
                            .tw_channels
                            .get_mut(&user.login)
                            .expect("Get channel save by login failed");
                        channel_save.login = user.login.clone();
                        channel_save.profile_img = user.profile_image_url.clone();
                        channel_save.name = user.display_name.clone();
                    }
                }
            }
        }

        self.save().await;
    }

    pub fn touch_yt_channel(&mut self, id: &str) {
        if let Some(ch) = self.yt_channels.get_mut(id) {
            ch.last_time_used = Utc::now();
        }
    }

    pub fn touch_tw_channel(&mut self, login: &str) {
        if let Some(ch) = self.tw_channels.get_mut(login) {
            ch.last_time_used = Utc::now();
        }
    }
}
