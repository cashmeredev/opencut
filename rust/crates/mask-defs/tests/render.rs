use mask_defs::{feather_for_gpu, is_active, render_mask_body, render_mask_stroke};
use scene::{
    FreeformPathMaskParams, FreeformPathPoint, Mask, RectangleMaskParams, SplitMaskParams,
    StrokeAlign, TextDecoration, TextFontStyle, TextFontWeight, TextMaskParams,
};
use tiny_skia::Pixmap;

fn box_params() -> RectangleMaskParams {
    RectangleMaskParams {
        feather: 0.0,
        inverted: false,
        stroke_color: "#ffffff".to_string(),
        stroke_width: 0.0,
        stroke_align: StrokeAlign::Center,
        center_x: 0.0,
        center_y: 0.0,
        width: 0.6,
        height: 0.6,
        rotation: 0.0,
        scale: 1.0,
    }
}

fn split_params() -> SplitMaskParams {
    SplitMaskParams {
        feather: 0.0,
        inverted: false,
        stroke_color: "#ffffff".to_string(),
        stroke_width: 0.0,
        stroke_align: StrokeAlign::Center,
        center_x: 0.0,
        center_y: 0.0,
        rotation: 0.0,
    }
}

fn text_params(content: &str) -> TextMaskParams {
    TextMaskParams {
        feather: 0.0,
        inverted: false,
        stroke_color: "#ffffff".to_string(),
        stroke_width: 0.0,
        stroke_align: StrokeAlign::Center,
        content: content.to_string(),
        font_size: 15.0,
        font_family: "Arial".to_string(),
        font_weight: TextFontWeight::Normal,
        font_style: TextFontStyle::Normal,
        text_decoration: TextDecoration::None,
        letter_spacing: 0.0,
        line_height: 1.2,
        center_x: 0.0,
        center_y: 0.0,
        rotation: 0.0,
        scale: 1.0,
    }
}

fn freeform_params(points: Vec<FreeformPathPoint>, closed: bool) -> FreeformPathMaskParams {
    FreeformPathMaskParams {
        feather: 0.0,
        inverted: false,
        stroke_color: "#ffffff".to_string(),
        stroke_width: 0.0,
        stroke_align: StrokeAlign::Center,
        path: points,
        closed,
        center_x: 0.0,
        center_y: 0.0,
        rotation: 0.0,
        scale: 1.0,
    }
}

fn point(id: &str, x: f64, y: f64) -> FreeformPathPoint {
    FreeformPathPoint {
        id: id.to_string(),
        x,
        y,
        in_x: 0.0,
        in_y: 0.0,
        out_x: 0.0,
        out_y: 0.0,
    }
}

fn alpha_at(pixmap: &Pixmap, x: u32, y: u32) -> u8 {
    pixmap.pixel(x, y).map(|p| p.alpha()).unwrap_or(0)
}

fn assert_opaque(pixmap: &Pixmap, x: u32, y: u32) {
    assert!(
        alpha_at(pixmap, x, y) > 250,
        "expected opaque at ({x}, {y}), got {}",
        alpha_at(pixmap, x, y)
    );
}

fn assert_transparent(pixmap: &Pixmap, x: u32, y: u32) {
    assert!(
        alpha_at(pixmap, x, y) < 5,
        "expected transparent at ({x}, {y}), got {}",
        alpha_at(pixmap, x, y)
    );
}

fn box_mask(kind: &str, params: RectangleMaskParams) -> Mask {
    match kind {
        "cinematic-bars" => Mask::CinematicBars { id: "m".into(), params },
        "rectangle" => Mask::Rectangle { id: "m".into(), params },
        "ellipse" => Mask::Ellipse { id: "m".into(), params },
        "heart" => Mask::Heart { id: "m".into(), params },
        "diamond" => Mask::Diamond { id: "m".into(), params },
        _ => Mask::Star { id: "m".into(), params },
    }
}

#[test]
fn rectangle_body_center_opaque_corners_transparent() {
    let mask = box_mask("rectangle", box_params());
    let pixmap = render_mask_body(&mask, 200, 200).unwrap();
    assert_opaque(&pixmap, 100, 100);
    assert_opaque(&pixmap, 50, 50);
    assert_transparent(&pixmap, 20, 20);
    assert_transparent(&pixmap, 180, 180);
}

#[test]
fn rectangle_body_rotated_90_swaps_extents() {
    let mut params = box_params();
    params.width = 0.6;
    params.height = 0.3;
    params.rotation = 90.0;
    let mask = box_mask("rectangle", params);
    let pixmap = render_mask_body(&mask, 200, 100).unwrap();
    assert_opaque(&pixmap, 100, 50);
    assert_opaque(&pixmap, 100, 10);
    assert_transparent(&pixmap, 150, 50);
}

