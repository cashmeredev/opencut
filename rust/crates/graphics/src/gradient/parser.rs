use thiserror::Error;

use super::ast::{
    Color, ColorStop, Distance, GradientAst, GradientKind, GradientOrientation,
    LinearOrientation, Position, RadialOrientation, Shape, ShapeKind, ShapeStyle,
};

#[derive(Debug, Clone, Error, PartialEq)]
#[error("{input}: {message}")]
pub struct GradientParseError {
    pub input: String,
    pub message: String,
}

type PResult<T> = Result<T, GradientParseError>;

type TokenMatch = (usize, usize, usize);

fn match_ci_prefix(s: &str, keyword: &str) -> Option<usize> {
    if s.len() >= keyword.len() && s[..keyword.len()].eq_ignore_ascii_case(keyword) {
        Some(keyword.len())
    } else {
        None
    }
}

fn scan_number_str(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let mut index = 0;
    while index < bytes.len() && bytes[index].is_ascii_digit() {
        index += 1;
    }
    let int_len = index;
    if index < bytes.len() && bytes[index] == b'.' {
        index += 1;
        let frac_start = index;
        while index < bytes.len() && bytes[index].is_ascii_digit() {
            index += 1;
        }
        if index > frac_start {
            return Some(index);
        }
        if int_len > 0 {
            return Some(frac_start);
        }
        return None;
    }
    if int_len > 0 { Some(int_len) } else { None }
}

fn scan_signed_number_unit(s: &str, unit: &str) -> Option<TokenMatch> {
    let unsigned = s.strip_prefix('-').unwrap_or(s);
    let sign_len = s.len() - unsigned.len();
    let number_len = scan_number_str(unsigned)?;
    if unsigned[number_len..].starts_with(unit) {
        Some((sign_len + number_len + unit.len(), 0, sign_len + number_len))
    } else {
        None
    }
}

fn whole(len: usize) -> Option<TokenMatch> {
    Some((len, 0, len))
}

fn scan_percentage(s: &str) -> Option<TokenMatch> {
    scan_signed_number_unit(s, "%")
}

fn scan_px(s: &str) -> Option<TokenMatch> {
    scan_signed_number_unit(s, "px")
}

fn scan_em(s: &str) -> Option<TokenMatch> {
    scan_signed_number_unit(s, "em")
}

fn scan_deg(s: &str) -> Option<TokenMatch> {
    scan_signed_number_unit(s, "deg")
}

fn scan_rad(s: &str) -> Option<TokenMatch> {
    scan_signed_number_unit(s, "rad")
}

fn scan_number_token(s: &str) -> Option<TokenMatch> {
    scan_number_str(s).and_then(whole)
}

fn scan_hex_color(s: &str) -> Option<TokenMatch> {
    let rest = s.strip_prefix('#')?;
    let digits_len = rest.find(|c: char| !c.is_ascii_hexdigit()).unwrap_or(rest.len());
    if digits_len == 0 {
        return None;
    }
    Some((1 + digits_len, 1, 1 + digits_len))
}

fn scan_literal_color(s: &str) -> Option<TokenMatch> {
    let len = s.find(|c: char| !c.is_ascii_alphabetic()).unwrap_or(s.len());
    if len == 0 {
        return None;
    }
    whole(len)
}

fn scan_variable_name(s: &str) -> Option<TokenMatch> {
    let rest = s.strip_prefix("--")?;
    let len = rest
        .find(|c: char| {
            !(c.is_ascii_alphanumeric() || c == '-' || c == ',' || c == '#' || c.is_whitespace())
        })
        .unwrap_or(rest.len());
    if len == 0 {
        return None;
    }
    whole(2 + len)
}

fn scan_position_keyword(s: &str) -> Option<TokenMatch> {
    for keyword in ["left", "center", "right", "top", "bottom"] {
        if let Some(len) = match_ci_prefix(s, keyword) {
            return whole(len);
        }
    }
    None
}

