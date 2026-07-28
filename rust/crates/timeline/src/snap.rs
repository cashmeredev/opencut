use std::collections::HashSet;

use scene::SceneTracks;
use time::MediaTime;

pub const BASE_TIMELINE_PIXELS_PER_SECOND: f64 = 50.0;
pub const TIMELINE_ZOOM_MIN: f64 = 0.1;
pub const TIMELINE_ZOOM_MAX: f64 = 100.0;
pub const DEFAULT_TIMELINE_SNAP_THRESHOLD_PX: f64 = 10.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SnapPointType {
    ElementStart,
    ElementEnd,
    Playhead,
    Bookmark,
    Keyframe,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SnapPoint {
    pub time: MediaTime,
    pub kind: SnapPointType,
    pub element_id: Option<String>,
    pub track_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SnapResult {
    pub snapped_time: MediaTime,
    pub snap_point: Option<SnapPoint>,
    pub snap_distance: f64,
}

pub fn resolve_timeline_snap(
    target_time: MediaTime,
    snap_points: &[SnapPoint],
    max_snap_distance: f64,
) -> SnapResult {
    let mut closest_snap_point: Option<&SnapPoint> = None;
    let mut closest_distance = f64::INFINITY;

    for snap_point in snap_points {
        let distance = (target_time - snap_point.time).as_ticks().unsigned_abs() as f64;
        if distance <= max_snap_distance && distance < closest_distance {
            closest_distance = distance;
            closest_snap_point = Some(snap_point);
        }
    }

    SnapResult {
        snapped_time: closest_snap_point
            .map(|snap_point| snap_point.time)
            .unwrap_or(target_time),
        snap_point: closest_snap_point.cloned(),
        snap_distance: closest_distance,
    }
}

pub fn get_timeline_snap_threshold_in_ticks(
    zoom_level: f64,
    snap_threshold_px: Option<f64>,
) -> f64 {
    let threshold_px = snap_threshold_px.unwrap_or(DEFAULT_TIMELINE_SNAP_THRESHOLD_PX);
    let pixels_per_second = BASE_TIMELINE_PIXELS_PER_SECOND * zoom_level;
    (threshold_px / pixels_per_second) * time::TICKS_PER_SECOND as f64
}

pub fn build_timeline_snap_points(
    sources: impl IntoIterator<Item = Vec<SnapPoint>>,
) -> Vec<SnapPoint> {
    sources.into_iter().flatten().collect()
}

pub fn get_element_edge_snap_points(
    tracks: &SceneTracks,
    exclude_element_ids: Option<&HashSet<&str>>,
) -> Vec<SnapPoint> {
    let mut snap_points = Vec::new();

    for track in crate::placement::ordered_tracks(tracks) {
        for element in track.elements() {
            let base = element.base();
            if exclude_element_ids.is_some_and(|ids| ids.contains(base.id.as_str())) {
                continue;
            }

            snap_points.push(SnapPoint {
                time: base.start_time,
                kind: SnapPointType::ElementStart,
                element_id: Some(base.id.clone()),
                track_id: Some(track.id().to_string()),
            });
            snap_points.push(SnapPoint {
                time: base.start_time + base.duration,
                kind: SnapPointType::ElementEnd,
                element_id: Some(base.id.clone()),
                track_id: Some(track.id().to_string()),
            });
        }
    }

    snap_points
}

pub fn get_playhead_snap_points(playhead_time: MediaTime) -> Vec<SnapPoint> {
    vec![SnapPoint {
        time: playhead_time,
        kind: SnapPointType::Playhead,
        element_id: None,
        track_id: None,
    }]
}