#[test]
fn rectangle_stroke_ring_present_when_stroke_width_positive() {
    let mut params = box_params();
    params.stroke_width = 10.0;
    let mask = box_mask("rectangle", params);
    let stroke = render_mask_stroke(&mask, 200, 200).unwrap().unwrap();
    assert_opaque(&stroke, 100, 40);
    assert_transparent(&stroke, 100, 100);
    assert_transparent(&stroke, 10, 10);
}

#[test]
fn stroke_is_none_when_stroke_width_zero() {
    let mask = box_mask("rectangle", box_params());
    assert!(render_mask_stroke(&mask, 200, 200).unwrap().is_none());
}

#[test]
fn ellipse_body_center_opaque_corners_transparent() {
    let mask = box_mask("ellipse", box_params());
    let pixmap = render_mask_body(&mask, 200, 200).unwrap();
    assert_opaque(&pixmap, 100, 100);
    assert_opaque(&pixmap, 130, 100);
    assert_transparent(&pixmap, 45, 45);
    assert_transparent(&pixmap, 10, 10);
}

#[test]
fn heart_body_region_coverage() {
    let mask = box_mask("heart", box_params());
    let pixmap = render_mask_body(&mask, 200, 200).unwrap();
    assert_opaque(&pixmap, 100, 110);
    assert_opaque(&pixmap, 75, 80);
    assert_transparent(&pixmap, 100, 60);
    assert_transparent(&pixmap, 10, 10);
}

#[test]
fn diamond_body_region_coverage() {
    let mask = box_mask("diamond", box_params());
    let pixmap = render_mask_body(&mask, 200, 200).unwrap();
    assert_opaque(&pixmap, 100, 100);
    assert_opaque(&pixmap, 130, 100);
    assert_transparent(&pixmap, 145, 145);
    assert_transparent(&pixmap, 10, 10);
}

#[test]
fn star_body_region_coverage() {
    let mask = box_mask("star", box_params());
    let pixmap = render_mask_body(&mask, 200, 200).unwrap();
    assert_opaque(&pixmap, 100, 100);
    assert_opaque(&pixmap, 100, 52);
    assert_transparent(&pixmap, 121, 71);
    assert_transparent(&pixmap, 10, 10);
}

#[test]
fn cinematic_bars_body_full_width_band() {
    let mut params = box_params();
    params.width = 1.5;
    params.height = 0.6;
    let mask = box_mask("cinematic-bars", params);
    let pixmap = render_mask_body(&mask, 200, 100).unwrap();
    assert_opaque(&pixmap, 100, 50);
    assert_opaque(&pixmap, 5, 50);
    assert_transparent(&pixmap, 100, 10);
    assert_transparent(&pixmap, 100, 95);
}

#[test]
fn split_body_zero_feather_hard_edge() {
    let mask = Mask::Split { id: "m".into(), params: split_params() };
    let pixmap = render_mask_body(&mask, 200, 100).unwrap();
    assert_opaque(&pixmap, 150, 50);
    assert_opaque(&pixmap, 101, 50);
    assert_transparent(&pixmap, 99, 50);
    assert_transparent(&pixmap, 50, 50);
}

#[test]
fn split_body_feather_gradient_band() {
    let mut params = split_params();
    params.feather = 40.0;
    let mask = Mask::Split { id: "m".into(), params };
    let pixmap = render_mask_body(&mask, 200, 100).unwrap();
    assert_opaque(&pixmap, 150, 50);
    assert_transparent(&pixmap, 50, 50);
    let mid = alpha_at(&pixmap, 100, 50);
    assert!(
        (108..=148).contains(&mid),
        "expected mid alpha ~128 at split line, got {mid}"
    );
    let quarter = alpha_at(&pixmap, 110, 50) as u16;
    assert!(
        (170..=215).contains(&quarter),
        "expected alpha ~191 at feather quarter, got {quarter}"
    );
}

#[test]
fn split_stroke_segment_vertical_and_horizontal() {
    let mut params = split_params();
    params.stroke_width = 4.0;
    let mask = Mask::Split { id: "m".into(), params };
    let stroke = render_mask_stroke(&mask, 200, 100).unwrap().unwrap();
    assert_opaque(&stroke, 100, 50);
    assert_transparent(&stroke, 106, 50);

    let mut rotated = split_params();
    rotated.rotation = 90.0;
    rotated.stroke_width = 4.0;
    let mask = Mask::Split { id: "m".into(), params: rotated };
    let stroke = render_mask_stroke(&mask, 200, 100).unwrap().unwrap();
    assert_opaque(&stroke, 50, 50);
    assert_transparent(&stroke, 50, 56);
}

