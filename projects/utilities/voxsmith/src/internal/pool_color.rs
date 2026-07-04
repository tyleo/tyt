use ty_math::TyLinearRgbaColorF64;
use voxcore::VoxValuePool;

/// Decodes the color at `index` in a color `pool` to sRGB `[r, g, b, a]` bytes,
/// or `None` when `index` is out of range or `pool` is not a color kind. An
/// sRGB pool's components map straight to bytes; a linear pool's re-encode to
/// sRGB. A three-component color takes opaque alpha.
pub fn pool_color(pool: &VoxValuePool, index: u32) -> Option<[u8; 4]> {
    let index = index as usize;

    match pool {
        VoxValuePool::Srgb { values } => values
            .get(index)
            .map(|&[r, g, b]| srgb_bytes([r, g, b, 1.0])),
        VoxValuePool::Srgba { values } => values.get(index).map(|&color| srgb_bytes(color)),
        VoxValuePool::LinearRgb { values } => values.get(index).map(|&[r, g, b]| {
            TyLinearRgbaColorF64::new(r, g, b, 1.0)
                .to_srgba()
                .to_array()
        }),
        VoxValuePool::LinearRgba { values } => values
            .get(index)
            .map(|&[r, g, b, a]| TyLinearRgbaColorF64::new(r, g, b, a).to_srgba().to_array()),
        _ => None,
    }
}

/// Maps sRGB-encoded float components in `[0, 1]` to bytes.
fn srgb_bytes(color: [f64; 4]) -> [u8; 4] {
    color.map(|component| (component.clamp(0.0, 1.0) * 255.0).round() as u8)
}
