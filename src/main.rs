mod server;
mod tw_api;
mod yt_api;

use once_cell::sync::Lazy;
use serde::Deserialize;

const CONFIG_PATH: &str = "config.toml";
static mut REQWEST_CLIENT: Lazy<reqwest::Client> = Lazy::new(|| {
    reqwest::Client::builder()
        .local_address(local_ip_address::local_ip().unwrap())
        .build()
        .unwrap()
});

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
#[allow(non_camel_case_types)]
enum Compression {
    none,
    gzip,
    dflate,
    brotli,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    api_key: String,
    socket: String,
    video_refresh_interval: u64,
    channel_refresh_interval: u64,
    channel_expire_min: i64,
    log_level: String,
    compression: Compression,
    tls: Option<Tls>,
    twitch_key: Option<TwAppKey>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Tls {
    socket: String,
    cert: String,
    key: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TwAppKey {
    client_id: String,
    client_secret: String,
}

#[tokio::main]
async fn main() {
    // and log using log crate macros!
    let config = match tokio::fs::read_to_string(CONFIG_PATH).await {
        Ok(config_file) => {
            println!("Found config file: {}", CONFIG_PATH);
            match toml::from_str::<Config>(&config_file) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Parse config file failed: {}", e);
                    std::process::exit(1)
                }
            }
        }
        Err(e) => {
            eprintln!("Open config file failed: {}", e);
            std::process::exit(1)
        }
    };

    fern::Dispatch::new()
        // Perform allocation-free log formatting
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                chrono::Utc::now(),
                record.level(),
                record.target(),
                message
            ))
        })
        .chain(std::io::stdout())
        .chain(fern::log_file("output.log").unwrap())
        .level(match config.log_level.to_lowercase().as_str() {
            "off" => log::LevelFilter::Off,
            "error" => log::LevelFilter::Error,
            "warn" => log::LevelFilter::Warn,
            "info" => log::LevelFilter::Info,
            "debug" => log::LevelFilter::Debug,
            "trace" => log::LevelFilter::Trace,
            _ => panic!("Invalid log level"),
        })
        .apply()
        .unwrap();

    log::info!("starting");
    server::server_start(&config).await;
}
