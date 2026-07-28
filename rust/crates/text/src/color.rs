#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const TRANSPARENT: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 0xFF }
    }

    pub fn parse(value: &str) -> Option<Self> {
        if value == "transparent" {
            return Some(Self::TRANSPARENT);
        }
        let hex = value.strip_prefix('#')?;
        let channel = |pair: &str| u8::from_str_radix(pair, 16).ok();
        match hex.len() {
            3 => {
                let r = channel(&hex[0..1].repeat(2))?;
                let g = channel(&hex[1..2].repeat(2))?;
                let b = channel(&hex[2..3].repeat(2))?;
                Some(Self::rgb(r, g, b))
            }
            4 => {
                let r = channel(&hex[0..1].repeat(2))?;
                let g = channel(&hex[1..2].repeat(2))?;
                let b = channel(&hex[2..3].repeat(2))?;
                let a = channel(&hex[3..4].repeat(2))?;
                Some(Self::rgba(r, g, b, a))
            }
            6 => {
                let r = channel(&hex[0..2])?;
                let g = channel(&hex[2..4])?;
                let b = channel(&hex[4..6])?;
                Some(Self::rgb(r, g, b))
            }
            8 => {
                let r = channel(&hex[0..2])?;
                let g = channel(&hex[2..4])?;
                let b = channel(&hex[4..6])?;
                let a = channel(&hex[6..8])?;
                Some(Self::rgba(r, g, b, a))
            }
            _ => None,
        }
    }

    pub(crate) fn to_cosmic(self) -> cosmic_text::Color {
        cosmic_text::Color::rgba(self.r, self.g, self.b, self.a)
    }

    pub(crate) fn from_cosmic(color: cosmic_text::Color) -> Self {
        Self::rgba(color.r(), color.g(), color.b(), color.a())
    }

    pub(crate) fn scaled_alpha(self, coverage: f64) -> Self {
        let alpha = (f64::from(self.a) * coverage).round() as u8;
        Self::rgba(self.r, self.g, self.b, alpha)
    }
}
