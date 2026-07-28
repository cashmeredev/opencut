use graphics::gradient::ast::*;
use graphics::gradient::parser::parse_gradient;

fn single(code: &str) -> GradientAst {
    let ast = parse_gradient(code).unwrap_or_else(|err| panic!("failed to parse {code:?}: {err}"));
    assert_eq!(ast.len(), 1, "expected one gradient in {code:?}");
    ast.into_iter().next().unwrap()
}

#[test]
fn linear_minimal() {
    let gradient = single("linear-gradient(red, blue)");
    assert_eq!(gradient.kind, GradientKind::Linear);
    assert_eq!(gradient.orientation, None);
    assert_eq!(
        gradient.color_stops,
        vec![
            ColorStop { color: Color::Literal("red".into()), length: None },
            ColorStop { color: Color::Literal("blue".into()), length: None },
        ]
    );
}

#[test]
fn linear_directional() {
    let gradient = single("linear-gradient(to right, #fff, #000)");
    assert_eq!(
        gradient.orientation,
        Some(GradientOrientation::Linear(LinearOrientation::Directional(
            "right".into()
        )))
    );
    assert_eq!(gradient.color_stops[0].color, Color::Hex("fff".into()));
}

#[test]
fn linear_corner() {
    let gradient = single("linear-gradient(to left top, red, blue)");
    assert_eq!(
        gradient.orientation,
        Some(GradientOrientation::Linear(LinearOrientation::Directional(
            "left top".into()
        )))
    );
    let gradient = single("linear-gradient(to bottom right, red, blue)");
    assert_eq!(
        gradient.orientation,
        Some(GradientOrientation::Linear(LinearOrientation::Directional(
            "bottom right".into()
        )))
    );
}

#[test]
fn linear_angles() {
    let gradient = single("linear-gradient(45deg, red, blue)");
    assert_eq!(
        gradient.orientation,
        Some(GradientOrientation::Linear(LinearOrientation::Angular(
            "45".into()
        )))
    );
    let gradient = single("linear-gradient(-90deg, red, blue)");
    assert_eq!(
        gradient.orientation,
        Some(GradientOrientation::Linear(LinearOrientation::Angular(
            "-90".into()
        )))
    );
    let gradient = single("linear-gradient(0.5rad, red, blue)");
    assert_eq!(
        gradient.orientation,
        Some(GradientOrientation::Linear(LinearOrientation::Angular(
            "0.5".into()
        )))
    );
    let gradient = single("linear-gradient(.25deg, red, blue)");
    assert_eq!(
        gradient.orientation,
        Some(GradientOrientation::Linear(LinearOrientation::Angular(
            ".25".into()
        )))
    );
}

#[test]
fn linear_legacy_direction() {
    let gradient = single("linear-gradient(left, red, blue)");
    assert_eq!(
        gradient.orientation,
        Some(GradientOrientation::Linear(LinearOrientation::Directional(
            "left".into()
        )))
    );
}

#[test]
fn repeating_and_vendor_prefix() {
    assert_eq!(
        single("repeating-linear-gradient(red, blue)").kind,
        GradientKind::RepeatingLinear
    );
    assert_eq!(
        single("repeating-radial-gradient(red, blue)").kind,
        GradientKind::RepeatingRadial
    );
    assert_eq!(
        single("-webkit-linear-gradient(red, blue)").kind,
        GradientKind::Linear
    );
    assert_eq!(
        single("-moz-radial-gradient(red, blue)").kind,
        GradientKind::Radial
    );
    assert_eq!(
        single("LINEAR-GRADIENT(red, blue)").kind,
        GradientKind::Linear
    );
}

