use crate::{AtlasShape, Error, Result};

/// The pixel dimensions of a palette atlas of `count` texels under `shape`. The
/// texels fill the top-left in row-major order and any extra area is padding.
/// The layout is a pure function of the count and shape, so the image bake and
/// the UVs that sample it agree on where each texel sits. Errors when an
/// [`AtlasShape::Exact`] side is too small to hold the texels.
pub(crate) fn atlas_dimensions(count: usize, shape: AtlasShape) -> Result<(u32, u32)> {
    let count = count.max(1) as u32;

    // The near-square width, the fewest columns that keep the packing no taller
    // than it is wide.
    let base_width = ((count as f64).sqrt().ceil() as u32).max(1);

    match shape {
        AtlasShape::Line => Ok((count, 1)),

        AtlasShape::Fit => Ok((base_width, count.div_ceil(base_width).max(1))),

        AtlasShape::Square => Ok((base_width, base_width)),

        AtlasShape::Pot => {
            let side = base_width.next_power_of_two();
            Ok((side, side))
        }

        AtlasShape::Exact(side) => {
            if u64::from(side) * u64::from(side) < u64::from(count) {
                return Err(Error::invalid(format!(
                    "a {side}x{side} atlas is too small for {count} materials; the \
                     smallest square that fits is {base_width}"
                )));
            }

            Ok((side, side))
        }
    }
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
    use crate::{AtlasShape, atlas_dimensions};

    #[test]
    fn line_lays_the_texels_in_a_single_row() {
        // (count, expected width); a `count`-wide, one-tall strip with no pad.
        for (count, width) in [(0, 1), (1, 1), (2, 2), (5, 5), (255, 255)] {
            let dimensions = atlas_dimensions(count, AtlasShape::Line).unwrap();
            assert_eq!(dimensions, (width, 1), "count {count}");
        }
    }

    #[test]
    fn fit_packs_texels_near_square_with_room_for_all() {
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
            let dimensions = atlas_dimensions(count, AtlasShape::Fit).unwrap();
            assert_eq!(dimensions, (width, height), "count {count}");
            assert!((width * height) as usize >= count.max(1));
        }
    }

    #[test]
    fn square_pads_the_near_square_to_a_full_square() {
        // (count, expected side); the side is fit's width, height padded up.
        for (count, side) in [(0, 1), (1, 1), (2, 2), (5, 3), (17, 5), (255, 16)] {
            let dimensions = atlas_dimensions(count, AtlasShape::Square).unwrap();
            assert_eq!(dimensions, (side, side), "count {count}");
        }
    }

    #[test]
    fn pot_rounds_the_square_side_up_to_a_power_of_two() {
        // (count, expected side); a square whose side is the next power of two.
        for (count, side) in [(1, 1), (2, 2), (5, 4), (17, 8), (255, 16), (257, 32)] {
            let dimensions = atlas_dimensions(count, AtlasShape::Pot).unwrap();
            assert_eq!(dimensions, (side, side), "count {count}");
        }
    }

    #[test]
    fn exact_keeps_a_side_that_fits_and_rejects_one_too_small() {
        // A side at or above the smallest square holds every texel.
        assert_eq!(atlas_dimensions(5, AtlasShape::Exact(3)).unwrap(), (3, 3));
        assert_eq!(
            atlas_dimensions(5, AtlasShape::Exact(256)).unwrap(),
            (256, 256)
        );

        // A side below it cannot; 5 texels need at least a 3-wide square.
        assert!(atlas_dimensions(5, AtlasShape::Exact(2)).is_err());
        assert!(atlas_dimensions(255, AtlasShape::Exact(15)).is_err());
    }
}
