/// A texture wrap mode: how a texture coordinate outside `[0, 1]` maps back onto
/// the image.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum MeshWrap {
    /// Tile the image (the fractional part of the coordinate).
    Repeat,

    /// Clamp to the nearest edge texel.
    Clamp,

    /// Tile the image, flipping every other repeat.
    Mirror,
}

impl MeshWrap {
    /// The nearest-neighbor texel index for normalized `coord` along an axis of
    /// `size` texels.
    pub fn texel(self, coord: f64, size: u32) -> usize {
        let last = size as i64 - 1;
        let index = (coord * size as f64).floor() as i64;

        let wrapped = match self {
            MeshWrap::Repeat => index.rem_euclid(size as i64),
            MeshWrap::Clamp => index.clamp(0, last),
            MeshWrap::Mirror => {
                let period = 2 * size as i64;
                let folded = index.rem_euclid(period);
                if folded < size as i64 {
                    folded
                } else {
                    period - 1 - folded
                }
            }
        };

        wrapped as usize
    }
}
