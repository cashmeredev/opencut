use super::ast::{Color, ColorStop, Distance};
use crate::color::{is_transparent, parse_color_to_rgb};

#[derive(Debug, Clone, PartialEq)]
pub struct NormalizedColorStop {
    pub color: String,
    pub offset: f64,
}

pub fn color_to_string(color: &Color) -> String {
    match color {
        Color::Hex(value) => format!("#{value}"),
        Color::Literal(value) => value.clone(),
        Color::Rgb(values) => format!("rgb({})", values.join(",")),
        Color::Rgba(values) => format!("rgba({})", values.join(",")),
        Color::Hsl(values) => format!("hsl({})", values.join(",")),
        Color::Hsla(values) => format!("hsla({})", values.join(",")),
        Color::Var(value) => format!("var({value})"),
    }
}

fn clamp01(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

fn resolve_stop_offset(stop: &ColorStop, gradient_length: f64) -> Option<f64> {
    match stop.length.as_ref()? {
        Distance::Percent(value) => value.parse::<f64>().ok().map(|v| v / 100.0),
        Distance::Px(value) => value.parse::<f64>().ok().map(|v| v / gradient_length),
        Distance::Em(value) => value
            .parse::<f64>()
            .ok()
            .map(|v| (v * 16.0) / gradient_length),
        _ => None,
    }
}

fn fix_transparent_stops(stops: &[(String, Option<f64>)]) -> Vec<(String, Option<f64>)> {
    let mut result = stops.to_vec();
    for i in 0..result.len() {
        if !is_transparent(&result[i].0) {
            continue;
        }
        let mut donor = None;
        for j in (0..i).rev() {
            if !is_transparent(&result[j].0) {
                donor = parse_color_to_rgb(&result[j].0);
                if donor.is_some() {
                    break;
                }
            }
        }
        if donor.is_none() {
            for item in result.iter().skip(i + 1) {
                if !is_transparent(&item.0) {
                    donor = parse_color_to_rgb(&item.0);
                    if donor.is_some() {
                        break;
                    }
                }
            }
        }
        if let Some(donor) = donor {
            result[i].0 = format!("rgba({},{},{},0)", donor.r, donor.g, donor.b);
        }
    }
    result
}

pub fn normalize_color_stops(
    color_stops: &[ColorStop],
    gradient_length: f64,
) -> Vec<NormalizedColorStop> {
    let mapped: Vec<(String, Option<f64>)> = color_stops
        .iter()
        .map(|stop| {
            (
                color_to_string(&stop.color),
                resolve_stop_offset(stop, gradient_length),
            )
        })
        .collect();

    let fixed = fix_transparent_stops(&mapped);
    let mut resolved = fixed;
    let known_indices: Vec<usize> = resolved
        .iter()
        .enumerate()
        .filter_map(|(index, stop)| stop.1.map(|_| index))
        .collect();

    if known_indices.is_empty() {
        let step = if resolved.len() > 1 {
            1.0 / (resolved.len() - 1) as f64
        } else {
            1.0
        };
        for (index, stop) in resolved.iter_mut().enumerate() {
            stop.1 = Some(step * index as f64);
        }
        return resolved
            .into_iter()
            .map(|(color, offset)| NormalizedColorStop {
                color,
                offset: clamp01(offset.unwrap_or(0.0)),
            })
            .collect();
    }

    let first_known = known_indices[0];
    if resolved[first_known].1.is_none() {
        resolved[first_known].1 = Some(0.0);
    }
    for index in 0..first_known {
        let next_offset = resolved[first_known].1.unwrap_or(0.0);
        resolved[index].1 = Some((next_offset * index as f64) / first_known as f64);
    }

    for window in known_indices.windows(2) {
        let start_index = window[0];
        let end_index = window[1];
        let start_offset = resolved[start_index].1.unwrap_or(0.0);
        let end_offset = resolved[end_index].1.unwrap_or(start_offset);
        let gap = end_index - start_index;
        if gap <= 1 {
            continue;
        }
        let step = (end_offset - start_offset) / gap as f64;
        for index in 1..gap {
            resolved[start_index + index].1 = Some(start_offset + step * index as f64);
        }
    }

    let last_known = known_indices[known_indices.len() - 1];
    let last_offset = resolved[last_known].1.unwrap_or(1.0);
    if last_known < resolved.len() - 1 {
        let gap = resolved.len() - 1 - last_known;
        let step = (1.0 - last_offset) / gap as f64;
        for index in 1..=gap {
            resolved[last_known + index].1 = Some(last_offset + step * index as f64);
        }
    }

    resolved
        .into_iter()
        .map(|(color, offset)| NormalizedColorStop {
            color,
            offset: clamp01(offset.unwrap_or(0.0)),
        })
        .collect()
}