#[test]
fn radial_shapes() {
    let gradient = single("radial-gradient(circle, red, blue)");
    assert_eq!(
        gradient.orientation,
        Some(GradientOrientation::Radial(vec![RadialOrientation::Shape(
            Shape { value: ShapeKind::Circle, style: None, at: None }
        )]))
    );

    let gradient = single("radial-gradient(circle 50px, red, blue)");
    assert_eq!(
        gradient.orientation,
        Some(GradientOrientation::Radial(vec![RadialOrientation::Shape(
            Shape {
                value: ShapeKind::Circle,
                style: Some(ShapeStyle::Distance(Distance::Px("50".into()))),
                at: None,
            }
        )]))
    );

    let gradient = single("radial-gradient(ellipse closest-side, red, blue)");
    assert_eq!(
        gradient.orientation,
        Some(GradientOrientation::Radial(vec![RadialOrientation::Shape(
            Shape {
                value: ShapeKind::Ellipse,
                style: Some(ShapeStyle::ExtentKeyword("closest-side".into())),
                at: None,
            }
        )]))
    );
}

#[test]
fn radial_with_position() {
    let gradient = single("radial-gradient(circle at 25% 25%, red, blue)");
    assert_eq!(
        gradient.orientation,
        Some(GradientOrientation::Radial(vec![RadialOrientation::Shape(
            Shape {
                value: ShapeKind::Circle,
                style: None,
                at: Some(Position {
                    x: Some(Distance::Percent("25".into())),
                    y: Some(Distance::Percent("25".into())),
                }),
            }
        )]))
    );

    let gradient = single("radial-gradient(farthest-corner at 10px 2em, red, blue)");
    assert_eq!(
        gradient.orientation,
        Some(GradientOrientation::Radial(vec![
            RadialOrientation::ExtentKeyword {
                value: "farthest-corner".into(),
                at: Some(Position {
                    x: Some(Distance::Px("10".into())),
                    y: Some(Distance::Em("2".into())),
                }),
            }
        ]))
    );

    let gradient = single("radial-gradient(at center, red, blue)");
    assert_eq!(
        gradient.orientation,
        Some(GradientOrientation::Radial(vec![
            RadialOrientation::DefaultRadial {
                at: Position {
                    x: Some(Distance::PositionKeyword("center".into())),
                    y: None,
                },
            }
        ]))
    );

    let gradient = single("radial-gradient(100px 50px at 10% 20%, red, blue)");
    assert_eq!(
        gradient.orientation,
        Some(GradientOrientation::Radial(vec![RadialOrientation::Shape(
            Shape {
                value: ShapeKind::Ellipse,
                style: Some(ShapeStyle::Position(Position {
                    x: Some(Distance::Px("100".into())),
                    y: Some(Distance::Px("50".into())),
                })),
                at: Some(Position {
                    x: Some(Distance::Percent("10".into())),
                    y: Some(Distance::Percent("20".into())),
                }),
            }
        )]))
    );
}

#[test]
fn radial_multiple_orientations() {
    let gradient = single("radial-gradient(circle, ellipse, red, blue)");
    match gradient.orientation {
        Some(GradientOrientation::Radial(list)) => assert_eq!(list.len(), 2),
        other => panic!("unexpected orientation: {other:?}"),
    }
}

#[test]
fn color_formats() {
    let gradient = single("linear-gradient(#abcdef, rgb(1,2,3), rgba(1,2,3,0.5), hsl(120,50%,50%), hsla(120,50%,50%,0.5), var(--brand), rebeccapurple)");
    assert_eq!(gradient.color_stops[0].color, Color::Hex("abcdef".into()));
    assert_eq!(
        gradient.color_stops[1].color,
        Color::Rgb(vec!["1".into(), "2".into(), "3".into()])
    );
    assert_eq!(
        gradient.color_stops[2].color,
        Color::Rgba(vec!["1".into(), "2".into(), "3".into(), "0.5".into()])
    );
    assert_eq!(
        gradient.color_stops[3].color,
        Color::Hsl(["120".into(), "50".into(), "50".into()])
    );
    assert_eq!(
        gradient.color_stops[4].color,
        Color::Hsla(["120".into(), "50".into(), "50".into(), "0.5".into()])
    );
    assert_eq!(gradient.color_stops[5].color, Color::Var("--brand".into()));
    assert_eq!(
        gradient.color_stops[6].color,
        Color::Literal("rebeccapurple".into())
    );
}