fn scan_extent_keyword(s: &str) -> Option<TokenMatch> {
    for keyword in [
        "closest-side",
        "closest-corner",
        "farthest-side",
        "farthest-corner",
        "contain",
        "cover",
    ] {
        if s.starts_with(keyword) {
            return whole(keyword.len());
        }
    }
    None
}

fn scan_side_or_corner(s: &str) -> Option<TokenMatch> {
    match_ci_prefix(s, "to ")?;
    let rest = &s[3..];
    let pairs: [(&str, &[&str]); 4] = [
        ("left", &["top", "bottom"]),
        ("right", &["top", "bottom"]),
        ("top", &["left", "right"]),
        ("bottom", &["left", "right"]),
    ];
    for (first, seconds) in pairs {
        if let Some(first_len) = match_ci_prefix(rest, first) {
            let after_first = &rest[first_len..];
            for second in seconds {
                let with_space = format!(" {second}");
                if let Some(second_len) = match_ci_prefix(after_first, &with_space) {
                    let end = 3 + first_len + second_len;
                    return Some((end, 3, end));
                }
            }
        }
    }
    for keyword in ["left", "right", "top", "bottom"] {
        if let Some(len) = match_ci_prefix(rest, keyword) {
            let end = 3 + len;
            return Some((end, 3, end));
        }
    }
    None
}

fn gradient_prefix(s: &str, name: &str) -> Option<TokenMatch> {
    let mut offset = 0;
    if s.starts_with('-') {
        for vendor in ["webkit", "o", "ms", "moz"] {
            let prefix = format!("-{vendor}-");
            if let Some(len) = match_ci_prefix(s, &prefix) {
                offset = len;
                break;
            }
        }
        if offset == 0 {
            return None;
        }
    }
    let name_len = match_ci_prefix(&s[offset..], name)?;
    whole(offset + name_len)
}

struct Parser<'a> {
    rest: &'a str,
}

impl<'a> Parser<'a> {
    fn error(&self, message: &str) -> GradientParseError {
        GradientParseError {
            input: self.rest.to_string(),
            message: message.to_string(),
        }
    }

    fn skip_whitespace(&mut self) {
        self.rest = self.rest.trim_start();
    }

    fn consume(&mut self, size: usize) {
        self.rest = &self.rest[size..];
    }

    fn scan_token(&mut self, pattern: impl Fn(&str) -> Option<TokenMatch>) -> Option<String> {
        self.skip_whitespace();
        let (total, capture_start, capture_end) = pattern(self.rest)?;
        let capture = self.rest[capture_start..capture_end].to_string();
        self.consume(total);
        Some(capture)
    }

    fn scan_char(&mut self, target: u8) -> bool {
        self.skip_whitespace();
        if self.rest.as_bytes().first() == Some(&target) {
            self.consume(1);
            return true;
        }
        false
    }

    fn scan_comma(&mut self) -> bool {
        self.scan_char(b',')
    }

    fn match_call<T>(
        &mut self,
        pattern: impl Fn(&str) -> Option<TokenMatch>,
        callback: impl FnOnce(&mut Self) -> PResult<T>,
    ) -> PResult<Option<T>> {
        self.skip_whitespace();
        let Some((total, _, _)) = pattern(self.rest) else {
            return Ok(None);
        };
        self.consume(total);
        if !self.scan_char(b'(') {
            return Err(self.error("Missing ("));
        }
        let result = callback(self)?;
        if !self.scan_char(b')') {
            return Err(self.error("Missing )"));
        }
        Ok(Some(result))
    }

    fn match_listing<T>(
        &mut self,
        matcher: fn(&mut Self) -> PResult<Option<T>>,
    ) -> PResult<Vec<T>> {
        let mut result = Vec::new();
        let Some(first) = matcher(self)? else {
            return Ok(result);
        };
        result.push(first);
        while self.scan_comma() {
            let next = matcher(self)?.ok_or_else(|| self.error("One extra comma"))?;
            result.push(next);
        }
        Ok(result)
    }

