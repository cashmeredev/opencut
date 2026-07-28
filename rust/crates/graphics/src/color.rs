#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgba {
    pub r: f64,
    pub g: f64,
    pub b: f64,
    pub a: f64,
}

const NAMED_COLORS: &[(&str, u8, u8, u8)] = &[
    ("aliceblue", 240, 248, 255),
    ("antiquewhite", 250, 235, 215),
    ("aqua", 0, 255, 255),
    ("aquamarine", 127, 255, 212),
    ("azure", 240, 255, 255),
    ("beige", 245, 245, 220),
    ("bisque", 255, 228, 196),
    ("black", 0, 0, 0),
    ("blanchedalmond", 255, 235, 205),
    ("blue", 0, 0, 255),
    ("blueviolet", 138, 43, 226),
    ("brown", 165, 42, 42),
    ("burlywood", 222, 184, 135),
    ("cadetblue", 95, 158, 160),
    ("chartreuse", 127, 255, 0),
    ("chocolate", 210, 105, 30),
    ("coral", 255, 127, 80),
    ("cornflowerblue", 100, 149, 237),
    ("cornsilk", 255, 248, 220),
    ("crimson", 220, 20, 60),
    ("cyan", 0, 255, 255),
    ("darkblue", 0, 0, 139),
    ("darkcyan", 0, 139, 139),
    ("darkgoldenrod", 184, 134, 11),
    ("darkgray", 169, 169, 169),
    ("darkgreen", 0, 100, 0),
    ("darkgrey", 169, 169, 169),
    ("darkkhaki", 189, 183, 107),
    ("darkmagenta", 139, 0, 139),
    ("darkolivegreen", 85, 107, 47),
    ("darkorange", 255, 140, 0),
    ("darkorchid", 153, 50, 204),
    ("darkred", 139, 0, 0),
    ("darksalmon", 233, 150, 122),
    ("darkseagreen", 143, 188, 143),
    ("darkslateblue", 72, 61, 139),
    ("darkslategray", 47, 79, 79),
    ("darkslategrey", 47, 79, 79),
    ("darkturquoise", 0, 206, 209),
    ("darkviolet", 148, 0, 211),
    ("deeppink", 255, 20, 147),
    ("deepskyblue", 0, 191, 255),
    ("dimgray", 105, 105, 105),
    ("dimgrey", 105, 105, 105),
    ("dodgerblue", 30, 144, 255),
    ("firebrick", 178, 34, 34),
    ("floralwhite", 255, 250, 240),
    ("forestgreen", 34, 139, 34),
    ("fuchsia", 255, 0, 255),
    ("gainsboro", 220, 220, 220),
    ("ghostwhite", 248, 248, 255),
    ("gold", 255, 215, 0),
    ("goldenrod", 218, 165, 32),
    ("gray", 128, 128, 128),
    ("green", 0, 128, 0),
    ("greenyellow", 173, 255, 47),
    ("grey", 128, 128, 128),
    ("honeydew", 240, 255, 240),
    ("hotpink", 255, 105, 180),
    ("indianred", 205, 92, 92),
    ("indigo", 75, 0, 130),
    ("ivory", 255, 255, 240),
    ("khaki", 240, 230, 140),
    ("lavender", 230, 230, 250),
    ("lavenderblush", 255, 240, 245),
    ("lawngreen", 124, 252, 0),
    ("lemonchiffon", 255, 250, 205),
    ("lightblue", 173, 216, 230),
    ("lightcoral", 240, 128, 128),
    ("lightcyan", 224, 255, 255),
    ("lightgoldenrodyellow", 250, 250, 210),
    ("lightgray", 211, 211, 211),
    ("lightgreen", 144, 238, 144),
    ("lightgrey", 211, 211, 211),
    ("lightpink", 255, 182, 193),
    ("lightsalmon", 255, 160, 122),
    ("lightseagreen", 32, 178, 170),
    ("lightskyblue", 135, 206, 250),
    ("lightslategray", 119, 136, 153),
    ("lightslategrey", 119, 136, 153),
    ("lightsteelblue", 176, 196, 222),
    ("lightyellow", 255, 255, 224),
    ("lime", 0, 255, 0),
    ("limegreen", 50, 205, 50),
    ("linen", 250, 240, 230),
    ("magenta", 255, 0, 255),
    ("maroon", 128, 0, 0),
    ("mediumaquamarine", 102, 205, 170),
    ("mediumblue", 0, 0, 205),
    ("mediumorchid", 186, 85, 211),
    ("mediumpurple", 147, 112, 219),
    ("mediumseagreen", 60, 179, 113),
    ("mediumslateblue", 123, 104, 238),
    ("mediumspringgreen", 0, 250, 154),
    ("mediumturquoise", 72, 209, 204),
    ("mediumvioletred", 199, 21, 133),
    ("midnightblue", 25, 25, 112),
    ("mintcream", 245, 255, 250),
    ("mistyrose", 255, 228, 225),
    ("moccasin", 255, 228, 181),
    ("navajowhite", 255, 222, 173),
    ("navy", 0, 0, 128),
    ("oldlace", 253, 245, 230),
    ("olive", 128, 128, 0),
    ("olivedrab", 107, 142, 35),
    ("orange", 255, 165, 0),
    ("orangered", 255, 69, 0),
    ("orchid", 218, 112, 214),
    ("palegoldenrod", 238, 232, 170),
    ("palegreen", 152, 251, 152),
    ("paleturquoise", 175, 238, 238),
    ("palevioletred", 219, 112, 147),
    ("papayawhip", 255, 239, 213),
    ("peachpuff", 255, 218, 185),
    ("peru", 205, 133, 63),
    ("pink", 255, 192, 203),
    ("plum", 221, 160, 221),
    ("powderblue", 176, 224, 230),
    ("purple", 128, 0, 128),
    ("rebeccapurple", 102, 51, 153),
    ("red", 255, 0, 0),
    ("rosybrown", 188, 143, 143),
    ("royalblue", 65, 105, 225),
    ("saddlebrown", 139, 69, 19),
    ("salmon", 250, 128, 114),
    ("sandybrown", 244, 164, 96),
    ("seagreen", 46, 139, 87),
    ("seashell", 255, 245, 238),
    ("sienna", 160, 82, 45),
    ("silver", 192, 192, 192),
    ("skyblue", 135, 206, 235),
    ("slateblue", 106, 90, 205),
    ("slategray", 112, 128, 144),
    ("slategrey", 112, 128, 144),
    ("snow", 255, 250, 250),
    ("springgreen", 0, 255, 127),
    ("steelblue", 70, 130, 180),
    ("tan", 210, 180, 140),
    ("teal", 0, 128, 128),
    ("thistle", 216, 191, 216),
    ("tomato", 255, 99, 71),
    ("turquoise", 64, 224, 208),
    ("violet", 238, 130, 238),
    ("wheat", 245, 222, 179),
    ("white", 255, 255, 255),
    ("whitesmoke", 245, 245, 245),
    ("yellow", 255, 255, 0),
    ("yellowgreen", 154, 205, 50),
];