#[test]
fn stop_lengths() {
    let gradient = single("linear-gradient(red 10%, blue 20px, green 3em, white calc(10% + 5px))");
    assert_eq!(
        gradient.color_stops[0].length,
        Some(Distance::Percent("10".into()))
    );
    assert_eq!(
        gradient.color_stops[1].length,
        Some(Distance::Px("20".into()))
    );
    assert_eq!(
        gradient.color_stops[2].length,
        Some(Distance::Em("3".into()))
    );
    assert_eq!(
        gradient.color_stops[3].length,
        Some(Distance::Calc("10% + 5px".into()))
    );
}

#[test]
fn multiple_gradients() {
    let ast = parse_gradient("linear-gradient(red, blue), radial-gradient(circle, white, black)")
        .unwrap();
    assert_eq!(ast.len(), 2);
    assert_eq!(ast[0].kind, GradientKind::Linear);
    assert_eq!(ast[1].kind, GradientKind::Radial);
}

#[test]
fn trailing_semicolon_and_whitespace() {
    let ast = parse_gradient("  linear-gradient(red, blue);  ").unwrap();
    assert_eq!(ast.len(), 1);
}

#[test]
fn empty_input_is_empty_list() {
    assert_eq!(parse_gradient("").unwrap(), vec![]);
}

#[test]
fn double_space_directional_quirk() {
    let gradient = single("linear-gradient(to  right, red, blue)");
    assert_eq!(gradient.orientation, None);
    assert_eq!(gradient.color_stops[0].color, Color::Literal("to".into()));
    assert_eq!(
        gradient.color_stops[0].length,
        Some(Distance::PositionKeyword("right".into()))
    );
}

#[test]
fn error_missing_close_paren() {
    let err = parse_gradient("linear-gradient(red, blue").unwrap_err();
    assert_eq!(err.message, "Missing )");
}

#[test]
fn error_extra_comma() {
    let err = parse_gradient("linear-gradient(red,,blue)").unwrap_err();
    assert_eq!(err.message, "Expected color definition");
}

#[test]
fn error_missing_comma_before_color_stops() {
    let err = parse_gradient("linear-gradient(to left red, blue)").unwrap_err();
    assert_eq!(err.message, "Missing comma before color stops");
}

#[test]
fn error_invalid_input_not_eof() {
    let err = parse_gradient("foo(red)").unwrap_err();
    assert_eq!(err.message, "Invalid input not EOF");
    assert_eq!(err.input, "foo(red)");
}

#[test]
fn error_hsl_hue_percentage() {
    let err = parse_gradient("linear-gradient(hsl(50%, 50%, 50%), red)").unwrap_err();
    assert!(err.message.starts_with("HSL hue value must be a number"));
}

#[test]
fn error_hsl_missing_percentages() {
    let err = parse_gradient("linear-gradient(hsl(120, 50, 50), red)").unwrap_err();
    assert_eq!(
        err.message,
        "Expected percentage value for saturation and lightness in HSL"
    );
}

#[test]
fn error_rgb_empty() {
    let err = parse_gradient("linear-gradient(rgb(), red)").unwrap_err();
    assert_eq!(err.message, "Expected number");
}

#[test]
fn error_calc_unbalanced() {
    let err = parse_gradient("linear-gradient(red calc(10%").unwrap_err();
    assert_eq!(err.message, "Missing closing parenthesis in calc() expression");
}

#[test]
fn error_var_missing_name() {
    let err = parse_gradient("linear-gradient(var(), red)").unwrap_err();
    assert_eq!(err.message, "Expected CSS variable name");
}

#[test]
fn error_at_missing_position() {
    let err = parse_gradient("radial-gradient(circle at, red, blue)").unwrap_err();
    assert_eq!(err.message, "Missing positioning value");
}

#[test]
fn error_display_format() {
    let err = parse_gradient("foo()").unwrap_err();
    assert_eq!(err.to_string(), "foo(): Invalid input not EOF");
}
