use scene::RetimeConfig;
use timeline::retime::{
    clamp_retime_rate, get_clip_time_at_source_time, get_effective_rate_at,
    get_source_span_at_clip_time, get_source_time_at_clip_time,
    get_timeline_duration_for_source_span, split_retime_at_clip_time,
};

const TWO_X: RetimeConfig = RetimeConfig {
    rate: 2.0,
    maintain_pitch: None,
};
const HALF_X: RetimeConfig = RetimeConfig {
    rate: 0.5,
    maintain_pitch: None,
};

#[test]
fn maps_clip_time_to_source_time_at_2x_speed() {
    assert_eq!(get_source_time_at_clip_time(5.0, Some(&TWO_X)), 10.0);
}

#[test]
fn maps_clip_time_to_source_time_at_half_speed() {
    assert_eq!(get_source_time_at_clip_time(4.0, Some(&HALF_X)), 2.0);
}

#[test]
fn returns_clip_time_unchanged_when_no_retime() {
    assert_eq!(get_source_time_at_clip_time(7.0, None), 7.0);
}

#[test]
fn inverts_source_time_back_to_clip_time_at_2x_speed() {
    assert_eq!(get_clip_time_at_source_time(10.0, Some(&TWO_X)), 5.0);
}

#[test]
fn returns_effective_rate() {
    assert_eq!(get_effective_rate_at(Some(&TWO_X)), 2.0);
    assert_eq!(get_effective_rate_at(None), 1.0);
}

#[test]
fn derives_timeline_duration_for_a_visible_source_span() {
    assert_eq!(
        get_timeline_duration_for_source_span(10.0, Some(&TWO_X)),
        5.0
    );
    assert_eq!(
        get_timeline_duration_for_source_span(10.0, Some(&HALF_X)),
        20.0
    );
}

#[test]
fn clamps_invalid_rates_to_one() {
    let zero = RetimeConfig {
        rate: 0.0,
        maintain_pitch: None,
    };
    let negative = RetimeConfig {
        rate: -1.0,
        maintain_pitch: None,
    };
    assert_eq!(get_source_time_at_clip_time(5.0, Some(&zero)), 5.0);
    assert_eq!(get_source_time_at_clip_time(5.0, Some(&negative)), 5.0);
    assert_eq!(clamp_retime_rate(f64::NAN), 1.0);
    assert_eq!(clamp_retime_rate(f64::INFINITY), 1.0);
}

#[test]
fn caps_retime_rates_above_5x() {
    let hundred = RetimeConfig {
        rate: 100.0,
        maintain_pitch: None,
    };
    assert_eq!(get_source_time_at_clip_time(5.0, Some(&hundred)), 25.0);
    assert_eq!(
        get_timeline_duration_for_source_span(10.0, Some(&hundred)),
        2.0
    );
}

#[test]
fn measures_source_span_at_a_clip_time() {
    assert_eq!(get_source_span_at_clip_time(5.0, Some(&TWO_X)), 10.0);
}

#[test]
fn returns_zero_for_non_positive_clip_time() {
    assert_eq!(get_source_span_at_clip_time(0.0, None), 0.0);
    assert_eq!(get_source_span_at_clip_time(-1.0, None), 0.0);
}

#[test]
fn passes_the_same_retime_to_both_halves_when_splitting() {
    let retime = RetimeConfig {
        rate: 1.5,
        maintain_pitch: None,
    };
    let (left, right) = split_retime_at_clip_time(Some(&retime));
    assert_eq!(left, Some(retime));
    assert_eq!(right, Some(retime));
}

#[test]
fn returns_none_on_both_sides_when_no_retime() {
    let (left, right) = split_retime_at_clip_time(None);
    assert_eq!(left, None);
    assert_eq!(right, None);
}
