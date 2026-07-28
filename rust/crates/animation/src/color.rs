#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LinearRgba {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

fn clamp_unit(value: f64) -> f64 {
    value.max(0.0).min(1.0)
}

fn srgb_to_linear(value: f64) -> f64 {
    if value <= 0.04045 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(value: f64) -> f64 {
    let clamped = clamp_unit(value);
    if clamped <= 0.003_130_8 {
        clamped * 12.92
    } else {
        1.055 * clamped.powf(1.0 / 2.4) - 0.055
    }
}

fn parse_hex_digit(digit: char) -> Option<f64> {
    digit.to_digit(16).map(|value| value as f64 / 15.0)
}

fn parse_hex_pair(pair: &str) -> Option<f64> {
    u8::from_str_radix(pair, 16)
        .ok()
        .map(|value| value as f64 / 255.0)
}

pub fn parse_color_to_linear_rgba(color: &str) -> Option<LinearRgba> {
    let hex = color.strip_prefix('#')?;
    let (r, g, b, a) = match hex.len() {
        3 | 4 => {
            let mut digits = hex.chars();
            let r = parse_hex_digit(digits.next()?)?;
            let g = parse_hex_digit(digits.next()?)?;
            let b = parse_hex_digit(digits.next()?)?;
            let a = match digits.next() {
                Some(digit) => parse_hex_digit(digit)?,
                None => 1.0,
            };
            (r, g, b, a)
        }
        6 | 8 => {
            let r = parse_hex_pair(&hex[0..2])?;
            let g = parse_hex_pair(&hex[2..4])?;
            let b = parse_hex_pair(&hex[4..6])?;
            let a = if hex.len() == 8 {
                parse_hex_pair(&hex[6..8])?
            } else {
                1.0
            };
            (r, g, b, a)
        }
        _ => return None,
    };
    Some(LinearRgba {
        r: srgb_to_linear(r),
        g: srgb_to_linear(g),
        b: srgb_to_linear(b),
        a: clamp_unit(a),
    })
}

fn hex_component(value: f64) -> u8 {
    (clamp_unit(value) * 255.0).round() as u8
}

pub fn format_linear_rgba(color: &LinearRgba) -> String {
    let r = hex_component(linear_to_srgb(color.r));
    let g = hex_component(linear_to_srgb(color.g));
    let b = hex_component(linear_to_srgb(color.b));
    let a = clamp_unit(color.a);
    if a < 1.0 {
        format!("#{:02x}{:02x}{:02x}{:02x}", r, g, b, hex_component(a))
    } else {
        format!("#{:02x}{:02x}{:02x}", r, g, b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hex_colors_to_linear() {
        let red = parse_color_to_linear_rgba("#ff0000").unwrap();
        assert_eq!(red.r, 1.0);
        assert_eq!(red.g, 0.0);
        assert_eq!(red.b, 0.0);
        assert_eq!(red.a, 1.0);

        let short = parse_color_to_linear_rgba("#fff").unwrap();
        assert_eq!(short.r, 1.0);
        assert_eq!(short.g, 1.0);

        let with_alpha = parse_color_to_linear_rgba("#00000080").unwrap();
        assert!((with_alpha.a - 128.0 / 255.0).abs() < 1e-9);

        assert!(parse_color_to_linear_rgba("red").is_none());
        assert!(parse_color_to_linear_rgba("#ff").is_none());
    }

    #[test]
    fn formats_linear_colors_as_hex() {
        let red = LinearRgba {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };
        assert_eq!(format_linear_rgba(&red), "#ff0000");

        let translucent = LinearRgba {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.5,
        };
        assert_eq!(format_linear_rgba(&translucent), "#00000080");
    }

    #[test]
    fn hex_round_trips_through_linear_space() {
        for color in ["#ffffff", "#000000", "#123456", "#abcdef"] {
            let parsed = parse_color_to_linear_rgba(color).unwrap();
            assert_eq!(format_linear_rgba(&parsed), color);
        }
    }
}
