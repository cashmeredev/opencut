use time::{MediaTime, TICKS_PER_SECOND};

pub const DEFAULT_NEW_ELEMENT_DURATION: MediaTime =
    MediaTime::from_ticks(5 * TICKS_PER_SECOND);

pub fn to_element_duration_ticks(seconds: Option<f64>) -> Option<MediaTime> {
    match seconds {
        None => Some(DEFAULT_NEW_ELEMENT_DURATION),
        Some(seconds) => MediaTime::from_seconds_f64(seconds),
    }
}
