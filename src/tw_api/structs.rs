use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TokenResponse {
    pub access_token: String,
    pub expires_in: u64,
    pub token_type: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Pagination {
    pub cursor: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PagedResponses<T> {
    pub data: Vec<T>,
    pub pagination: Option<Pagination>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Responses<T> {
    pub data: Vec<T>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChannelSearchResult {
    pub broadcaster_language: String,
    pub broadcaster_login: String,
    pub display_name: String,
    pub game_id: String,
    pub game_name: String,
    pub id: String,
    pub is_live: bool,
    pub tag_ids: Vec<String>,
    pub tags: Vec<String>,
    pub thumbnail_url: String,
    pub title: String,
    pub started_at: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChannelInformation {
    pub broadcaster_id: String,
    pub broadcaster_login: String,
    pub broadcaster_name: String,
    pub broadcaster_language: String,
    pub game_id: String,
    pub game_name: String,
    pub title: String,
    pub delay: usize,
    pub tags: Vec<String>,
    pub content_classification_labels: Vec<String>,
    pub is_branded_content: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StreamInformation {
    pub id: String,
    pub user_id: String,
    pub user_login: String,
    pub user_name: String,
    pub game_id: String,
    pub game_name: String,
    #[serde(rename = "type")]
    pub stream_type: String,
    pub title: String,
    pub tags: Vec<String>,
    pub viewer_count: usize,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub language: String,
    pub thumbnail_url: String,
    pub tag_ids: Vec<String>,
    pub is_mature: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserInformation {
    pub id: String,
    pub login: String,
    pub display_name: String,
    #[serde(rename = "type")]
    pub user_type: String,
    pub broadcaster_type: String,
    pub description: String,
    pub profile_image_url: String,
    pub offline_image_url: String,
    pub view_count: usize,
    pub email: Option<String>,
    pub created_at: String,
}
