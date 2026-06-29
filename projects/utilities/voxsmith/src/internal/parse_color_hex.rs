use voxcore::VoxValue;

/// Parses a `#RRGGBB` or `#RRGGBBAA` color string into RGBA bytes. A missing
/// alpha defaults to opaque; a missing or malformed value defaults to
/// transparent black.
pub fn parse_color_hex(value: Option<&VoxValue>) -> [u8; 4] {
    let Some(VoxValue::Text(hex)) = value else {
        return [0, 0, 0, 0];
    };
    let hex = hex.strip_prefix('#').unwrap_or(hex);
    let byte = |index: usize| {
        hex.get(index * 2..index * 2 + 2)
            .and_then(|byte| u8::from_str_radix(byte, 16).ok())
    };
    match (byte(0), byte(1), byte(2)) {
        (Some(r), Some(g), Some(b)) => [r, g, b, byte(3).unwrap_or(255)],
        _ => [0, 0, 0, 0],
    }
}
