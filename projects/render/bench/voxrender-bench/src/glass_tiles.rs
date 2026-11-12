use crate::live_boxes;
use ty_math::TyVector3U32;
use voxrender::RenderObject;
use voxrender_wgpu::dense_cells;

/// The cells along each side of a tile.
const TILE: u32 = 16;

/// An object's surfaces between materials that pass light, by tile, as a
/// glass walk reads them.
///
/// A ray meets such a surface where it crosses a face of a cell that passes
/// light into a cell that is neither opaque nor of the same material. A
/// tile's busy cells are the cells with such a face and the opaque cells,
/// which end a walk. A ray crosses any other cell with nothing to do, so a
/// body of water draws its skin alone.
///
/// A tile's words start with a word for each row of 16 cells along z, 256
/// in the order of a cell's index, `(y * 16 + x) * 16 + z`. A row's low 16
/// bits hold its busy cells, and the bits above them the tile's busy cells
/// before the row, as a shaded [`TileMasks`](crate::TileMasks)' rows do. A
/// word for each busy cell follows in index order. An opaque cell's is
/// [`OPAQUE`](Self::OPAQUE). Any other's low 6 bits hold a bit for each
/// face with a surface, under the cell's material from bit 8. Face
/// `2 * axis` looks down its axis and face `2 * axis + 1` up it.
pub struct GlassTiles {
    /// The box each tile draws, as its low cell and the corner past its
    /// high cell: the box around its busy cells. The tile's opaque cells
    /// draw a box whose faces can lie before a surface they do not hide,
    /// so a walk starts no later than that box does.
    pub boxes: Vec<(TyVector3U32, TyVector3U32)>,

    /// The box around each tile's cells with a surface, where a walk can
    /// end.
    pub surfaces: Vec<(TyVector3U32, TyVector3U32)>,

    /// The offset in [`words`](Self::words) of each box's tile.
    pub tiles: Vec<u32>,

    /// Each tile's words.
    pub words: Vec<u32>,
}

impl GlassTiles {
    /// The rows of a tile, and the offset in its words of its busy cells'
    /// words.
    pub const ROWS: u32 = TILE * TILE;

    /// An opaque cell's word.
    pub const OPAQUE: u32 = 1 << 6;

    /// The tiles of `object` that hold a surface between materials that
    /// pass light.
    ///
    /// # Arguments
    ///
    /// 1. `opaque`: whether a material id passes no light
    /// 2. `place`: what a cell's word holds for a material id, under 2^24
    pub fn new(
        object: &RenderObject,
        opaque: &dyn Fn(u32) -> bool,
        place: &mut dyn FnMut(u32) -> u32,
    ) -> Self {
        let bounds = object.bounds();
        let cells = dense_cells(object);

        // The value of the cell at `cell`: 0 when empty or outside the
        // object, else its material id plus one.
        let value = |cell: [i64; 3]| {
            let inside = (0..3).all(|axis| cell[axis] >= 0 && cell[axis] < i64::from(bounds[axis]));

            if inside {
                let [x, y, z] = cell.map(|coordinate| coordinate as usize);

                cells[(x * bounds.y as usize + y) * bounds.z as usize + z]
            } else {
                0
            }
        };

        let passes = |value: u32| value.checked_sub(1).is_some_and(|id| !opaque(id));

        let mut tiles = GlassTiles {
            boxes: Vec::new(),
            surfaces: Vec::new(),
            tiles: Vec::new(),
            words: Vec::new(),
        };

        for (low, _) in live_boxes(object, TILE) {
            let corner = (low / TILE * TILE).to_array().map(i64::from);

            // The tile's cell of `index`, in the object's grid.
            let cell_of = |index: u32| {
                [
                    corner[0] + i64::from(index / TILE % TILE),
                    corner[1] + i64::from(index / (TILE * TILE)),
                    corner[2] + i64::from(index % TILE),
                ]
            };

            let mut rows = vec![0; Self::ROWS as usize];
            let mut busy = Vec::new();

            // The low and high corners around the cells with a surface,
            // then around every busy cell.
            let mut surfaces = ([i64::MAX; 3], [i64::MIN; 3]);
            let mut drawn = surfaces;

            let grow = |(least, most): &mut ([i64; 3], [i64; 3]), cell: [i64; 3]| {
                for axis in 0..3 {
                    least[axis] = least[axis].min(cell[axis]);
                    most[axis] = most[axis].max(cell[axis] + 1);
                }
            };

            for index in 0..TILE * TILE * TILE {
                let cell = cell_of(index);
                let own = value(cell);

                if index % TILE == 0 {
                    rows[(index / TILE) as usize] = (busy.len() as u32) << 16;
                }

                let word = if passes(own) {
                    let faces = (0..6u32)
                        .filter(|face| {
                            let mut beside = cell;

                            beside[(face / 2) as usize] += if face % 2 == 1 { 1 } else { -1 };

                            let other = value(beside);

                            other != own && (other == 0 || passes(other))
                        })
                        .fold(0, |faces, face| faces | 1 << face);

                    if faces == 0 {
                        continue;
                    }

                    grow(&mut surfaces, cell);

                    place(own - 1) << 8 | faces
                } else if own != 0 {
                    Self::OPAQUE
                } else {
                    continue;
                };

                grow(&mut drawn, cell);
                rows[(index / TILE) as usize] |= 1 << (index % TILE);
                busy.push(word);
            }

            // A tile with no surface draws nothing.
            if surfaces.0[0] == i64::MAX {
                continue;
            }

            let corners = |(least, most): ([i64; 3], [i64; 3])| {
                let corner_of = |cell: [i64; 3]| {
                    let [x, y, z] = cell.map(|side| side as u32);

                    TyVector3U32::new(x, y, z)
                };

                (corner_of(least), corner_of(most))
            };

            tiles.boxes.push(corners(drawn));
            tiles.surfaces.push(corners(surfaces));
            tiles.tiles.push(
                u32::try_from(tiles.words.len())
                    .expect("an object's glass words number under 2^32"),
            );
            tiles.words.extend(rows);
            tiles.words.extend(busy);
        }

        tiles
    }
}

