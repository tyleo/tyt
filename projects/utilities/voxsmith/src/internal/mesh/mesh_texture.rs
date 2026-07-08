use crate::MeshWrap;

/// A decoded texture image: `width` by `height` RGBA8 texels in row-major order,
/// row `0` at the top to match the glTF texture-coordinate origin. The store is
/// color-space-neutral raw bytes; the sample site decodes sRGB (base color,
/// emissive) or straight linear data (metallic-roughness, occlusion) per the map
/// it feeds, so one image feeding maps of different kinds decodes correctly for
/// each.
pub(crate) struct MeshTexture {
    width: u32,
    height: u32,
    texels: Vec<[u8; 4]>,
}

impl MeshTexture {
    /// Builds a texture from its dimensions and row-major texels.
    pub fn new(width: u32, height: u32, texels: Vec<[u8; 4]>) -> Self {
        Self {
            width,
            height,
            texels,
        }
    }

    /// The nearest-neighbor texel at texture coordinate `(u, v)` under the given
    /// wrap modes, `v` running top-down. A zero-size image yields opaque white.
    pub fn sample(&self, u: f64, v: f64, wrap_s: MeshWrap, wrap_t: MeshWrap) -> [u8; 4] {
        if self.texels.is_empty() {
            return [255, 255, 255, 255];
        }

        let x = wrap_s.texel(u, self.width);
        let y = wrap_t.texel(v, self.height);

        self.texels[y * self.width as usize + x]
    }
}
