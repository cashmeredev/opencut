use scene::RetimeConfig;
use time::MediaTime;

pub const DEFAULT_RETIME_RATE: f64 = 1.0;
pub const MIN_RETIME_RATE: f64 = 0.01;
pub const MAX_RETIME_RATE: f64 = 5.0;

pub fn clamp_retime_rate(rate: f64) -> f64 {
    if !rate.is_finite() || rate <= 0.0 {
        return DEFAULT_RETIME_RATE;
    }

    rate.clamp(MIN_RETIME_RATE, MAX_RETIME_RATE)
}

pub fn can_maintain_pitch(rate: f64) -> bool {
    rate.is_finite() && rate > 0.0
}

pub fn should_maintain_pitch(rate: f64, maintain_pitch: Option<bool>) -> bool {
    maintain_pitch == Some(true) && can_maintain_pitch(rate)
}

fn safe_rate(retime: Option<&RetimeConfig>) -> f64 {
    clamp_retime_rate(retime.map(|retime| retime.rate).unwrap_or(1.0))
}

pub fn get_source_time_at_clip_time(clip_time: f64, retime: Option<&RetimeConfig>) -> f64 {
    clip_time * safe_rate(retime)
}

pub fn get_clip_time_at_source_time(source_time: f64, retime: Option<&RetimeConfig>) -> f64 {
    source_time / safe_rate(retime)
}

pub fn get_effective_rate_at(retime: Option<&RetimeConfig>) -> f64 {
    safe_rate(retime)
}

pub fn get_timeline_duration_for_source_span(
    source_span: f64,
    retime: Option<&RetimeConfig>,
) -> f64 {
    if source_span <= 0.0 {
        return 0.0;
    }
    source_span / safe_rate(retime)
}

pub fn get_source_span_at_clip_time(clip_time: f64, retime: Option<&RetimeConfig>) -> f64 {
    get_source_time_at_clip_time(clip_time, retime).max(0.0)
}

pub fn split_retime_at_clip_time(
    retime: Option<&RetimeConfig>,
) -> (Option<RetimeConfig>, Option<RetimeConfig>) {
    (retime.copied(), retime.copied())
}

pub fn adjust_retime_for_trim_change(retime: Option<&RetimeConfig>) -> Option<RetimeConfig> {
    retime.copied()
}

pub fn round_media_time(time: f64) -> MediaTime {
    let rounded_magnitude = time.abs().round();
    if rounded_magnitude == 0.0 {
        return MediaTime::ZERO;
    }
    let magnitude = rounded_magnitude as i64;
    MediaTime::from_ticks(if time < 0.0 { -magnitude } else { magnitude })
}
