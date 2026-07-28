use scene::params::{ParamValue, ParamValues};
use text::{
    Color, FontStyle, FontWeight, TextAlign, TextBackground, TextDecoration, TextEngine,
    TextLayoutParams, align_offset, decoration_rect, measure_text_block, resolve_text_layout,
    text_background_from, text_background_rect, text_layout_params_from, text_rect,
    text_visual_rect,
};

const FONT_REGULAR: &[u8] = include_bytes!("fixtures/DejaVuSans.ttf");
const FONT_BOLD: &[u8] = include_bytes!("fixtures/DejaVuSans-Bold.ttf");

fn engine() -> TextEngine {
    TextEngine::with_font_bytes(&[FONT_REGULAR, FONT_BOLD])
}

fn dejavu_params(content: &str) -> TextLayoutParams {
    TextLayoutParams {
        content: content.to_string(),
        font_family: "DejaVu Sans".to_string(),
        ..TextLayoutParams::default()
    }
}

#[test]
fn params_fall_back_to_web_defaults() {
    let params = ParamValues::new();
    let layout = text_layout_params_from(&params);
    assert_eq!(layout, TextLayoutParams::default());
    assert_eq!(layout.content, "Default text");
    assert_eq!(layout.font_size, 15.0);
    assert_eq!(layout.font_family, "Arial");
    assert_eq!(layout.text_align, TextAlign::Center);
    assert_eq!(layout.text_decoration, TextDecoration::None);
    assert_eq!(layout.letter_spacing, 0.0);
    assert_eq!(layout.line_height, 1.2);

    let background = text_background_from(&params);
    assert_eq!(background, TextBackground::default());
    assert!(!background.enabled);
    assert_eq!(background.color, "#000000");
    assert_eq!(background.padding_x, 30.0);
    assert_eq!(background.padding_y, 42.0);
}

#[test]
fn params_read_typed_values_and_reject_wrong_types() {
    let mut params = ParamValues::new();
    params.insert("content".into(), ParamValue::String("Hello".into()));
    params.insert("fontSize".into(), ParamValue::Number(24.0));
    params.insert("fontFamily".into(), ParamValue::String("Inter".into()));
    params.insert("fontWeight".into(), ParamValue::String("bold".into()));
    params.insert("fontStyle".into(), ParamValue::String("italic".into()));
    params.insert("textAlign".into(), ParamValue::String("right".into()));
    params.insert(
        "textDecoration".into(),
        ParamValue::String("line-through".into()),
    );
    params.insert("letterSpacing".into(), ParamValue::Number(2.5));
    params.insert("lineHeight".into(), ParamValue::Number(1.5));
    params.insert("background.enabled".into(), ParamValue::Bool(true));
    params.insert(
        "background.color".into(),
        ParamValue::String("#ff0000".into()),
    );
    params.insert("background.cornerRadius".into(), ParamValue::Number(50.0));
    params.insert("background.paddingX".into(), ParamValue::Number(8.0));
    params.insert("background.paddingY".into(), ParamValue::Number(6.0));
    params.insert("background.offsetX".into(), ParamValue::Number(-4.0));
    params.insert("background.offsetY".into(), ParamValue::Number(3.0));
    params.insert("opacity".into(), ParamValue::String("not-a-number".into()));

    let layout = text_layout_params_from(&params);
    assert_eq!(layout.content, "Hello");
    assert_eq!(layout.font_size, 24.0);
    assert_eq!(layout.font_family, "Inter");
    assert_eq!(layout.font_weight, FontWeight::Bold);
    assert_eq!(layout.font_style, FontStyle::Italic);
    assert_eq!(layout.text_align, TextAlign::Right);
    assert_eq!(layout.text_decoration, TextDecoration::LineThrough);
    assert_eq!(layout.letter_spacing, 2.5);
    assert_eq!(layout.line_height, 1.5);

    let background = text_background_from(&params);
    assert!(background.enabled);
    assert_eq!(background.color, "#ff0000");
    assert_eq!(background.corner_radius, 50.0);
    assert_eq!(background.padding_x, 8.0);
    assert_eq!(background.padding_y, 6.0);
    assert_eq!(background.offset_x, -4.0);
    assert_eq!(background.offset_y, 3.0);
}

#[test]
fn params_ignore_mismatched_value_types() {
    let mut params = ParamValues::new();
    params.insert("content".into(), ParamValue::Number(42.0));
    params.insert("fontSize".into(), ParamValue::String("big".into()));
    params.insert("fontWeight".into(), ParamValue::String("heavy".into()));
    params.insert("textAlign".into(), ParamValue::String("justify".into()));
    params.insert("background.enabled".into(), ParamValue::String("yes".into()));

    let layout = text_layout_params_from(&params);
    assert_eq!(layout.content, "Default text");
    assert_eq!(layout.font_size, 15.0);
    assert_eq!(layout.font_weight, FontWeight::Normal);
    assert_eq!(layout.text_align, TextAlign::Center);
    assert!(!text_background_from(&params).enabled);
}

