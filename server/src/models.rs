use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Entity {
    Song,
    Album,
    Author,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RangeFilter {
    pub min: u32,
    pub max: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Filters {
    pub song_id: Option<RangeFilter>,
    pub album_id: Option<RangeFilter>,
    pub author: Option<String>,
    pub album_name: Option<String>,
    pub song_name: Option<String>,
    pub year: Option<RangeFilter>,
    pub duration_max: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SortField {
    Year,
    SongName,
    AlbumName,
    Author,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SortOrder {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Sort {
    pub field: Option<SortField>,
    pub order: Option<SortOrder>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiRequest {
    pub entity: Entity,
    pub filters: Filters,
    pub sort: Sort,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SongRecord {
    pub song_id: u32,
    pub author: String,
    pub album_name: String,
    pub song_name: String,
    pub year: u32,
    pub duration_sec: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AlbumRecord {
    pub album_id: u32,
    pub author: String,
    pub album_name: String,
    pub description: String,
    pub cover_url: String,
    pub year: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthorPayload {
    pub author: String,
    pub albums: Vec<AlbumRecord>,
    pub images: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "entity", content = "items", rename_all = "snake_case")]
pub enum ResponseData {
    Song(Vec<SongRecord>),
    Album(Vec<AlbumRecord>),
    Author(AuthorPayload),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiResponse {
    pub entity: Option<Entity>,
    pub filters: Filters,
    pub sort: Sort,
    pub error: u8,
    pub error_message: Option<String>,
    pub data: Option<ResponseData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: &'static str,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RequestLogEntry {
    pub timestamp_utc: String,
    pub ip: String,
    pub entity: Entity,
    pub error: u8,
    pub error_message: Option<String>,
}
