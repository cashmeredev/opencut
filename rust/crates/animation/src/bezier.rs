use scene::{CurveHandle, ScalarAnimationKey};
use time::MediaTime;

pub const BEZIER_SOLVE_ITERATIONS: usize = 20;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BezierHandle {
    pub dt: f64,
    pub dv: f64,
}

impl From<CurveHandle> for BezierHandle {
    fn from(handle: CurveHandle) -> Self {
        Self {
            dt: handle.dt.as_ticks() as f64,
            dv: handle.dv,
        }
    }
}

pub fn bezier_point(progress: f64, p0: f64, p1: f64, p2: f64, p3: f64) -> f64 {
    let mt = 1.0 - progress;
    mt * mt * mt * p0
        + 3.0 * mt * mt * progress * p1
        + 3.0 * mt * progress * progress * p2
        + progress * progress * progress * p3
}

pub fn default_right_handle(
    left_key: &ScalarAnimationKey,
    right_key: &ScalarAnimationKey,
) -> BezierHandle {
    let span = (right_key.time - left_key.time).as_ticks() as f64;
    let value_delta = right_key.value - left_key.value;
    BezierHandle {
        dt: span / 3.0,
        dv: value_delta / 3.0,
    }
}

pub fn default_left_handle(
    left_key: &ScalarAnimationKey,
    right_key: &ScalarAnimationKey,
) -> BezierHandle {
    let span = (right_key.time - left_key.time).as_ticks() as f64;
    let value_delta = right_key.value - left_key.value;
    BezierHandle {
        dt: -span / 3.0,
        dv: -value_delta / 3.0,
    }
}

pub fn segment_handles(
    left_key: &ScalarAnimationKey,
    right_key: &ScalarAnimationKey,
) -> (BezierHandle, BezierHandle) {
    let right_handle = left_key
        .right_handle
        .map(BezierHandle::from)
        .unwrap_or_else(|| default_right_handle(left_key, right_key));
    let left_handle = right_key
        .left_handle
        .map(BezierHandle::from)
        .unwrap_or_else(|| default_left_handle(left_key, right_key));
    (right_handle, left_handle)
}

pub fn solve_bezier_progress_for_time(
    time: MediaTime,
    left_key: &ScalarAnimationKey,
    right_key: &ScalarAnimationKey,
) -> f64 {
    let (right_handle, left_handle) = segment_handles(left_key, right_key);
    let target = time.as_ticks() as f64;
    let p0 = left_key.time.as_ticks() as f64;
    let p3 = right_key.time.as_ticks() as f64;
    let mut lower = 0.0;
    let mut upper = 1.0;

    for _ in 0..BEZIER_SOLVE_ITERATIONS {
        let mid = (lower + upper) / 2.0;
        let estimate = bezier_point(mid, p0, p0 + right_handle.dt, p3 + left_handle.dt, p3);
        if estimate < target {
            lower = mid;
        } else {
            upper = mid;
        }
    }

    (lower + upper) / 2.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use scene::{ScalarSegmentType, TangentMode};

    fn key(time: i64, value: f64) -> ScalarAnimationKey {
        ScalarAnimationKey {
            id: format!("k{time}"),
            time: MediaTime::from_ticks(time),
            value,
            left_handle: None,
            right_handle: None,
            segment_to_next: ScalarSegmentType::Bezier,
            tangent_mode: TangentMode::Flat,
        }
    }

    #[test]
    fn bezier_point_evaluates_cubic() {
        assert_eq!(bezier_point(0.0, 0.0, 1.0, 2.0, 3.0), 0.0);
        assert_eq!(bezier_point(1.0, 0.0, 1.0, 2.0, 3.0), 3.0);
        assert_eq!(bezier_point(0.5, 0.0, 0.0, 1.0, 1.0), 0.5);
    }

    #[test]
    fn default_handles_split_span_into_thirds() {
        let left = key(0, 10.0);
        let right = key(90, 40.0);
        let right_handle = default_right_handle(&left, &right);
        let left_handle = default_left_handle(&left, &right);
        assert_eq!(right_handle.dt, 30.0);
        assert_eq!(right_handle.dv, 10.0);
        assert_eq!(left_handle.dt, -30.0);
        assert_eq!(left_handle.dv, -10.0);
    }

    #[test]
    fn solve_progress_is_linear_for_diagonal_control_points() {
        let left = key(0, 0.0);
        let right = key(100, 100.0);
        let progress = solve_bezier_progress_for_time(MediaTime::from_ticks(50), &left, &right);
        assert!((progress - 0.5).abs() < 1e-6);
        let progress = solve_bezier_progress_for_time(MediaTime::from_ticks(25), &left, &right);
        assert!((progress - 0.25).abs() < 1e-6);
    }

    #[test]
    fn solve_progress_respects_flat_tangents() {
        let mut left = key(0, 0.0);
        left.right_handle = Some(CurveHandle {
            dt: MediaTime::from_ticks(50),
            dv: 0.0,
        });
        let mut right = key(100, 100.0);
        right.left_handle = Some(CurveHandle {
            dt: MediaTime::from_ticks(-50),
            dv: 0.0,
        });
        let early = solve_bezier_progress_for_time(MediaTime::from_ticks(25), &left, &right);
        let mid = solve_bezier_progress_for_time(MediaTime::from_ticks(50), &left, &right);
        let late = solve_bezier_progress_for_time(MediaTime::from_ticks(75), &left, &right);
        assert!(early < 0.25);
        assert!((mid - 0.5).abs() < 1e-3);
        assert!(late > 0.75);
    }
}
