use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tokio::{sync::Mutex, task::JoinHandle};
use uuid::Uuid;

static MAX_KEEP_TIME: Lazy<chrono::Duration> = Lazy::new(|| chrono::Duration::days(30));
static SAVE_INTERVAL: Lazy<std::time::Duration> =
    Lazy::new(|| std::time::Duration::from_secs(10 * 60));
static mut SAVER_HANDLE: Option<JoinHandle<()>> = None;
const SAVE_NAME: &str = "sync_keys.json";

#[derive(Debug, Serialize, Deserialize)]
struct KeySave {
    key: Uuid,
    last_used: DateTime<Utc>,
    #[serde(default)]
    yt_channels: HashSet<String>,
    #[serde(default)]
    tw_channels: HashSet<String>,
}

impl Default for KeySave {
    fn default() -> Self {
        Self {
            key: Uuid::new_v4(),
            last_used: Utc::now(),
            yt_channels: HashSet::new(),
            tw_channels: HashSet::new(),
        }
    }
}

impl KeySave {
    pub fn key(&self) -> &Uuid {
        &self.key
    }

    pub fn yt_channels(&mut self) -> &mut HashSet<String> {
        self.last_used = Utc::now();
        &mut self.yt_channels
    }

    pub fn tw_channels(&mut self) -> &mut HashSet<String> {
        self.last_used = Utc::now();
        &mut self.tw_channels
    }

    pub fn last_used(&self) -> &DateTime<Utc> {
        &self.last_used
    }
}

static SYNC_KEY_SAVES: Lazy<Mutex<Vec<KeySave>>> = Lazy::new(|| {
    Mutex::new(
        serde_json::from_str::<Vec<KeySave>>(
            &std::fs::read_to_string(SAVE_NAME)
                .map_err(|e| {
                    log::error!("Open sync key save failed: {e}");
                    e
                })
                .unwrap_or_default(),
        )
        .map_err(|e| {
            log::error!("Deserialize sync key save failed: {e}");
            e
        })
        .unwrap_or_default(),
    )
});

pub async fn new_key() -> Uuid {
    let new_key = KeySave::default();
    let id = *new_key.key();
    log::info!("Generated new sync key: {id}");
    SYNC_KEY_SAVES.lock().await.push(new_key);
    save().await;
    id
}

pub async fn get_yt_channel(key: &Uuid) -> Option<HashSet<String>> {
    init_saver();
    SYNC_KEY_SAVES
        .lock()
        .await
        .iter_mut()
        .find(|s| s.key() == key)
        .map(|s| s.yt_channels().clone())
}

pub async fn get_tw_channel(key: &Uuid) -> Option<HashSet<String>> {
    init_saver();
    SYNC_KEY_SAVES
        .lock()
        .await
        .iter_mut()
        .find(|s| s.key() == key)
        .map(|s| s.tw_channels().clone())
}

pub async fn set_yt_channels(
    key: &Uuid,
    ch: impl Iterator<Item = impl ToString>,
) -> Result<(), ()> {
    init_saver();
    let resp = SYNC_KEY_SAVES
        .lock()
        .await
        .iter_mut()
        .find(|s| s.key() == key)
        .map(|s| {
            *s.yt_channels() = ch.map(|s| s.to_string()).collect();
        })
        .ok_or(());
    save().await;
    resp
}

pub async fn set_tw_channels(
    key: &Uuid,
    ch: impl Iterator<Item = impl ToString>,
) -> Result<(), ()> {
    init_saver();
    let resp = SYNC_KEY_SAVES
        .lock()
        .await
        .iter_mut()
        .find(|s| s.key() == key)
        .map(|s| {
            *s.tw_channels() = ch.map(|s| s.to_string()).collect();
        })
        .ok_or(());
    save().await;
    resp
}

pub async fn save() {
    trim().await;
    match serde_json::to_string(&*SYNC_KEY_SAVES.lock().await) {
        Ok(s) => {
            if let Err(e) = tokio::fs::write(SAVE_NAME, s.as_bytes()).await {
                log::error!("Save sync keys to file failed: {e}");
            }
        }
        Err(e) => {
            log::error!("Serialize sync keys to file failed: {e}");
        }
    }
}

pub async fn trim() {
    let mut remove_idx = vec![];
    let mut lock = SYNC_KEY_SAVES.lock().await;
    let now = Utc::now();
    for (idx, v) in lock.iter().enumerate().rev() {
        if now - *v.last_used() > *MAX_KEEP_TIME {
            log::info!("Remove sync key {} due to not used", v.key);
            remove_idx.push(idx);
        }
    }
    for idx in remove_idx.into_iter() {
        lock.remove(idx);
    }
}

fn init_saver() {
    unsafe {
        if SAVER_HANDLE.is_none() {
            SAVER_HANDLE = Some(tokio::spawn(async {
                loop {
                    tokio::time::sleep(*SAVE_INTERVAL).await;
                    save().await;
                }
            }));
        }
    }
}
