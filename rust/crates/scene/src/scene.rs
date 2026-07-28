use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use time::MediaTime;

use crate::iso_date;
use crate::track::SceneTracks;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Bookmark {
    pub time: MediaTime,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<MediaTime>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Scene {
    pub id: String,
    pub name: String,
    pub is_main: bool,
    pub tracks: SceneTracks,
    #[serde(default)]
    pub bookmarks: Vec<Bookmark>,
    #[serde(with = "iso_date")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "iso_date")]
    pub updated_at: DateTime<Utc>,
}
