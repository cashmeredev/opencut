use time::{MediaTime, TICKS_PER_SECOND};

use crate::snap::BASE_TIMELINE_PIXELS_PER_SECOND;

pub fn get_mouse_time_from_client_x(
    client_x: f64,
    container_left: f64,
    zoom_level: f64,
    scroll_left: f64,
) -> MediaTime {
    let mouse_x = client_x - container_left + scroll_left;
    let seconds = (mouse_x / (BASE_TIMELINE_PIXELS_PER_SECOND * zoom_level)).max(0.0);
    MediaTime::from_ticks((seconds * TICKS_PER_SECOND as f64).round() as i64)
}