pub fn named_color(name: &str) -> Option<Rgba> {
    let lowered = name.to_lowercase();
    if lowered == "transparent" {
        return Some(Rgba { r: 0.0, g: 0.0, b: 0.0, a: 0.0 });
    }
    NAMED_COLORS
        .binary_search_by(|(key, _, _, _)| (*key).cmp(lowered.as_str()))
        .ok()
        .map(|index| {
            let (_, r, g, b) = NAMED_COLORS[index];
            Rgba {
                r: f64::from(r) / 255.0,
                g: f64::from(g) / 255.0,
                b: f64::from(b) / 255.0,
                a: 1.0,
            }
        })
}

fn parse_hex_color(hex: &str) -> Option<Rgba> {
    if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
            Some(Rgba {
                r: f64::from(r) / 255.0,
                g: f64::from(g) / 255.0,
                b: f64::from(b) / 255.0,
                a: 1.0,
            })
        }
        4 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
            let a = u8::from_str_radix(&hex[3..4].repeat(2), 16).ok()?;
            Some(Rgba {
                r: f64::from(r) / 255.0,
                g: f64::from(g) / 255.0,
                b: f64::from(b) / 255.0,
                a: f64::from(a) / 255.0,
            })
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(Rgba {
                r: f64::from(r) / 255.0,
                g: f64::from(g) / 255.0,
                b: f64::from(b) / 255.0,
                a: 1.0,
            })
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
            Some(Rgba {
                r: f64::from(r) / 255.0,
                g: f64::from(g) / 255.0,
                b: f64::from(b) / 255.0,
                a: f64::from(a) / 255.0,
            })
        }
        _ => None,
    }
}

