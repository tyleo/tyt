use crate::{
    Error, Result,
    operations::mesh::{ArrayDomain, MeshElement, TextureShape},
};
use ty_math::TyVector2F64;

/// Each corner's texel in a 2x2 block, going around the block as the corners
/// go around the face, so the block's bilinear blend follows the face's.
const CORNER_TEXELS: [[u32; 2]; 4] = [[0, 0], [1, 0], [1, 1], [0, 1]];

/// One atlas's cells on the canvas, in raster order from the top left. A cell
/// is one texel, or a 2x2 block on the corner atlas, and the canvas's unused
/// cells stay empty.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AtlasLayout {
    domain: ArrayDomain,
    columns: u32,
    rows: u32,
}

impl AtlasLayout {
    /// Lays `count` cells of `domain` on the `shape` canvas. Errors if an
    /// exact canvas is too small.
    pub(crate) fn shape(domain: ArrayDomain, count: usize, shape: TextureShape) -> Result<Self> {
        let count = u32::try_from(count.max(1)).expect("an entry count fits u32");

        // The fewest columns that keep the packing no taller than it is wide.
        let base = ((f64::from(count)).sqrt().ceil() as u32).max(1);

        let (columns, rows) = match shape {
            TextureShape::Exact(side) => {
                if u64::from(side) * u64::from(side) < u64::from(count) {
                    return Err(Error::mesh_record(
                        MeshElement::TextureShape,
                        format!(
                            "is a {side}x{side} canvas, too small for {count} {domain} cells; \
                             the smallest square that fits is {base}"
                        ),
                    ));
                }

                (side, side)
            }

            TextureShape::Fit => (base, count.div_ceil(base)),

            TextureShape::Line => (count, 1),

            TextureShape::Pot => {
                let side = base.next_power_of_two();
                (side, side)
            }

            TextureShape::Square => (base, base),
        };

        Ok(AtlasLayout {
            domain,
            columns,
            rows,
        })
    }

    /// The texels along one side of a cell.
    pub(crate) fn cell_side(&self) -> u32 {
        match self.domain {
            ArrayDomain::Corner => 2,
            ArrayDomain::Face | ArrayDomain::Swatch | ArrayDomain::Voxel => 1,
        }
    }

    /// The canvas width in texels.
    pub(crate) fn width(&self) -> u32 {
        self.columns * self.cell_side()
    }

    /// The canvas height in texels.
    pub(crate) fn height(&self) -> u32 {
        self.rows * self.cell_side()
    }

    /// The texel of `cell`'s `corner`, in the face's corner order; every
    /// corner of a one-texel cell shares it.
    pub(crate) fn texel(&self, cell: usize, corner: usize) -> [u32; 2] {
        let cell = u32::try_from(cell).expect("a cell index fits u32");
        let side = self.cell_side();

        let [dx, dy] = if side == 1 {
            [0, 0]
        } else {
            CORNER_TEXELS[corner]
        };

        [
            (cell % self.columns) * side + dx,
            (cell / self.columns) * side + dy,
        ]
    }

    /// The UV at the center of `cell`'s `corner` texel, with the origin at the
    /// canvas's top left.
    pub(crate) fn uv(&self, cell: usize, corner: usize) -> TyVector2F64 {
        let [x, y] = self.texel(cell, corner);

        TyVector2F64::new(
            (f64::from(x) + 0.5) / f64::from(self.width()),
            (f64::from(y) + 0.5) / f64::from(self.height()),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::operations::mesh::{ArrayDomain, AtlasLayout, MeshElement, TextureShape};
    use ty_math::TyVector2F64;

    fn canvas(count: usize, shape: TextureShape) -> (u32, u32) {
        let layout = AtlasLayout::shape(ArrayDomain::Face, count, shape).unwrap();
        (layout.width(), layout.height())
    }

    #[test]
    fn each_shape_holds_every_cell_and_an_empty_atlas_keeps_one() {
        for (count, fit, square, pot) in [
            (0, (1, 1), (1, 1), (1, 1)),
            (1, (1, 1), (1, 1), (1, 1)),
            (2, (2, 1), (2, 2), (2, 2)),
            (5, (3, 2), (3, 3), (4, 4)),
            (17, (5, 4), (5, 5), (8, 8)),
            (255, (16, 16), (16, 16), (16, 16)),
            (257, (17, 16), (17, 17), (32, 32)),
        ] {
            assert_eq!(canvas(count, TextureShape::Fit), fit, "fit {count}");
            assert_eq!(canvas(count, TextureShape::Line), (count.max(1) as u32, 1));
            assert_eq!(canvas(count, TextureShape::Pot), pot, "pot {count}");
            assert_eq!(
                canvas(count, TextureShape::Square),
                square,
                "square {count}"
            );
        }
    }

    #[test]
    fn an_exact_canvas_keeps_its_side_and_errors_when_too_small() {
        assert_eq!(canvas(5, TextureShape::Exact(3)), (3, 3));
        assert_eq!(canvas(5, TextureShape::Exact(256)), (256, 256));

        let error = AtlasLayout::shape(ArrayDomain::Face, 5, TextureShape::Exact(2)).unwrap_err();
        assert!(
            matches!(
                &error,
                crate::Error::MeshRecord {
                    element: MeshElement::TextureShape,
                    ..
                }
            ),
            "{error}"
        );
        assert!(error.to_string().contains("5 face cells"), "{error}");
    }

    #[test]
    fn a_cell_sits_at_its_texel_center_and_a_corner_block_goes_around() {
        let layout = AtlasLayout::shape(ArrayDomain::Face, 5, TextureShape::Fit).unwrap();
        assert_eq!(layout.texel(4, 0), [1, 1]);
        assert_eq!(layout.uv(4, 3), TyVector2F64::new(0.5, 0.75));

        let corners = AtlasLayout::shape(ArrayDomain::Corner, 5, TextureShape::Fit).unwrap();
        assert_eq!((corners.width(), corners.height()), (6, 4));
        assert_eq!(
            (0..4)
                .map(|corner| corners.texel(4, corner))
                .collect::<Vec<_>>(),
            [[2, 2], [3, 2], [3, 3], [2, 3]]
        );
        assert_eq!(corners.uv(4, 1), TyVector2F64::new(3.5 / 6.0, 2.5 / 4.0));
    }
}
