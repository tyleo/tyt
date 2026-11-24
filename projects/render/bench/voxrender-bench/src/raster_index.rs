use ty_math::TyVector3U32;

/// The place of `at` in the raster order of a grid of `counts`:
/// `(x * counts.y + y) * counts.z + z`.
pub fn raster_index(at: TyVector3U32, counts: TyVector3U32) -> usize {
    let [x, y, z] = at.to_array().map(|side| side as usize);

    (x * counts.y as usize + y) * counts.z as usize + z
}