fn parse_channel(text: &str) -> Option<f64> {
    let trimmed = text.trim();
    if let Some(percent) = trimmed.strip_suffix('%') {
        return percent.trim().parse::<f64>().ok().map(|v| v * 255.0 / 100.0);
    }
    trimmed.parse::<f64>().ok()
}

fn parse_alpha(text: &str) -> Option<f64> {
    let trimmed = text.trim();
    if let Some(percent) = trimmed.strip_suffix('%') {
        return percent.trim().parse::<f64>().ok().map(|v| v / 100.0);
    }
    trimmed.parse::<f64>().ok()
}

fn split_function_args(body: &str) -> Vec<String> {
    if body.contains(',') {
        return body.split(',').map(|part| part.trim().to_string()).collect();
    }
    let mut parts: Vec<String> = Vec::new();
    let mut alpha: Option<String> = None;
    for token in body.split_whitespace() {
        if let Some(rest) = token.strip_prefix('/') {
            alpha = Some(rest.trim().to_string());
            continue;
        }
        parts.push(token.to_string());
    }
    if let Some(alpha) = alpha {
        if !alpha.is_empty() {
            parts.push(alpha);
        }
    }
    parts
}

fn parse_rgb_function(body: &str) -> Option<Rgba> {
    let parts = split_function_args(body);
    if parts.len() < 3 || parts.len() > 4 {
        return None;
    }
    let r = parse_channel(&parts[0])?;
    let g = parse_channel(&parts[1])?;
    let b = parse_channel(&parts[2])?;
    let a = if parts.len() == 4 {
        parse_alpha(&parts[3])?
    } else {
        1.0
    };
    Some(Rgba {
        r: r / 255.0,
        g: g / 255.0,
        b: b / 255.0,
        a,
    })
}

fn parse_hue(text: &str) -> Option<f64> {
    let trimmed = text.trim();
    if let Some(deg) = trimmed.strip_suffix("deg") {
        return deg.trim().parse::<f64>().ok();
    }
    if let Some(turn) = trimmed.strip_suffix("turn") {
        return turn.trim().parse::<f64>().ok().map(|v| v * 360.0);
    }
    if let Some(rad) = trimmed.strip_suffix("rad") {
        return rad.trim().parse::<f64>().ok().map(|v| v.to_degrees());
    }
    if let Some(grad) = trimmed.strip_suffix("grad") {
        return grad.trim().parse::<f64>().ok().map(|v| v * 0.9);
    }
    trimmed.parse::<f64>().ok()
}

fn parse_percentage(text: &str) -> Option<f64> {
    text.trim()
        .strip_suffix('%')
        .and_then(|v| v.trim().parse::<f64>().ok())
        .map(|v| v / 100.0)
}

pub fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (f64, f64, f64) {
    let hue = h.rem_euclid(360.0);
    let sat = s.clamp(0.0, 1.0);
    let light = l.clamp(0.0, 1.0);
    let chroma = (1.0 - (2.0 * light - 1.0).abs()) * sat;
    let segment = hue / 60.0;
    let x = chroma * (1.0 - (segment % 2.0 - 1.0).abs());
    let (r1, g1, b1) = match segment as u32 {
        0 => (chroma, x, 0.0),
        1 => (x, chroma, 0.0),
        2 => (0.0, chroma, x),
        3 => (0.0, x, chroma),
        4 => (x, 0.0, chroma),
        _ => (chroma, 0.0, x),
    };
    let m = light - chroma / 2.0;
    (r1 + m, g1 + m, b1 + m)
}

