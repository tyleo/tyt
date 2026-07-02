/// The near-square pixel dimensions of a palette atlas of `count` texels: width
/// `ceil(sqrt(count))`, height `ceil(count / width)`, both at least one. The
/// layout is a pure function of the count, so the image bake and the UVs that
/// sample it agree on where each texel sits.
pub(crate) fn atlas_dimensions(count: usize) -> (u32, u32) {
    let count = count.max(1) as u32;

    let width = ((count as f64).sqrt().ceil() as u32).max(1);

    let height = count.div_ceil(width).max(1);

    (width, height)
}

/// The UV at the center of texel `index` in a `width` x `height` atlas, so a
/// face sampling it with nearest filtering reads exactly that texel. The layout
/// matches [`atlas_dimensions`], row-major from the top-left, glTF's UV origin.
pub(crate) fn texel_center(index: u32, width: u32, height: u32) -> [f32; 2] {
    let column = index % width;
    let row = index / width;

    [
        (column as f32 + 0.5) / width as f32,
        (row as f32 + 0.5) / height as f32,
    ]
}

#[cfg(test)]
mod tests {
    use crate::atlas_dimensions;

    #[test]
    fn packs_texels_near_square_with_room_for_all() {
        // (count, expected width, expected height); width*height covers count.
        for (count, width, height) in [
            (0, 1, 1),
            (1, 1, 1),
            (2, 2, 1),
            (3, 2, 2),
            (4, 2, 2),
            (5, 3, 2),
            (2040, 46, 45),
        ] {
            assert_eq!(atlas_dimensions(count), (width, height), "count {count}");
            assert!((width * height) as usize >= count.max(1));
        }
    }
}
