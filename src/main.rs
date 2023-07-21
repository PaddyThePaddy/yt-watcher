mod server;
mod yt_api;
use std::{net::SocketAddr, str::FromStr};

use serde::Deserialize;

const CONFIG_PATH: &'static str = "config.toml";

#[derive(Debug, Deserialize)]
struct Config {
    api_key: String,
    socket: String,
    refresh_interval: u64,
    log_level: String,
    tls: Option<Tls>,
}

#[derive(Debug, Deserialize)]
struct Tls {
    socket: String,
    cert: String,
    key: String,
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

    let mut new_logger = fern::Dispatch::new()
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
        .chain(fern::log_file("output.log").unwrap());
    match config.log_level.as_str() {
        "Off" => new_logger = new_logger.level(log::LevelFilter::Off),
        "Error" => new_logger = new_logger.level(log::LevelFilter::Error),
        "Warn" => new_logger = new_logger.level(log::LevelFilter::Warn),
        "Info" => new_logger = new_logger.level(log::LevelFilter::Info),
        "Debug" => new_logger = new_logger.level(log::LevelFilter::Debug),
        "Trace" => new_logger = new_logger.level(log::LevelFilter::Trace),
        _ => panic!("Invalid log level"),
    }
    new_logger.apply().unwrap();

    log::info!("starting");
    let socket = match SocketAddr::from_str(&config.socket) {
        Ok(s) => s,
        Err(e) => panic!("Invalid socket: {}", e),
    };
    let tls_info = config.tls.map(|tls| {
        (
            match SocketAddr::from_str(&tls.socket) {
                Ok(s) => s,
                Err(e) => panic!("Invalid socket: {}", e),
            },
            tls.cert,
            tls.key,
        )
    });
    server::server_start(socket, tls_info, &config.api_key, config.refresh_interval).await;
}
