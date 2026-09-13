use crate::{PALETTE_COLORS, PLACEHOLDER_COLOR};
use vmax::{VMaxFile, VMaxObject};

/// The 0-based RGBA color table for an object. The `palette*.png` pixels when
/// present (its trailing transparent terminator dropped), else the material
/// sidecar's packed `colors` table, and finally a uniform placeholder so color
/// indices are still preserved.
pub(crate) fn color_cells(serde: &VMaxFile, object: &VMaxObject) -> Vec<[u8; 4]> {
    if let Some(png) = serde.palette_png_files.get(&object.palette) {
        return png.0.iter().take(PALETTE_COLORS).copied().collect();
    }
    if let Some(stem) = object.palette.strip_suffix(".png") {
        let sidecar = format!("{stem}.settings.vmaxpsb");
        if let Some(palette) = serde.palette_settings_files.get(&sidecar)
            && !palette.colors.is_empty()
        {
            // The sidecar stores colors packed (4 bytes per cell); unpack them.
            return palette
                .colors
                .chunks_exact(4)
                .map(|c| [c[0], c[1], c[2], c[3]])
                .collect();
        }
    }
    (0..PALETTE_COLORS).map(|_| PLACEHOLDER_COLOR).collect()
}
