use text::{
    Color, RasterOptions, StrokeStyle, TextAlign, TextBackground, TextEngine, TextLayoutParams,
    rasterize, text_visual_rect,
};

const FONT_REGULAR: &[u8] = include_bytes!("fixtures/DejaVuSans.ttf");
const FONT_BOLD: &[u8] = include_bytes!("fixtures/DejaVuSans-Bold.ttf");

fn engine() -> TextEngine {
    TextEngine::with_font_bytes(&[FONT_REGULAR, FONT_BOLD])
}

fn params(content: &str) -> TextLayoutParams {
    TextLayoutParams {
        content: content.to_string(),
        font_family: "DejaVu Sans".to_string(),
        ..TextLayoutParams::default()
    }
}

fn opaque_pixels(buffer: &text::RgbaBuffer) -> usize {
    buffer.pixels.chunks(4).filter(|px| px[3] > 0).count()
}

#[test]
fn raster_produces_non_empty_output_with_visual_rect_dimensions() {
    let mut engine = engine();
    let measured = engine.measure(&params("Hello"), 90.0);
    let visual = text_visual_rect(
        TextAlign::Center,
        &measured.block,
        &TextBackground::default(),
        1.0,
    );
    let buffer = rasterize(
        &mut engine,
        &measured,
        &RasterOptions {
            text_color: Color::rgb(255, 255, 255),
            background: None,
            stroke: None,
        },
    );

    assert_eq!(buffer.width, visual.width.ceil() as u32);
    assert_eq!(buffer.height, visual.height.ceil() as u32);
    assert_eq!(buffer.pixels.len(), buffer.width as usize * buffer.height as usize * 4);
    assert!(opaque_pixels(&buffer) > 0);
    assert_eq!(buffer.pixel(0, 0).a, 0);
}

#[test]
fn raster_fills_background_inside_padded_box() {
    let mut engine = engine();
    let measured = engine.measure(&params("Hello"), 90.0);
    let background = TextBackground {
        enabled: true,
        color: "#ff0000".to_string(),
        ..TextBackground::default()
    };
    let red = Color::rgb(255, 0, 0);
    let buffer = rasterize(
        &mut engine,
        &measured,
        &RasterOptions {
            text_color: Color::rgb(255, 255, 255),
            background: Some((&background, red)),
            stroke: None,
        },
    );

    assert_eq!(buffer.height, 102);
    let corner = buffer.pixel(2, 2);
    assert!(corner.a > 200);
    assert!(corner.r > 200);
    assert!(corner.g < 60);
    assert!(corner.b < 60);
}

#[test]
fn raster_dimensions_grow_with_background_padding() {
    let mut engine = engine();
    let measured = engine.measure(&params("Hello"), 90.0);
    let plain = rasterize(
        &mut engine,
        &measured,
        &RasterOptions {
            text_color: Color::rgb(255, 255, 255),
            background: None,
            stroke: None,
        },
    );
    let background = TextBackground {
        enabled: true,
        color: "#ff0000".to_string(),
        ..TextBackground::default()
    };
    let padded = rasterize(
        &mut engine,
        &measured,
        &RasterOptions {
            text_color: Color::rgb(255, 255, 255),
            background: Some((&background, Color::rgb(255, 0, 0))),
            stroke: None,
        },
    );

    assert_eq!(padded.width, plain.width + 60);
    assert_eq!(padded.height, plain.height + 84);
}

#[test]
fn raster_underline_adds_pixels_below_text() {
    let mut engine = engine();
    let plain_params = params("Hello");
    let plain = engine.measure(&plain_params, 90.0);
    let plain_buffer = rasterize(
        &mut engine,
        &plain,
        &RasterOptions {
            text_color: Color::rgb(255, 255, 255),
            background: None,
            stroke: None,
        },
    );

    let underlined_params = TextLayoutParams {
        text_decoration: text::TextDecoration::Underline,
        ..params("Hello")
    };
    let underlined = engine.measure(&underlined_params, 90.0);
    let underlined_buffer = rasterize(
        &mut engine,
        &underlined,
        &RasterOptions {
            text_color: Color::rgb(255, 255, 255),
            background: None,
            stroke: None,
        },
    );

    assert!(opaque_pixels(&underlined_buffer) > opaque_pixels(&plain_buffer));
}

#[test]
fn raster_stroke_widens_glyph_coverage() {
    let mut engine = engine();
    let measured = engine.measure(&params("Hello"), 180.0);
    let fill_only = rasterize(
        &mut engine,
        &measured,
        &RasterOptions {
            text_color: Color::rgb(255, 255, 255),
            background: None,
            stroke: None,
        },
    );
    let stroked = rasterize(
        &mut engine,
        &measured,
        &RasterOptions {
            text_color: Color::rgb(255, 255, 255),
            background: None,
            stroke: Some(StrokeStyle {
                color: Color::rgb(0, 0, 0),
                width: 4.0,
            }),
        },
    );

    assert!(opaque_pixels(&stroked) > opaque_pixels(&fill_only));
}
