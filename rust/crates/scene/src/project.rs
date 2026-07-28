use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use time::{FrameRate, MediaTime};

use crate::iso_date;
use crate::scene::Scene;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum Background {
    Color { color: String },
    Blur { blur_intensity: f64 },
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CanvasSize {
    pub width: u32,
    pub height: u32,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum CanvasSizeMode {
    Preset,
    Custom,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectMetadata {
    pub id: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
    pub duration: MediaTime,
    #[serde(with = "iso_date")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "iso_date")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSettings {
    pub fps: FrameRate,
    pub canvas_size: CanvasSize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canvas_size_mode: Option<CanvasSizeMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_custom_canvas_size: Option<CanvasSize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_canvas_size: Option<CanvasSize>,
    pub background: Background,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TimelineViewState {
    pub zoom_level: f64,
    pub scroll_left: f64,
    pub playhead_time: MediaTime,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub metadata: ProjectMetadata,
    #[serde(default)]
    pub scenes: Vec<Scene>,
    pub current_scene_id: String,
    pub settings: ProjectSettings,
    pub version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeline_view_state: Option<TimelineViewState>,
}