    fn match_definition(&mut self) -> PResult<Option<GradientAst>> {
        for (kind, name) in [
            (GradientKind::Linear, "linear-gradient"),
            (GradientKind::RepeatingLinear, "repeating-linear-gradient"),
            (GradientKind::Radial, "radial-gradient"),
            (GradientKind::RepeatingRadial, "repeating-radial-gradient"),
        ] {
            if let Some(ast) = self.match_gradient(kind, name)? {
                return Ok(Some(ast));
            }
        }
        Ok(None)
    }

    fn match_gradient(
        &mut self,
        kind: GradientKind,
        name: &'static str,
    ) -> PResult<Option<GradientAst>> {
        self.match_call(
            |s| gradient_prefix(s, name),
            |parser| {
                let orientation = if kind.is_linear() {
                    parser.match_linear_orientation()?
                } else {
                    parser.match_list_radial_orientations()?
                };
                if orientation.is_some() && !parser.scan_comma() {
                    return Err(parser.error("Missing comma before color stops"));
                }
                Ok(GradientAst {
                    kind,
                    orientation,
                    color_stops: parser.match_listing(Self::match_color_stop)?,
                })
            },
        )
    }

    fn match_linear_orientation(&mut self) -> PResult<Option<GradientOrientation>> {
        if let Some(value) = self.scan_token(scan_side_or_corner) {
            return Ok(Some(GradientOrientation::Linear(
                LinearOrientation::Directional(value),
            )));
        }
        if let Some(value) = self.scan_token(scan_position_keyword) {
            return Ok(Some(GradientOrientation::Linear(
                LinearOrientation::Directional(value),
            )));
        }
        let angular = self
            .scan_token(scan_deg)
            .or_else(|| self.scan_token(scan_rad));
        Ok(angular.map(|value| GradientOrientation::Linear(LinearOrientation::Angular(value))))
    }

    fn match_list_radial_orientations(&mut self) -> PResult<Option<GradientOrientation>> {
        let Some(first) = self.match_radial_orientation()? else {
            return Ok(None);
        };
        let mut orientations = vec![first];
        let lookahead = self.rest;
        if !self.scan_comma() {
            return Ok(Some(GradientOrientation::Radial(orientations)));
        }
        let Some(next) = self.match_radial_orientation()? else {
            self.rest = lookahead;
            return Ok(Some(GradientOrientation::Radial(orientations)));
        };
        orientations.push(next);
        Ok(Some(GradientOrientation::Radial(orientations)))
    }

    fn match_radial_orientation(&mut self) -> PResult<Option<RadialOrientation>> {
        let shape = self.match_circle()?.or(self.match_ellipse()?);
        if let Some(mut shape) = shape {
            shape.at = self.match_at_position()?;
            return Ok(Some(RadialOrientation::Shape(shape)));
        }

        if let Some(value) = self.scan_token(scan_extent_keyword) {
            let at = self.match_at_position()?;
            return Ok(Some(RadialOrientation::ExtentKeyword { value, at }));
        }

        if let Some(implicit) = self.match_implicit_ellipse()? {
            return Ok(Some(RadialOrientation::Shape(implicit)));
        }

        if let Some(at) = self.match_at_position()? {
            return Ok(Some(RadialOrientation::DefaultRadial { at }));
        }

        if let Some(at) = self.match_positioning()? {
            return Ok(Some(RadialOrientation::DefaultRadial { at }));
        }

        Ok(None)
    }

    fn match_implicit_ellipse(&mut self) -> PResult<Option<Shape>> {
        let lookahead = self.rest;
        let Some(width) = self.match_distance()? else {
            return Ok(None);
        };
        let Some(height) = self.match_distance()? else {
            self.rest = lookahead;
            return Ok(None);
        };
        let Some(at) = self.match_at_position()? else {
            self.rest = lookahead;
            return Ok(None);
        };
        Ok(Some(Shape {
            value: ShapeKind::Ellipse,
            style: Some(ShapeStyle::Position(Position {
                x: Some(width),
                y: Some(height),
            })),
            at: Some(at),
        }))
    }