#[test]
fn resolve_scales_font_size_with_canvas_height() {
    let params = dejavu_params("Hello");
    let resolved = resolve_text_layout(&params, 90.0);
    assert_eq!(resolved.scaled_font_size, 15.0);
    assert_eq!(resolved.line_height_px, 18.0);
    assert_eq!(resolved.font_size_ratio, 1.0);

    let doubled = resolve_text_layout(&params, 180.0);
    assert_eq!(doubled.scaled_font_size, 30.0);
    assert_eq!(doubled.line_height_px, 36.0);
}

#[test]
fn measure_splits_explicit_lines_and_sizes_block() {
    let mut engine = engine();
    let params = dejavu_params("ab\ncdef\ng");
    let measured = engine.measure(&params, 90.0);

    assert_eq!(measured.lines, vec!["ab", "cdef", "g"]);
    assert_eq!(measured.line_metrics.len(), 3);
    assert_eq!(measured.block.height, 54.0);
    assert_eq!(measured.block.visual_center_offset, 18.0);

    let widest = measured.line_metrics[1].width;
    assert_eq!(measured.block.max_width, widest);
    assert!(widest > measured.line_metrics[0].width);
    assert!(measured.line_metrics[0].width > measured.line_metrics[2].width);
    assert!(measured.line_metrics[2].width > 0.0);
}

#[test]
fn measure_trailing_empty_line_keeps_block_height() {
    let mut engine = engine();
    let params = dejavu_params("ab\n");
    let measured = engine.measure(&params, 90.0);

    assert_eq!(measured.lines.len(), 2);
    assert_eq!(measured.block.height, 36.0);
    assert_eq!(measured.line_metrics[1].width, 0.0);
    assert_eq!(
        measured.line_metrics[1].ascent,
        measured.resolved.scaled_font_size * 0.8
    );
    assert_eq!(
        measured.line_metrics[1].descent,
        measured.resolved.scaled_font_size * 0.2
    );
}

#[test]
fn wrap_width_controls_visual_line_count() {
    let mut engine = engine();
    let long_word = "a".repeat(30);
    let params = dejavu_params(&long_word);

    let unwrapped = engine.measure(&params, 90.0);
    assert_eq!(unwrapped.lines.len(), 1);

    let ten_a = dejavu_params(&"a".repeat(10));
    let ten_width = engine.measure(&ten_a, 90.0).line_metrics[0].width;
    let wrapped = engine.measure_with_wrap(&params, 90.0, Some(ten_width + 1.0));
    assert_eq!(wrapped.lines.len(), 3);
    assert_eq!(wrapped.block.height, 3.0 * wrapped.resolved.line_height_px);
    for metrics in &wrapped.line_metrics {
        assert!(metrics.width <= ten_width + 1.0);
    }
}

#[test]
fn line_height_scales_block_height() {
    let mut engine = engine();
    let single = dejavu_params("ab\ncd");
    let relaxed = TextLayoutParams {
        line_height: 2.4,
        ..single.clone()
    };

    let tight_layout = engine.measure(&single, 90.0);
    let relaxed_layout = engine.measure(&relaxed, 90.0);
    assert_eq!(relaxed_layout.block.height, tight_layout.block.height * 2.0);
    assert_eq!(
        relaxed_layout.block.visual_center_offset,
        tight_layout.block.visual_center_offset * 2.0
    );
}

#[test]
fn letter_spacing_widens_lines_monotonically() {
    let mut engine = engine();
    let content = "a".repeat(10);
    let width_at = |engine: &mut TextEngine, spacing: f64| {
        let params = TextLayoutParams {
            content: content.clone(),
            font_family: "DejaVu Sans".to_string(),
            letter_spacing: spacing,
            ..TextLayoutParams::default()
        };
        engine.measure(&params, 90.0).line_metrics[0].width
    };

    let zero = width_at(&mut engine, 0.0);
    let five = width_at(&mut engine, 5.0);
    let ten = width_at(&mut engine, 10.0);
    assert!(five > zero);
    assert!(ten > five);
    assert!((ten - zero - 90.0).abs() < 0.5);
}

#[test]
fn block_measurement_matches_web_formulas() {
    let metrics = [
        text::LineMetrics {
            width: 40.0,
            ascent: 12.0,
            descent: 3.0,
        },
        text::LineMetrics {
            width: 75.0,
            ascent: 12.0,
            descent: 3.0,
        },
    ];
    let block = measure_text_block(&metrics, 20.0);
    assert_eq!(block.max_width, 75.0);
    assert_eq!(block.height, 40.0);
    assert_eq!(block.visual_center_offset, 10.0);

    assert_eq!(align_offset(TextAlign::Left, 50.0), 0.0);
    assert_eq!(align_offset(TextAlign::Center, 50.0), -25.0);
    assert_eq!(align_offset(TextAlign::Right, 50.0), -50.0);
}