#[test]
fn freeform_closed_triangle_region_coverage() {
    let points = vec![
        point("a", -0.4, -0.4),
        point("b", 0.4, -0.4),
        point("c", 0.0, 0.4),
    ];
    let mask = Mask::Freeform { id: "m".into(), params: freeform_params(points, true) };
    let pixmap = render_mask_body(&mask, 200, 200).unwrap();
    assert_opaque(&pixmap, 100, 110);
    assert_transparent(&pixmap, 10, 10);
    assert_transparent(&pixmap, 100, 180);
}

#[test]
fn freeform_open_path_is_inactive_and_empty() {
    let points = vec![
        point("a", -0.4, -0.4),
        point("b", 0.4, -0.4),
        point("c", 0.0, 0.4),
    ];
    let mask = Mask::Freeform { id: "m".into(), params: freeform_params(points, false) };
    assert!(!is_active(&mask));
    let pixmap = render_mask_body(&mask, 200, 200).unwrap();
    for pixel in pixmap.pixels() {
        assert_eq!(pixel.alpha(), 0);
    }
    let mut stroke_params = freeform_params(
        vec![point("a", -0.4, -0.4), point("b", 0.4, -0.4), point("c", 0.0, 0.4)],
        false,
    );
    stroke_params.stroke_width = 5.0;
    let mask = Mask::Freeform { id: "m".into(), params: stroke_params };
    let stroke = render_mask_stroke(&mask, 200, 200).unwrap().unwrap();
    for pixel in stroke.pixels() {
        assert_eq!(pixel.alpha(), 0);
    }
}

#[test]
fn text_body_rasterizes_glyphs() {
    let mask = Mask::Text { id: "m".into(), params: text_params("Hi") };
    let pixmap = render_mask_body(&mask, 200, 200).unwrap();
    let opaque_count = pixmap.pixels().iter().filter(|p| p.alpha() > 200).count();
    assert!(opaque_count > 20, "expected rasterized glyphs, got {opaque_count} opaque pixels");
    assert_transparent(&pixmap, 2, 2);
    assert_transparent(&pixmap, 197, 197);
}

#[test]
fn text_stroke_rasterizes_ring() {
    let mut params = text_params("Hi");
    params.stroke_width = 3.0;
    let mask = Mask::Text { id: "m".into(), params };
    let stroke = render_mask_stroke(&mask, 200, 200).unwrap().unwrap();
    let opaque_count = stroke.pixels().iter().filter(|p| p.alpha() > 200).count();
    assert!(opaque_count > 20, "expected stroked glyphs, got {opaque_count} opaque pixels");
}

#[test]
fn is_active_rules_match_web() {
    let text = Mask::Text { id: "m".into(), params: text_params("   ") };
    assert!(!is_active(&text));
    let text = Mask::Text { id: "m".into(), params: text_params("Mask") };
    assert!(is_active(&text));
    assert!(is_active(&box_mask("rectangle", box_params())));
    assert!(is_active(&Mask::Split { id: "m".into(), params: split_params() }));
}

#[test]
fn feather_for_gpu_zeroes_draw_with_feather_bodies() {
    let mut split = split_params();
    split.feather = 40.0;
    let mask = Mask::Split { id: "m".into(), params: split };
    assert_eq!(feather_for_gpu(&mask), 0.0);

    let mut rect = box_params();
    rect.feather = 40.0;
    let mask = box_mask("rectangle", rect);
    assert_eq!(feather_for_gpu(&mask), 40.0);

    let mut text = text_params("Hi");
    text.feather = 25.0;
    let mask = Mask::Text { id: "m".into(), params: text };
    assert_eq!(feather_for_gpu(&mask), 25.0);
}

#[test]
fn stroke_align_outside_keeps_outer_ring() {
    let mut params = box_params();
    params.stroke_width = 10.0;
    params.stroke_align = StrokeAlign::Outside;
    let mask = Mask::Freeform {
        id: "m".into(),
        params: FreeformPathMaskParams {
            stroke_width: 10.0,
            stroke_align: StrokeAlign::Outside,
            ..freeform_params(
                vec![
                    point("a", -0.3, -0.3),
                    point("b", 0.3, -0.3),
                    point("c", 0.3, 0.3),
                    point("d", -0.3, 0.3),
                ],
                true,
            )
        },
    };
    let _ = params;
    let stroke = render_mask_stroke(&mask, 200, 200).unwrap().unwrap();
    assert_opaque(&stroke, 100, 35);
    assert_transparent(&stroke, 100, 100);
    assert_transparent(&stroke, 100, 43);
}
