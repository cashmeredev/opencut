mod freeform;
mod geometry;
mod paint;
mod shapes;
mod split;
mod text_mask;

pub use freeform::build_freeform_path;

use scene::Mask;
use tiny_skia::Pixmap;

#[derive(Debug, thiserror::Error)]
pub enum MaskRenderError {
    #[error("invalid pixmap size {width}x{height}")]
    InvalidSize { width: u32, height: u32 },
    #[error("invalid stroke color: {0}")]
    InvalidStrokeColor(String),
}

fn new_pixmap(width: u32, height: u32) -> Result<Pixmap, MaskRenderError> {
    Pixmap::new(width, height).ok_or(MaskRenderError::InvalidSize { width, height })
}

pub fn is_active(mask: &Mask) -> bool {
    match mask {
        Mask::Text { params, .. } => !params.content.trim().is_empty(),
        Mask::Freeform { params, .. } => params.closed,
        _ => true,
    }
}

pub fn feather_for_gpu(mask: &Mask) -> f32 {
    match mask {
        Mask::Split { .. } => 0.0,
        Mask::CinematicBars { params, .. }
        | Mask::Rectangle { params, .. }
        | Mask::Ellipse { params, .. }
        | Mask::Heart { params, .. }
        | Mask::Diamond { params, .. }
        | Mask::Star { params, .. } => params.feather as f32,
        Mask::Text { params, .. } => params.feather as f32,
        Mask::Freeform { params, .. } => params.feather as f32,
    }
}

pub fn render_mask_body(
    mask: &Mask,
    width: u32,
    height: u32,
) -> Result<Pixmap, MaskRenderError> {
    let mut pixmap = new_pixmap(width, height)?;
    let w = width as f64;
    let h = height as f64;
    match mask {
        Mask::Split { params, .. } => {
            if params.feather == 0.0 {
                paint::fill_white(&split::split_opaque_body_path(params, w, h), &mut pixmap);
            } else {
                split::fill_split_feather_body(params, w, h, &mut pixmap);
            }
        }
        Mask::CinematicBars { params, .. } => {
            paint::fill_white(&shapes::cinematic_bars_body_path(params, w, h), &mut pixmap);
        }
        Mask::Rectangle { params, .. } => {
            paint::fill_white(&shapes::rectangle_body_path(params, w, h), &mut pixmap);
        }
        Mask::Ellipse { params, .. } => {
            paint::fill_white(&shapes::ellipse_body_path(params, w, h), &mut pixmap);
        }
        Mask::Heart { params, .. } => {
            paint::fill_white(&shapes::heart_body_path(params, w, h), &mut pixmap);
        }
        Mask::Diamond { params, .. } => {
            paint::fill_white(&shapes::diamond_body_path(params, w, h), &mut pixmap);
        }
        Mask::Star { params, .. } => {
            paint::fill_white(&shapes::star_body_path(params, w, h), &mut pixmap);
        }
        Mask::Text { params, .. } => {
            text_mask::render_body(params, w, h, &mut pixmap);
        }
        Mask::Freeform { params, .. } => {
            if params.closed {
                let path = build_freeform_path(
                    &params.path,
                    params.center_x,
                    params.center_y,
                    params.rotation,
                    params.scale,
                    w,
                    h,
                    true,
                );
                paint::fill_white(&path, &mut pixmap);
            }
        }
    }
    Ok(pixmap)
}

pub fn render_mask_stroke(
    mask: &Mask,
    width: u32,
    height: u32,
) -> Result<Option<Pixmap>, MaskRenderError> {
    let (stroke_width, stroke_color) = match mask {
        Mask::Split { params, .. } => (params.stroke_width, &params.stroke_color),
        Mask::CinematicBars { params, .. }
        | Mask::Rectangle { params, .. }
        | Mask::Ellipse { params, .. }
        | Mask::Heart { params, .. }
        | Mask::Diamond { params, .. }
        | Mask::Star { params, .. } => (params.stroke_width, &params.stroke_color),
        Mask::Text { params, .. } => (params.stroke_width, &params.stroke_color),
        Mask::Freeform { params, .. } => (params.stroke_width, &params.stroke_color),
    };
    if stroke_width <= 0.0 {
        return Ok(None);
    }

    let mut pixmap = new_pixmap(width, height)?;
    let w = width as f64;
    let h = height as f64;
    match mask {
        Mask::Split { params, .. } => {
            let color = paint::parse_stroke_color(stroke_color)?;
            paint::stroke_path_with(
                &split::split_stroke_path(params, w, h),
                &paint::canvas_stroke(stroke_width),
                color,
                &mut pixmap,
            );
        }
        Mask::CinematicBars { .. }
        | Mask::Rectangle { .. }
        | Mask::Ellipse { .. }
        | Mask::Heart { .. }
        | Mask::Diamond { .. }
        | Mask::Star { .. } => {
            let path = match mask {
                Mask::CinematicBars { params, .. } => {
                    shapes::cinematic_bars_stroke_path(params, w, h)
                }
                Mask::Rectangle { params, .. } => shapes::rectangle_stroke_path(params, w, h),
                Mask::Ellipse { params, .. } => shapes::ellipse_stroke_path(params, w, h),
                Mask::Heart { params, .. } => shapes::heart_stroke_path(params, w, h),
                Mask::Diamond { params, .. } => shapes::diamond_stroke_path(params, w, h),
                Mask::Star { params, .. } => shapes::star_stroke_path(params, w, h),
                _ => unreachable!(),
            };
            let color = paint::parse_stroke_color(stroke_color)?;
            paint::stroke_path_with(
                &path,
                &paint::canvas_stroke(stroke_width),
                color,
                &mut pixmap,
            );
        }
        Mask::Text { params, .. } => {
            text_mask::render_stroke(params, w, h, &mut pixmap)?;
        }
        Mask::Freeform { params, .. } => {
            if params.closed {
                let path = build_freeform_path(
                    &params.path,
                    params.center_x,
                    params.center_y,
                    params.rotation,
                    params.scale,
                    w,
                    h,
                    true,
                );
                let color = paint::parse_stroke_color(stroke_color)?;
                paint::stroke_path_with(
                    &path,
                    &paint::round_stroke(stroke_width),
                    color,
                    &mut pixmap,
                );
                if params.stroke_align != scene::StrokeAlign::Center {
                    let mut fill = new_pixmap(width, height)?;
                    paint::fill_white(&path, &mut fill);
                    paint::apply_stroke_align(&mut pixmap, &fill, params.stroke_align);
                }
            }
        }
    }
    Ok(Some(pixmap))
}
