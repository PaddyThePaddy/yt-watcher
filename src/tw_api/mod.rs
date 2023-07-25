#![allow(dead_code)]

use crate::REQWEST_CLIENT;
pub mod structs;
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::Response;
use serde::Serialize;
use structs::*;

static USER_LOGIN_PATTERN: Lazy<Regex> = Lazy::new(|| regex::Regex::new(r"^\w+$").unwrap());

pub fn validate_user_login(login: &str) -> bool {
    USER_LOGIN_PATTERN.is_match(login)
}

pub struct TwApiClient {
    client_id: String,
    client_secret: String,
    access_token: String,
}

#[derive(Debug)]
pub enum UserIdentity {
    Login(String),
    Id(String),
}

impl TwApiClient {
    pub async fn new(
        client_id: impl ToString,
        client_secret: impl ToString,
    ) -> Result<Self, reqwest::Error> {
        let client_id = client_id.to_string();
        let client_secret = client_secret.to_string();
        let access_token = Self::require_access_token(&client_id, &client_secret).await?;
        Ok(Self {
            client_id,
            client_secret,
            access_token,
        })
    }
    async fn require_access_token(id: &str, secret: &str) -> Result<String, reqwest::Error> {
        unsafe {
            let response: TokenResponse = REQWEST_CLIENT
                .post("https://id.twitch.tv/oauth2/token")
                .query(&[
                    ("client_id", id),
                    ("client_secret", secret),
                    ("grant_type", "client_credentials"),
                ])
                .send()
                .await?
                .json()
                .await?;
            Ok(response.access_token)
        }
    }

    async fn post_request<T: Serialize + ?Sized>(
        &mut self,
        url: &str,
        args: &T,
    ) -> Result<Response, reqwest::Error> {
        unsafe {
            let mut response = REQWEST_CLIENT
                .post(url)
                .query(args)
                .header("Authorization", format!("Bearer {}", self.access_token))
                .header("Client-Id", &self.client_id)
                .send()
                .await?
                .error_for_status();
            if let Err(e) = &response {
                if let Some(code) = e.status() {
                    if code.as_u16() == 401 {
                        let new_access_token =
                            Self::require_access_token(&self.client_id, &self.client_secret)
                                .await?;
                        self.access_token = new_access_token;
                        response = REQWEST_CLIENT
                            .post(url)
                            .query(args)
                            .header("Authorization", format!("Bearer {}", self.access_token))
                            .header("Client-Id", &self.client_id)
                            .send()
                            .await;
                    }
                }
            }
            response
        }
    }

    async fn get_request<T: Serialize + ?Sized + std::fmt::Debug>(
        &mut self,
        url: &str,
        args: &T,
    ) -> Result<Response, reqwest::Error> {
        unsafe {
            let mut response = REQWEST_CLIENT
                .get(url)
                .query(args)
                .header("Authorization", format!("Bearer {}", self.access_token))
                .header("Client-Id", &self.client_id)
                .send()
                .await?
                .error_for_status();
            if let Err(e) = &response {
                if let Some(code) = e.status() {
                    if code.as_u16() == 401 {
                        let new_access_token =
                            Self::require_access_token(&self.client_id, &self.client_secret)
                                .await?;
                        self.access_token = new_access_token;
                        response = REQWEST_CLIENT
                            .get(url)
                            .query(args)
                            .header("Authorization", format!("Bearer {}", self.access_token))
                            .header("Client-Id", &self.client_id)
                            .send()
                            .await;
                    }
                }
            }
            response
        }
    }

    pub async fn search_channel(
        &mut self,
        query: &str,
    ) -> Result<Vec<ChannelSearchResult>, reqwest::Error> {
        log::debug!("Search twitch channel: {query}");
        Ok(self
            .get_request(
                "https://api.twitch.tv/helix/search/channels",
                &[("query", query)],
            )
            .await?
            .json::<PagedResponses<ChannelSearchResult>>()
            .await?
            .data)
    }

    pub async fn get_channel_info(
        &mut self,
        channel_ids: &[String],
    ) -> Result<Vec<ChannelInformation>, reqwest::Error> {
        let mut result = vec![];
        let mut idx = 0;
        while idx * 100 < channel_ids.len() {
            result.extend(
                self.get_request(
                    "https://api.twitch.tv/helix/channels",
                    &channel_ids[idx * 100..channel_ids.len().min(idx * 100 + 100)]
                        .iter()
                        .map(|id| ("broadcaster_id", id))
                        .collect::<Vec<(&str, &String)>>(),
                )
                .await?
                .json::<Responses<ChannelInformation>>()
                .await?
                .data
                .into_iter(),
            );
            idx += 1;
        }
        Ok(result)
    }

    pub async fn get_stream_info(
        &mut self,
        identities: &[UserIdentity],
    ) -> Result<Vec<StreamInformation>, reqwest::Error> {
        let mut result = vec![];
        let mut idx = 0;
        while idx * 100 < identities.len() {
            result.extend(
                self.get_request(
                    "https://api.twitch.tv/helix/streams",
                    &identities[idx * 100..identities.len().min(idx * 100 + 100)]
                        .iter()
                        .map(|id| match id {
                            UserIdentity::Id(s) => ("user_id", s),
                            UserIdentity::Login(s) => ("user_login", s),
                        })
                        .collect::<Vec<(&str, &String)>>(),
                )
                .await?
                .json::<PagedResponses<StreamInformation>>()
                .await?
                .data
                .into_iter(),
            );
            idx += 1;
        }
        Ok(result)
    }

    pub async fn get_user_info(
        &mut self,
        identities: &[UserIdentity],
    ) -> Result<Vec<UserInformation>, reqwest::Error> {
        let mut result = vec![];
        let mut idx = 0;
        while idx * 100 < identities.len() {
            result.extend(
                self.get_request(
                    "https://api.twitch.tv/helix/users",
                    &identities[idx * 100..identities.len().min(idx * 100 + 100)]
                        .iter()
                        .map(|id| match id {
                            UserIdentity::Id(s) => ("id", s),
                            UserIdentity::Login(s) => ("login", s),
                        })
                        .collect::<Vec<(&str, &String)>>(),
                )
                .await?
                .json::<Responses<UserInformation>>()
                .await?
                .data
                .into_iter(),
            );
            idx += 1;
        }
        Ok(result)
    }
}

pub fn process_thumbnail_url(url: &str, width: usize, height: usize) -> String {
    url.replace("{width}", &format!("{}", width))
        .replace("{height}", &format!("{}", height))
}
