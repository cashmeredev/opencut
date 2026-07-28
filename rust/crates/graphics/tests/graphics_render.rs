use graphics::color::{hsl_to_rgb, is_transparent, parse_color_to_rgb, parse_css_color};
use graphics::{
    GraphicsError, ParamValue, ParamValues, build_default_graphic_instance, draw_css_background,
    get_graphic_definition, graphics_registry, parse_background_layers, render_graphic,
    resolve_graphic_params, split_css_layers, BackgroundLayer, GradientKind,
};
use tiny_skia::Pixmap;

fn pixel(pixmap: &Pixmap, x: u32, y: u32) -> (u8, u8, u8, u8) {
    let px = pixmap.pixel(x, y).unwrap();
    (px.red(), px.green(), px.blue(), px.alpha())
}

fn params(entries: &[(&str, ParamValue)]) -> ParamValues {
    entries
        .iter()
        .map(|(key, value)| (key.to_string(), value.clone()))
        .collect()
}

#[test]
fn registry_lookup() {
    let registry = graphics_registry();
    assert!(registry.has("rectangle"));
    assert!(registry.has("ellipse"));
    assert!(registry.has("polygon"));
    assert!(registry.has("star"));
    assert_eq!(registry.get_all().len(), 4);

    let rectangle = get_graphic_definition("rectangle").unwrap();
    assert_eq!(rectangle.name, "Rectangle");
    assert!(rectangle.keywords.contains(&"box"));

    let err = match get_graphic_definition("nope") {
        Ok(_) => panic!("expected unknown graphic error"),
        Err(err) => err,
    };
    assert_eq!(err, GraphicsError::UnknownGraphic("nope".to_string()));
    assert_eq!(err.to_string(), "Unknown graphic: nope");
}

#[test]
fn default_instance_and_resolve() {
    let instance = build_default_graphic_instance("polygon").unwrap();
    assert_eq!(instance.definition_id, "polygon");
    assert_eq!(instance.params.get("sides"), Some(&ParamValue::Number(5.0)));
    assert_eq!(
        instance.params.get("fill"),
        Some(&ParamValue::String("#ffffff".to_string()))
    );
    assert_eq!(
        instance.params.get("strokeAlign"),
        Some(&ParamValue::String("center".to_string()))
    );

    let definition = get_graphic_definition("star").unwrap();
    let resolved = resolve_graphic_params(
        definition,
        Some(&params(&[("depth", ParamValue::Number(30.0))])),
    );
    assert_eq!(resolved.get("depth"), Some(&ParamValue::Number(30.0)));
    assert_eq!(resolved.get("points"), Some(&ParamValue::Number(5.0)));

    let defaults = resolve_graphic_params(definition, None);
    assert_eq!(defaults.get("depth"), Some(&ParamValue::Number(45.0)));
}

#[test]
fn color_parsing() {
    let red = parse_css_color("#ff0000").unwrap();
    assert_eq!((red.r, red.g, red.b, red.a), (1.0, 0.0, 0.0, 1.0));
    let short = parse_css_color("#f00").unwrap();
    assert_eq!(short, red);
    let with_alpha = parse_css_color("#ff000080").unwrap();
    assert!((with_alpha.a - 128.0 / 255.0).abs() < 1e-9);
    let named = parse_css_color("rebeccapurple").unwrap();
    assert!((named.r - 102.0 / 255.0).abs() < 1e-9);
    assert_eq!(parse_css_color("transparent").unwrap().a, 0.0);
    let rgb = parse_css_color("rgb(255, 0, 0)").unwrap();
    assert_eq!(rgb, red);
    let rgb_percent = parse_css_color("rgb(100%, 0%, 0%)").unwrap();
    assert!((rgb_percent.r - 1.0).abs() < 1e-9);
    let rgba = parse_css_color("rgba(0, 255, 0, 0.5)").unwrap();
    assert_eq!((rgba.g, rgba.a), (1.0, 0.5));
    let hsl = parse_css_color("hsl(120, 100%, 50%)").unwrap();
    assert!((hsl.g - 1.0).abs() < 1e-9 && hsl.r.abs() < 1e-9);
    assert!(parse_css_color("not-a-color").is_none());

    let (r, g, b) = hsl_to_rgb(0.0, 1.0, 0.5);
    assert_eq!((r, g, b), (1.0, 0.0, 0.0));

    assert!(is_transparent("transparent"));
    assert!(is_transparent("rgba(1, 2, 3, 0)"));
    assert!(!is_transparent("rgba(1, 2, 3, 0.5)"));
    assert!(!is_transparent("#00000000"));

    let parsed = parse_color_to_rgb("#ff0000").unwrap();
    assert_eq!((parsed.r, parsed.g, parsed.b), (255.0, 0.0, 0.0));
    assert!(parse_color_to_rgb("#ff00").is_none());
    assert!(parse_color_to_rgb("hsl(0,100%,50%)").is_none());
}

#[test]
fn layer_splitting() {
    assert_eq!(
        split_css_layers("#00ff00, linear-gradient(red, blue), rgba(1,2,3,0.5)"),
        vec!["#00ff00", "linear-gradient(red, blue)", "rgba(1,2,3,0.5)"]
    );

    let layers = parse_background_layers("linear-gradient(red, blue)");
    assert!(matches!(
        &layers[0],
        BackgroundLayer::Gradient(ast) if ast.kind == GradientKind::Linear
    ));

    let layers = parse_background_layers("rebeccapurple");
    assert_eq!(layers, vec![BackgroundLayer::Color("rebeccapurple".to_string())]);
}

