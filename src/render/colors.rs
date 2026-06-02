//! ANSI 24-bit colors for bodies and trails.

use crate::physics::body::{Body, BodyClass};

/// Parse `#rrggbb` or `rrggbb` into RGB bytes.
pub fn parse_hex_color(hex: &str) -> Option<[u8; 3]> {
    let s = hex.trim().trim_start_matches('#');
    if s.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some([r, g, b])
}

/// Resolve display color for a body (scenario override, then name, then class).
#[must_use]
pub fn body_color(body: &Body) -> [u8; 3] {
    if let Some(rgb) = body.color_rgb {
        return rgb;
    }
    if let Some(rgb) = color_by_name(&body.name) {
        return rgb;
    }
    color_by_class(body.class)
}

fn color_by_name(name: &str) -> Option<[u8; 3]> {
    if name.eq_ignore_ascii_case("sun") {
        return Some(hex("fff4ea"));
    }
    if name.eq_ignore_ascii_case("mercury") {
        return Some(hex("8c7f70"));
    }
    if name.eq_ignore_ascii_case("venus") {
        return Some(hex("d9c27f"));
    }
    if name.eq_ignore_ascii_case("earth") {
        return Some(hex("3d8bfd"));
    }
    if name.eq_ignore_ascii_case("moon") {
        return Some(hex("c0c0c0"));
    }
    if name.eq_ignore_ascii_case("mars") {
        return Some(hex("c1440e"));
    }
    if name.eq_ignore_ascii_case("jupiter") {
        return Some(hex("d8a45f"));
    }
    if name.eq_ignore_ascii_case("saturn") {
        return Some(hex("e3c16f"));
    }
    if name.eq_ignore_ascii_case("uranus") {
        return Some(hex("7fdbff"));
    }
    if name.eq_ignore_ascii_case("neptune") {
        return Some(hex("4169e1"));
    }
    if name.eq_ignore_ascii_case("pluto") {
        return Some(hex("b89b72"));
    }
    None
}

fn color_by_class(class: BodyClass) -> [u8; 3] {
    match class {
        BodyClass::Star => hex("fff4ea"),
        BodyClass::Planet => hex("3d8bfd"),
        BodyClass::Moon => hex("c0c0c0"),
        BodyClass::DwarfPlanet => hex("b89b72"),
        BodyClass::Asteroid => hex("8c7f70"),
        BodyClass::Comet => hex("7fdbff"),
        BodyClass::Spacecraft => hex("51cf66"),
        BodyClass::Custom => hex("ff6b6b"),
    }
}

/// Fade trail brightness by age (0 = oldest, 1 = newest).
#[must_use]
pub fn trail_color(base: [u8; 3], age_fraction: f64) -> [u8; 3] {
    let t = age_fraction.clamp(0.0, 1.0);
    let dim = 0.25 + 0.75 * t;
    [
        (f64::from(base[0]) * dim) as u8,
        (f64::from(base[1]) * dim) as u8,
        (f64::from(base[2]) * dim) as u8,
    ]
}

const fn hex(s: &str) -> [u8; 3] {
    let bytes = s.as_bytes();
    [
        from_hex_pair(bytes[0], bytes[1]),
        from_hex_pair(bytes[2], bytes[3]),
        from_hex_pair(bytes[4], bytes[5]),
    ]
}

const fn from_hex_pair(h: u8, l: u8) -> u8 {
    (hex_digit(h) << 4) | hex_digit(l)
}

const fn hex_digit(c: u8) -> u8 {
    match c {
        b'0'..=b'9' => c - b'0',
        b'a'..=b'f' => c - b'a' + 10,
        b'A'..=b'F' => c - b'A' + 10,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hex_color() {
        assert_eq!(parse_hex_color("#3d8bfd"), Some([0x3d, 0x8b, 0xfd]));
    }
}
