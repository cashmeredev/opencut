pub(crate) fn js_max(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() { f64::NAN } else { a.max(b) }
}

pub(crate) fn js_min(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() { f64::NAN } else { a.min(b) }
}

pub(crate) fn js_parse_float(value: &str) -> f64 {
    let trimmed = value.trim_start();
    let (sign, rest) = match trimmed.strip_prefix('-') {
        Some(rest) => (-1.0, rest),
        None => (1.0, trimmed.strip_prefix('+').unwrap_or(trimmed)),
    };
    if rest.starts_with("Infinity") {
        return sign * f64::INFINITY;
    }
    let bytes = rest.as_bytes();
    let mut end = 0;
    let mut seen_digit = false;
    let mut seen_dot = false;
    while end < bytes.len() {
        match bytes[end] {
            b'0'..=b'9' => {
                seen_digit = true;
                end += 1;
            }
            b'.' if !seen_dot => {
                seen_dot = true;
                end += 1;
            }
            _ => break,
        }
    }
    if !seen_digit {
        return f64::NAN;
    }
    let mantissa_end = end;
    if end < bytes.len() && (bytes[end] == b'e' || bytes[end] == b'E') {
        let mut exponent_end = end + 1;
        if exponent_end < bytes.len()
            && (bytes[exponent_end] == b'+' || bytes[exponent_end] == b'-')
        {
            exponent_end += 1;
        }
        let digits_start = exponent_end;
        while exponent_end < bytes.len() && bytes[exponent_end].is_ascii_digit() {
            exponent_end += 1;
        }
        if exponent_end > digits_start {
            end = exponent_end;
        } else {
            end = mantissa_end;
        }
    }
    sign * rest[..end].parse::<f64>().unwrap_or(f64::NAN)
}