    fn match_circle(&mut self) -> PResult<Option<Shape>> {
        if self.scan_token(|s| match_ci_prefix(s, "circle").and_then(whole)).is_none() {
            return Ok(None);
        }
        let style = self
            .match_length()
            .map(ShapeStyle::Distance)
            .or_else(|| self.scan_token(scan_extent_keyword).map(ShapeStyle::ExtentKeyword));
        Ok(Some(Shape {
            value: ShapeKind::Circle,
            style,
            at: None,
        }))
    }

    fn match_ellipse(&mut self) -> PResult<Option<Shape>> {
        if self.scan_token(|s| match_ci_prefix(s, "ellipse").and_then(whole)).is_none() {
            return Ok(None);
        }
        let style = self
            .match_positioning()?
            .map(ShapeStyle::Position)
            .or_else(|| self.match_distance().ok().flatten().map(ShapeStyle::Distance))
            .or_else(|| self.scan_token(scan_extent_keyword).map(ShapeStyle::ExtentKeyword));
        Ok(Some(Shape {
            value: ShapeKind::Ellipse,
            style,
            at: None,
        }))
    }

    fn match_at_position(&mut self) -> PResult<Option<Position>> {
        if self.scan_token(|s| match_ci_prefix(s, "at").and_then(whole)).is_none() {
            return Ok(None);
        }
        let Some(position) = self.match_positioning()? else {
            return Err(self.error("Missing positioning value"));
        };
        Ok(Some(position))
    }

    fn match_positioning(&mut self) -> PResult<Option<Position>> {
        let position = Position {
            x: self.match_distance()?,
            y: self.match_distance()?,
        };
        if position.x.is_none() && position.y.is_none() {
            return Ok(None);
        }
        Ok(Some(position))
    }

    fn match_color_stop(&mut self) -> PResult<Option<ColorStop>> {
        let color = self
            .match_color()?
            .ok_or_else(|| self.error("Expected color definition"))?;
        let length = self.match_distance()?;
        Ok(Some(ColorStop { color, length }))
    }

    fn match_color(&mut self) -> PResult<Option<Color>> {
        if let Some(hex) = self.scan_token(scan_hex_color) {
            return Ok(Some(Color::Hex(hex)));
        }
        if let Some(hsla) = self.match_hsla_color()? {
            return Ok(Some(hsla));
        }
        if let Some(hsl) = self.match_hsl_color()? {
            return Ok(Some(hsl));
        }
        if let Some(rgba) = self.match_rgba_color()? {
            return Ok(Some(rgba));
        }
        if let Some(rgb) = self.match_rgb_color()? {
            return Ok(Some(rgb));
        }
        if let Some(var) = self.match_var_color()? {
            return Ok(Some(var));
        }
        Ok(self.scan_token(scan_literal_color).map(Color::Literal))
    }

    fn match_rgb_color(&mut self) -> PResult<Option<Color>> {
        self.match_call(
            |s| match_ci_prefix(s, "rgb").and_then(whole),
            |parser| {
                let values = parser.match_listing(|p| p.match_number().map(Some))?;
                Ok(Color::Rgb(values))
            },
        )
    }

    fn match_rgba_color(&mut self) -> PResult<Option<Color>> {
        self.match_call(
            |s| match_ci_prefix(s, "rgba").and_then(whole),
            |parser| {
                let values = parser.match_listing(|p| p.match_number().map(Some))?;
                Ok(Color::Rgba(values))
            },
        )
    }

    fn match_var_color(&mut self) -> PResult<Option<Color>> {
        self.match_call(
            |s| match_ci_prefix(s, "var").and_then(whole),
            |parser| {
                let name = parser
                    .scan_token(scan_variable_name)
                    .ok_or_else(|| parser.error("Expected CSS variable name"))?;
                Ok(Color::Var(name))
            },
        )
    }