#[test]
fn rectangle_render_regions() {
    let pixmap = render_graphic("rectangle", None, 64, 64).unwrap();
    assert_eq!(pixmap.width(), 64);
    assert_eq!(pixmap.height(), 64);
    assert_eq!(pixel(&pixmap, 32, 32), (255, 255, 255, 255));
    assert_eq!(pixel(&pixmap, 0, 0), (255, 255, 255, 255));

    let rounded = render_graphic(
        "rectangle",
        Some(&params(&[("cornerRadius", ParamValue::Number(50.0))])),
        64,
        64,
    )
    .unwrap();
    assert_eq!(pixel(&rounded, 32, 32), (255, 255, 255, 255));
    assert_eq!(pixel(&rounded, 0, 0), (0, 0, 0, 0));
}

#[test]
fn ellipse_render_regions() {
    let pixmap = render_graphic("ellipse", None, 64, 64).unwrap();
    assert_eq!(pixel(&pixmap, 32, 32), (255, 255, 255, 255));
    assert_eq!(pixel(&pixmap, 0, 0), (0, 0, 0, 0));
    assert_eq!(pixel(&pixmap, 63, 63), (0, 0, 0, 0));
}

#[test]
fn star_render_regions() {
    let pixmap = render_graphic("star", None, 64, 64).unwrap();
    assert_eq!(pixel(&pixmap, 32, 32), (255, 255, 255, 255));
    assert_eq!(pixel(&pixmap, 0, 0), (0, 0, 0, 0));
}

#[test]
fn polygon_render_regions() {
    let pixmap = render_graphic(
        "polygon",
        Some(&params(&[("sides", ParamValue::Number(6.0))])),
        64,
        64,
    )
    .unwrap();
    assert_eq!(pixel(&pixmap, 32, 32), (255, 255, 255, 255));
    assert_eq!(pixel(&pixmap, 0, 0), (0, 0, 0, 0));
}

#[test]
fn center_stroke_regions() {
    let pixmap = render_graphic(
        "rectangle",
        Some(&params(&[
            ("strokeWidth", ParamValue::Number(10.0)),
            ("strokeAlign", ParamValue::String("center".to_string())),
        ])),
        64,
        64,
    )
    .unwrap();
    assert_eq!(pixel(&pixmap, 2, 32), (0, 0, 0, 255));
    assert_eq!(pixel(&pixmap, 32, 32), (255, 255, 255, 255));
}

#[test]
fn inside_stroke_regions() {
    let pixmap = render_graphic(
        "rectangle",
        Some(&params(&[
            ("strokeWidth", ParamValue::Number(10.0)),
            ("strokeAlign", ParamValue::String("inside".to_string())),
        ])),
        64,
        64,
    )
    .unwrap();
    assert_eq!(pixel(&pixmap, 4, 32), (0, 0, 0, 255));
    assert_eq!(pixel(&pixmap, 32, 32), (255, 255, 255, 255));
}

#[test]
fn outside_stroke_has_no_inner_ring() {
    let pixmap = render_graphic(
        "rectangle",
        Some(&params(&[
            ("strokeWidth", ParamValue::Number(10.0)),
            ("strokeAlign", ParamValue::String("outside".to_string())),
        ])),
        64,
        64,
    )
    .unwrap();
    assert_eq!(pixel(&pixmap, 12, 32), (255, 255, 255, 255));
    assert_eq!(pixel(&pixmap, 32, 32), (255, 255, 255, 255));
}

#[test]
fn invalid_fill_color_errors() {
    let err = render_graphic(
        "rectangle",
        Some(&params(&[("fill", ParamValue::String("bogus".to_string()))])),
        8,
        8,
    )
    .unwrap_err();
    assert!(matches!(err, GraphicsError::InvalidColor(_)));
}

#[test]
fn gradient_background_linear() {
    let mut pixmap = Pixmap::new(64, 64).unwrap();
    draw_css_background(&mut pixmap, "linear-gradient(to right, #ff0000, #0000ff)").unwrap();
    let left = pixel(&pixmap, 1, 32);
    let right = pixel(&pixmap, 62, 32);
    assert!(left.0 > 200 && left.2 < 60, "left was {left:?}");
    assert!(right.2 > 200 && right.0 < 60, "right was {right:?}");
    assert_eq!(left.3, 255);
    assert_eq!(right.3, 255);
}

#[test]
fn gradient_background_radial() {
    let mut pixmap = Pixmap::new(64, 64).unwrap();
    draw_css_background(&mut pixmap, "radial-gradient(circle, #ffffff, #000000)").unwrap();
    let center = pixel(&pixmap, 32, 32);
    assert!(center.0 > 200, "center was {center:?}");
    let corner = pixel(&pixmap, 0, 0);
    assert!(corner.0 < 60, "corner was {corner:?}");
}

#[test]
fn background_layer_paint_order() {
    let mut pixmap = Pixmap::new(16, 16).unwrap();
    draw_css_background(&mut pixmap, "#00ff00, linear-gradient(#ff0000, #0000ff)").unwrap();
    assert_eq!(pixel(&pixmap, 8, 8), (0, 255, 0, 255));

    let mut pixmap = Pixmap::new(16, 16).unwrap();
    draw_css_background(&mut pixmap, "rgba(255,0,0,0.5)").unwrap();
    let px = pixel(&pixmap, 8, 8);
    assert!(px.3 > 120 && px.3 < 135, "alpha was {px:?}");
    assert!(px.0 > 120 && px.0 < 135, "premultiplied red was {px:?}");
    assert_eq!(px.1, 0);
}
