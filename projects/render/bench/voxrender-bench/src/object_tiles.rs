use crate::{CasterTiles, CellGrid, GlassTiles, Result, TileMasks, solid_slabs};
use ty_math::TyVector3U32;

/// The cells along each side of a tile.
const TILE: u32 = 16;

/// One object's cells by tile, as a view and its shadow bake keep them. An
/// object's grid can hold far more cells than a device has room for, and
/// these grow with the tiles that hold a cell.
pub struct ObjectTiles {
    pub bounds: TyVector3U32,

    /// The object's opaque cells as a shaded view marches them.
    pub masks: TileMasks,

    /// The object's surfaces between materials that pass light, or none
    /// for a scene that keeps no glass.
    pub glass: GlassTiles,

    /// The boxes of opaque cells alone that hide what lies behind them, or
    /// none for a coarser copy of an object, whose cells reach past the
    /// object's.
    pub slabs: Vec<(TyVector3U32, TyVector3U32)>,

    /// The object's cells as a shadow ray walks them.
    pub casters: CasterTiles,

    /// For each tile of the grid in raster order: its place among
    /// [`masks`](Self::masks)' tiles plus one, or zero for a tile with no
    /// opaque cell.
    places: Vec<u32>,

    /// For each tile of the grid in raster order: its place among
    /// [`glass`](Self::glass)' tiles plus one, or zero for a tile with no
    /// surface between materials that pass light.
    glass_places: Vec<u32>,
}

impl ObjectTiles {
    /// The tiles of `grid`. Errors as [`TileMasks::new`] and
    /// [`CasterTiles::new`] do.
    ///
    /// # Arguments
    ///
    /// 1. `opaque`: whether a material passes no light
    /// 2. `glass`: with it, the tiles keep the surfaces between materials
    ///    that pass light, and it gives what a glass cell's word holds for
    ///    a material, under 2^12
    /// 3. `slabs`: whether the tiles keep the grid's slabs
    pub fn new(
        grid: &CellGrid,
        opaque: &dyn Fn(u32) -> bool,
        glass: Option<&mut dyn FnMut(u32) -> u32>,
        slabs: bool,
    ) -> Result<Self> {
        let masks = TileMasks::new(grid, true, Some(opaque))?;
        let counts = (grid.bounds + TILE - 1) / TILE;

        // Each tile's place plus one among the tiles whose low corners
        // `corners` gives, in the grid's raster order.
        let placed = |corners: &mut dyn Iterator<Item = TyVector3U32>| {
            let mut places = vec![0; (counts.x * counts.y * counts.z) as usize];

            for (place, corner) in (1..).zip(corners) {
                let tile = corner / TILE;

                places[((tile.x * counts.y + tile.y) * counts.z + tile.z) as usize] = place;
            }

            places
        };

        let places = placed(&mut masks.corners.iter().copied());

        let glass = match glass {
            Some(place) => GlassTiles::new(grid, opaque, place),

            None => GlassTiles {
                boxes: Vec::new(),
                surfaces: Vec::new(),
                tiles: Vec::new(),
                cells: Vec::new(),
                words: Vec::new(),
            },
        };

        let glass_places = placed(&mut glass.boxes.iter().map(|&(low, _)| low));

        Ok(ObjectTiles {
            bounds: grid.bounds,
            masks,
            glass,
            slabs: if slabs {
                solid_slabs(grid, opaque)
            } else {
                Vec::new()
            },
            casters: CasterTiles::new(grid, opaque)?,
            places,
            glass_places,
        })
    }

    /// The tiles along each axis of the grid.
    pub fn counts(&self) -> TyVector3U32 {
        (self.bounds + TILE - 1) / TILE
    }

    /// The place among [`masks`](Self::masks)' tiles of the tile at `tile`
    /// among the grid's tiles, or `None` for a tile with no opaque cell.
    pub fn tile_at(&self, tile: TyVector3U32) -> Option<usize> {
        let counts = self.counts();
        let place = self.places[((tile.x * counts.y + tile.y) * counts.z + tile.z) as usize];

        place.checked_sub(1).map(|place| place as usize)
    }

    /// The place among [`glass`](Self::glass)' tiles of the tile at `tile`
    /// among the grid's tiles, or `None` for a tile with no surface
    /// between materials that pass light.
    pub fn glass_at(&self, tile: TyVector3U32) -> Option<usize> {
        let counts = self.counts();
        let place = self.glass_places[((tile.x * counts.y + tile.y) * counts.z + tile.z) as usize];

        place.checked_sub(1).map(|place| place as usize)
    }

    /// The opaque cell at `cell` of the grid, as its tile's place among
    /// [`masks`](Self::masks)' tiles and its index in the tile. `None` for
    /// any other cell and outside the grid.
    pub fn opaque_at(&self, cell: [i64; 3]) -> Option<(usize, u16)> {
        let inside =
            (0..3).all(|axis| cell[axis] >= 0 && cell[axis] < i64::from(self.bounds[axis]));

        if !inside {
            return None;
        }

        let [x, y, z] = cell.map(|coordinate| coordinate as u32);
        let place = self.tile_at(TyVector3U32::new(x, y, z) / TILE)?;
        let row = y % TILE * TILE + x % TILE;
        let bits = self.masks.words[(self.masks.offsets[place] + row) as usize];

        (bits >> (z % TILE) & 1 == 1).then_some((place, (row * TILE + z % TILE) as u16))
    }

    /// The material of the record cell of index `index` in the tile at
    /// `tile` among [`masks`](Self::masks)' tiles. `None` for a cell with
    /// no record, which has no surface.
    pub fn material_of(&self, tile: usize, index: u16) -> Option<u32> {
        let record = self.masks.recorded[tile].binary_search(&index).ok()?;
        let offset = self.masks.offsets[tile];
        let palette = self.masks.words[(offset + TileMasks::PALETTE) as usize];
        let entry = u32::from(self.masks.entries[tile][record]);

        Some(self.masks.words[(offset + (palette & 0xffff) + entry) as usize])
    }
}

#[cfg(test)]
mod tests {
    use crate::{CellGrid, ObjectTiles};
    use ty_math::TyVector3U32;

    #[test]
    fn an_opaque_cell_reads_back_with_its_material() {
        // A floor of material 2 across two tiles along x, under one glass
        // cell of material 5.
        let bounds = TyVector3U32::new(20, 3, 4);
        let mut cells = vec![0; 20 * 3 * 4];

        for x in 0..20 {
            for z in 0..4 {
                cells[(x * 3) * 4 + z] = 3;
            }
        }

        cells[(17 * 3 + 1) * 4 + 2] = 6;

        let grid = CellGrid { bounds, cells };
        let tiles = ObjectTiles::new(&grid, &|material| material == 2, None, true).unwrap();

        let (tile, index) = tiles.opaque_at([17, 0, 2]).unwrap();

        assert_eq!((tile, index), (1, 16 + 2));
        assert_eq!(tiles.material_of(tile, index), Some(2));
        assert_eq!(tiles.opaque_at([17, 1, 2]), None);
        assert_eq!(tiles.opaque_at([17, 2, 2]), None);
        assert_eq!(tiles.opaque_at([20, 0, 0]), None);
        assert_eq!(tiles.opaque_at([0, -1, 0]), None);
    }
}