    fn match_hsl_color(&mut self) -> PResult<Option<Color>> {
        self.match_call(
            |s| match_ci_prefix(s, "hsl").and_then(whole),
            |parser| {
                if parser.scan_token(scan_percentage).is_some() {
                    return Err(parser.error(
                        "HSL hue value must be a number in degrees (0-360) or normalized (-360 to 360), not a percentage",
                    ));
                }
                let hue = parser.match_number()?;
                parser.scan_comma();
                let sat = parser.scan_token(scan_percentage);
                parser.scan_comma();
                let light = parser.scan_token(scan_percentage);
                let sat = sat.ok_or_else(|| {
                    parser.error("Expected percentage value for saturation and lightness in HSL")
                })?;
                let light = light.ok_or_else(|| {
                    parser.error("Expected percentage value for saturation and lightness in HSL")
                })?;
                Ok(Color::Hsl([hue, sat, light]))
            },
        )
    }

    fn match_hsla_color(&mut self) -> PResult<Option<Color>> {
        self.match_call(
            |s| match_ci_prefix(s, "hsla").and_then(whole),
            |parser| {
                let hue = parser.match_number()?;
                parser.scan_comma();
                let sat = parser.scan_token(scan_percentage);
                parser.scan_comma();
                let light = parser.scan_token(scan_percentage);
                parser.scan_comma();
                let alpha = parser.match_number()?;
                let sat = sat.ok_or_else(|| {
                    parser.error("Expected percentage value for saturation and lightness in HSLA")
                })?;
                let light = light.ok_or_else(|| {
                    parser.error("Expected percentage value for saturation and lightness in HSLA")
                })?;
                Ok(Color::Hsla([hue, sat, light, alpha]))
            },
        )
    }

    fn match_number(&mut self) -> PResult<String> {
        self.scan_token(scan_number_token)
            .ok_or_else(|| self.error("Expected number"))
    }

    fn match_distance(&mut self) -> PResult<Option<Distance>> {
        if let Some(value) = self.scan_token(scan_percentage) {
            return Ok(Some(Distance::Percent(value)));
        }
        if let Some(value) = self.scan_token(scan_position_keyword) {
            return Ok(Some(Distance::PositionKeyword(value)));
        }
        if let Some(calc) = self.match_calc()? {
            return Ok(Some(calc));
        }
        Ok(self.match_length())
    }

    fn match_calc(&mut self) -> PResult<Option<Distance>> {
        self.match_call(
            |s| match_ci_prefix(s, "calc").and_then(whole),
            |parser| {
                let mut open_parens = 1i32;
                let mut index = 0usize;
                let bytes = parser.rest.as_bytes();
                while open_parens > 0 && index < bytes.len() {
                    match bytes[index] {
                        b'(' => open_parens += 1,
                        b')' => open_parens -= 1,
                        _ => {}
                    }
                    index += 1;
                }
                if open_parens > 0 {
                    return Err(parser.error("Missing closing parenthesis in calc() expression"));
                }
                let content = parser.rest[..index - 1].to_string();
                parser.consume(index - 1);
                Ok(Distance::Calc(content))
            },
        )
    }

    fn match_length(&mut self) -> Option<Distance> {
        if let Some(value) = self.scan_token(scan_px) {
            return Some(Distance::Px(value));
        }
        self.scan_token(scan_em).map(Distance::Em)
    }
}

pub fn parse_gradient(code: &str) -> Result<Vec<GradientAst>, GradientParseError> {
    let trimmed = code.trim();
    let source = trimmed.strip_suffix(';').unwrap_or(trimmed);
    let mut parser = Parser { rest: source };
    let ast = parser.match_listing(Parser::match_definition)?;
    if !parser.rest.is_empty() {
        return Err(parser.error("Invalid input not EOF"));
    }
    Ok(ast)
}
