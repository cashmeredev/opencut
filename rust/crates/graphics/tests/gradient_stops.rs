use graphics::gradient::ast::{Color, ColorStop, Distance};
use graphics::gradient::stops::{color_to_string, normalize_color_stops};

fn stop(color: Color, length: Option<Distance>) -> ColorStop {
    ColorStop { color, length }
}

fn literal(name: &str) -> Color {
    Color::Literal(name.to_string())
}

#[test]
fn color_string_round_trip() {
    assert_eq!(color_to_string(&Color::Hex("abc".into())), "#abc");
    assert_eq!(color_to_string(&literal("red")), "red");
    assert_eq!(
        color_to_string(&Color::Rgb(vec!["1".into(), "2".into(), "3".into()])),
        "rgb(1,2,3)"
    );
    assert_eq!(
        color_to_string(&Color::Rgba(vec!["1".into(), "2".into(), "3".into(), "0.5".into()])),
        "rgba(1,2,3,0.5)"
    );
    assert_eq!(
        color_to_string(&Color::Hsl(["120".into(), "50".into(), "50".into()])),
        "hsl(120,50,50)"
    );
    assert_eq!(
        color_to_string(&Color::Hsla(["120".into(), "50".into(), "50".into(), "0.5".into()])),
        "hsla(120,50,50,0.5)"
    );
    assert_eq!(color_to_string(&Color::Var("--x".into())), "var(--x)");
}

#[test]
fn evenly_distributed_stops() {
    let stops = normalize_color_stops(
        &[stop(literal("red"), None), stop(literal("blue"), None)],
        100.0,
    );
    assert_eq!(stops[0].offset, 0.0);
    assert_eq!(stops[1].offset, 1.0);

    let stops = normalize_color_stops(
        &[
            stop(literal("a"), None),
            stop(literal("b"), None),
            stop(literal("c"), None),
        ],
        100.0,
    );
    assert_eq!(stops[1].offset, 0.5);
}

#[test]
fn explicit_and_mixed_offsets() {
    let stops = normalize_color_stops(
        &[
            stop(literal("red"), Some(Distance::Percent("20".into()))),
            stop(literal("green"), None),
            stop(literal("blue"), Some(Distance::Percent("80".into()))),
        ],
        100.0,
    );
    assert_eq!(stops[0].offset, 0.2);
    assert_eq!(stops[1].offset, 0.5);
    assert_eq!(stops[2].offset, 0.8);
}

#[test]
fn px_and_em_offsets_use_gradient_length() {
    let stops = normalize_color_stops(
        &[
            stop(literal("red"), Some(Distance::Px("50".into()))),
            stop(literal("blue"), Some(Distance::Em("6.25".into()))),
        ],
        200.0,
    );
    assert_eq!(stops[0].offset, 0.25);
    assert!((stops[1].offset - 0.5).abs() < 1e-9);
}

#[test]
fn offsets_clamped() {
    let stops = normalize_color_stops(
        &[
            stop(literal("red"), Some(Distance::Percent("-20".into()))),
            stop(literal("blue"), Some(Distance::Percent("150".into()))),
        ],
        100.0,
    );
    assert_eq!(stops[0].offset, 0.0);
    assert_eq!(stops[1].offset, 1.0);
}

#[test]
fn leading_unpositioned_stops() {
    let stops = normalize_color_stops(
        &[
            stop(literal("a"), None),
            stop(literal("b"), None),
            stop(literal("c"), Some(Distance::Percent("90".into()))),
        ],
        100.0,
    );
    assert_eq!(stops[0].offset, 0.0);
    assert_eq!(stops[1].offset, 0.45);
    assert_eq!(stops[2].offset, 0.9);
}

#[test]
fn trailing_unpositioned_stops() {
    let stops = normalize_color_stops(
        &[
            stop(literal("a"), Some(Distance::Percent("10".into()))),
            stop(literal("b"), None),
            stop(literal("c"), None),
        ],
        100.0,
    );
    assert_eq!(stops[0].offset, 0.1);
    assert!((stops[1].offset - 0.55).abs() < 1e-9);
    assert_eq!(stops[2].offset, 1.0);
}

#[test]
fn transparent_stop_adopts_previous_color() {
    let stops = normalize_color_stops(
        &[
            stop(Color::Hex("ff0000".into()), None),
            stop(literal("transparent"), None),
            stop(Color::Hex("0000ff".into()), None),
        ],
        100.0,
    );
    assert_eq!(stops[1].color, "rgba(255,0,0,0)");
}

#[test]
fn transparent_stop_with_unparseable_donors_unchanged() {
    let stops = normalize_color_stops(
        &[
            stop(literal("red"), None),
            stop(literal("transparent"), None),
            stop(literal("blue"), None),
        ],
        100.0,
    );
    assert_eq!(stops[1].color, "transparent");
}

#[test]
fn transparent_stop_adopts_next_color_when_first() {
    let stops = normalize_color_stops(
        &[
            stop(literal("transparent"), None),
            stop(Color::Hex("00ff00".into()), None),
        ],
        100.0,
    );
    assert_eq!(stops[1].color, "#00ff00");
    assert_eq!(stops[0].color, "rgba(0,255,0,0)");
}

#[test]
fn all_transparent_stops_unchanged() {
    let stops = normalize_color_stops(
        &[
            stop(literal("transparent"), None),
            stop(Color::Rgba(vec!["0".into(), "0".into(), "0".into(), "0".into()]), None),
        ],
        100.0,
    );
    assert_eq!(stops[0].color, "transparent");
    assert_eq!(stops[1].color, "rgba(0,0,0,0)");
}