#[cfg(test)]
mod tests {
    use crate::{GlassTiles, painted_object};
    use branded_id::U32Id;
    use ty_math::TyVector3U32;

    #[test]
    fn a_cell_is_busy_where_a_ray_meets_a_surface_or_stops() {
        // Along x: a wall, two cells of glass, a cell of another glass,
        // and air. Material 0 is the wall's.
        let object = painted_object("pane", TyVector3U32::new(5, 1, 1), |position| {
            [Some(0), Some(1), Some(1), Some(2), None][position.x as usize].map(U32Id::from_u32)
        });

        let tiles = GlassTiles::new(&object, &|id| id == 0, &mut |id| id + 10);

        // The wall joins the box the tile draws.
        assert_eq!(
            tiles.boxes,
            [(TyVector3U32::new(0, 0, 0), TyVector3U32::new(4, 1, 1))]
        );
        assert_eq!(
            tiles.surfaces,
            [(TyVector3U32::new(1, 0, 0), TyVector3U32::new(4, 1, 1))]
        );
        assert_eq!(tiles.tiles, [0]);

        // A row runs along z, so each cell sits in a row of its own.
        let row = |x: usize| tiles.words[x];
        assert_eq!(
            [row(0), row(1), row(2), row(3), row(4)],
            [1, 1 | 1 << 16, 1 | 2 << 16, 1 | 3 << 16, 4 << 16]
        );

        // Each glass cell has a surface up and down y and z. The first
        // meets the wall down x and its like up x. The second meets the
        // other glass up x, which meets it down x and air up x.
        let sides = 0b111100;
        assert_eq!(
            tiles.words[GlassTiles::ROWS as usize..],
            [
                GlassTiles::OPAQUE,
                11 << 8 | sides,
                11 << 8 | sides | 0b10,
                12 << 8 | sides | 0b11
            ]
        );
    }

    #[test]
    fn a_body_of_glass_is_busy_at_its_skin_alone() {
        // A cube of glass 4 cells a side holds 8 cells with no surface.
        let object = painted_object("cube", TyVector3U32::splat(4), |_| Some(U32Id::from_u32(1)));
        let tiles = GlassTiles::new(&object, &|_| false, &mut |id| id);

        assert_eq!(tiles.words.len(), GlassTiles::ROWS as usize + 64 - 8);
    }

    #[test]
    fn an_object_with_no_glass_has_no_tiles() {
        let object = painted_object("wall", TyVector3U32::new(2, 2, 2), |_| {
            Some(U32Id::from_u32(0))
        });

        assert!(
            GlassTiles::new(&object, &|_| true, &mut |id| id)
                .boxes
                .is_empty()
        );
    }
}