#[test]
fn background_rect_matches_hand_computed_rect() {
    let mut engine = engine();
    let params = TextLayoutParams {
        text_align: TextAlign::Left,
        ..dejavu_params("abcd")
    };
    let measured = engine.measure(&params, 90.0);
    let block = measured.block;
    let width = block.max_width;

    let background = TextBackground {
        enabled: true,
        color: "#ff0000".to_string(),
        corner_radius: 0.0,
        padding_x: 10.0,
        padding_y: 5.0,
        offset_x: 3.0,
        offset_y: -2.0,
    };

    let rect = text_background_rect(TextAlign::Left, &block, &background, 1.0)
        .expect("visible background");
    assert_eq!(rect.left, -7.0);
    assert_eq!(rect.top, -16.0);
    assert_eq!(rect.width, width + 20.0);
    assert_eq!(rect.height, 28.0);

    let centered = text_background_rect(TextAlign::Center, &block, &background, 1.0)
        .expect("visible background");
    assert_eq!(centered.left, -width / 2.0 - 7.0);
    assert_eq!(centered.width, width + 20.0);

    let scaled = text_background_rect(TextAlign::Left, &block, &background, 2.0)
        .expect("visible background");
    assert_eq!(scaled.left, -17.0);
    assert_eq!(scaled.width, width + 40.0);
    assert_eq!(scaled.height, 38.0);
}

#[test]
fn visual_rect_unions_text_and_background() {
    let mut engine = engine();
    let params = dejavu_params("abcd");
    let measured = engine.measure(&params, 90.0);
    let block = measured.block;

    let disabled = TextBackground::default();
    let visual = text_visual_rect(TextAlign::Center, &block, &disabled, 1.0);
    assert_eq!(visual, text_rect(TextAlign::Center, &block));

    let background = TextBackground {
        enabled: true,
        padding_x: 10.0,
        padding_y: 5.0,
        ..TextBackground::default()
    };
    let visual = text_visual_rect(TextAlign::Center, &block, &background, 1.0);
    assert_eq!(visual.width, block.max_width + 20.0);
    assert_eq!(visual.height, block.height + 10.0);
    assert_eq!(visual.left, -block.max_width / 2.0 - 10.0);
    assert_eq!(visual.top, -block.height / 2.0 - 5.0);

    let invisible = TextBackground {
        enabled: true,
        color: "transparent".to_string(),
        ..TextBackground::default()
    };
    assert_eq!(
        text_background_rect(TextAlign::Center, &block, &invisible, 1.0),
        None
    );
}

#[test]
fn decoration_rect_matches_web_geometry() {
    let underline = decoration_rect(
        TextDecoration::Underline,
        TextAlign::Left,
        100.0,
        0.0,
        12.0,
        3.0,
        20.0,
    )
    .expect("underline");
    assert_eq!(underline.left, 0.0);
    assert_eq!(underline.width, 100.0);
    assert!((underline.height - 1.4).abs() < 1e-9);
    assert!((underline.top - 4.4).abs() < 1e-9);

    let strike = decoration_rect(
        TextDecoration::LineThrough,
        TextAlign::Center,
        100.0,
        0.0,
        12.0,
        3.0,
        20.0,
    )
    .expect("line-through");
    assert_eq!(strike.left, -50.0);
    assert!((strike.top - -3.15).abs() < 1e-9);
    assert!((strike.height - 1.4).abs() < 1e-9);

    let tiny = decoration_rect(TextDecoration::Underline, TextAlign::Right, 50.0, 0.0, 8.0, 2.0, 5.0)
        .expect("underline");
    assert_eq!(tiny.height, 1.0);
    assert_eq!(tiny.left, -50.0);

    assert_eq!(
        decoration_rect(TextDecoration::None, TextAlign::Left, 100.0, 0.0, 12.0, 3.0, 20.0),
        None
    );
}

#[test]
fn color_parses_web_hex_formats() {
    assert_eq!(Color::parse("#ff0000"), Some(Color::rgb(255, 0, 0)));
    assert_eq!(Color::parse("#f00"), Some(Color::rgb(255, 0, 0)));
    assert_eq!(
        Color::parse("#ff000080"),
        Some(Color::rgba(255, 0, 0, 128))
    );
    assert_eq!(Color::parse("transparent"), Some(Color::TRANSPARENT));
    assert_eq!(Color::parse("red"), None);
    assert_eq!(Color::parse(""), None);
}