fn parse_hsl_function(body: &str) -> Option<Rgba> {
    let parts = split_function_args(body);
    if parts.len() < 3 || parts.len() > 4 {
        return None;
    }
    let h = parse_hue(&parts[0])?;
    let s = parse_percentage(&parts[1])?;
    let l = parse_percentage(&parts[2])?;
    let a = if parts.len() == 4 {
        parse_alpha(&parts[3])?
    } else {
        1.0
    };
    let (r, g, b) = hsl_to_rgb(h, s, l);
    Some(Rgba { r, g, b, a })
}

fn clamp01(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

pub fn parse_css_color(input: &str) -> Option<Rgba> {
    let text = input.trim();
    if let Some(hex) = text.strip_prefix('#') {
        return parse_hex_color(hex);
    }
    let lowered = text.to_lowercase();
    let function_body = |names: &[&str]| -> Option<&str> {
        for name in names {
            if let Some(rest) = lowered.strip_prefix(name) {
                return rest.strip_suffix(')');
            }
        }
        None
    };
    let parsed = if let Some(body) = function_body(&["rgba(", "rgb("]) {
        parse_rgb_function(body)
    } else if let Some(body) = function_body(&["hsla(", "hsl("]) {
        parse_hsl_function(body)
    } else {
        named_color(&lowered)
    }?;
    Some(Rgba {
        r: clamp01(parsed.r),
        g: clamp01(parsed.g),
        b: clamp01(parsed.b),
        a: clamp01(parsed.a),
    })
}

fn is_digits_and_dots(text: &str) -> bool {
    !text.is_empty() && text.chars().all(|c| c.is_ascii_digit() || c == '.')
}

pub fn is_transparent(color: &str) -> bool {
    let lower = color.to_lowercase();
    let trimmed = lower.trim();
    if trimmed == "transparent" {
        return true;
    }
    let Some(body) = trimmed
        .strip_prefix("rgba(")
        .and_then(|rest| rest.strip_suffix(')'))
    else {
        return false;
    };
    let parts: Vec<&str> = body.split(',').map(str::trim).collect();
    if parts.len() != 4 {
        return false;
    }
    if !parts[..3].iter().all(|part| is_digits_and_dots(part)) {
        return false;
    }
    if !is_digits_and_dots(parts[3]) {
        return false;
    }
    parts[3].parse::<f64>().is_ok_and(|alpha| alpha == 0.0)
}

pub fn parse_color_to_rgb(color: &str) -> Option<Rgba> {
    let lower = color.to_lowercase();
    let trimmed = lower.trim();
    if let Some(hex) = trimmed.strip_prefix('#') {
        if hex.len() < 3 || hex.len() > 8 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
            return None;
        }
        match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
                let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
                let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
                return Some(Rgba {
                    r: f64::from(r),
                    g: f64::from(g),
                    b: f64::from(b),
                    a: 1.0,
                });
            }
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                return Some(Rgba {
                    r: f64::from(r),
                    g: f64::from(g),
                    b: f64::from(b),
                    a: 1.0,
                });
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                return Some(Rgba {
                    r: f64::from(r),
                    g: f64::from(g),
                    b: f64::from(b),
                    a: f64::from(a) / 255.0,
                });
            }
            _ => return None,
        }
    }
    if let Some(body) = trimmed
        .strip_prefix("rgb(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        let parts: Vec<&str> = body.split(',').map(str::trim).collect();
        if parts.len() == 3 && parts.iter().all(|part| is_digits_and_dots(part)) {
            let r = parts[0].parse::<f64>().ok()?;
            let g = parts[1].parse::<f64>().ok()?;
            let b = parts[2].parse::<f64>().ok()?;
            return Some(Rgba { r, g, b, a: 1.0 });
        }
        return None;
    }
    if let Some(body) = trimmed
        .strip_prefix("rgba(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        let parts: Vec<&str> = body.split(',').map(str::trim).collect();
        if parts.len() == 4 && parts.iter().all(|part| is_digits_and_dots(part)) {
            let r = parts[0].parse::<f64>().ok()?;
            let g = parts[1].parse::<f64>().ok()?;
            let b = parts[2].parse::<f64>().ok()?;
            let a = parts[3].parse::<f64>().ok()?;
            return Some(Rgba { r, g, b, a });
        }
        return None;
    }
    None
}
