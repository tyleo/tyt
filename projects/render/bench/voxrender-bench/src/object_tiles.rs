use crate::{
    BObjectTile, CasterTiles, CellBox, CellGrid, GlassTiles, Result, TILE, TileMasks, raster_index,
    solid_slabs,
};
use branded_id::UsizeId;
use ty_math::TyVector3U32;

/// One object's cells by tile, as a view and its shadow bake keep them.
/// They grow with the tiles that hold a cell, because an object's grid can
/// hold far more cells than a device has room for.
pub struct ObjectTiles {
    pub bounds: TyVector3U32,

    /// The object's opaque cells as a shaded view marches them.
    pub masks: TileMasks,

    /// The object's surfaces between materials that pass light, or none
    /// for a scene that keeps no glass.
    pub glass: GlassTiles,

    /// The object's [`solid_slabs`](solid_slabs()), or none for a coarser
    /// copy, whose cells reach past the object's.
    pub slabs: Vec<CellBox>,

    /// How many of its object's cells one of the cells spans along an axis:
    /// one for an object, and twice the one before for each of its coarser
    /// copies.
    pub span: u32,

    /// The empty cells the grid holds ahead of its object's first cell
    /// along each axis, which set its coarser copies on the lattice their
    /// scene's objects share.
    pub shift: TyVector3U32,

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
    /// 2. `glass`: the `place` that [`GlassTiles::new`] takes, or `None` to
    ///    keep no glass
    /// 3. `span` and `shift`: the tiles' [`span`](Self::span) and
    ///    [`shift`](Self::shift)
    pub fn new(
        grid: &CellGrid,
        opaque: &dyn Fn(u32) -> bool,
        glass: Option<&mut dyn FnMut(u32) -> u32>,
        span: u32,
        shift: TyVector3U32,
    ) -> Result<Self> {
        let masks = TileMasks::new(grid, opaque)?;
        let counts = (grid.bounds + TILE - 1) / TILE;

        // Each tile's place plus one among the tiles whose low corners
        // `corners` gives, in the grid's raster order.
        let placed = |corners: &mut dyn Iterator<Item = TyVector3U32>| {
            let mut places = vec![0; (counts.x * counts.y * counts.z) as usize];

            for (place, corner) in (1..).zip(corners) {
                places[raster_index(corner / TILE, counts)] = place;
            }

            places
        };

        let places = placed(&mut masks.tiles.iter().map(|tile| tile.corner));

        let glass = glass.map_or_else(GlassTiles::default, |place| {
            GlassTiles::new(grid, opaque, place)
        });

        let glass_places = placed(&mut glass.boxes.iter().map(|drawn| drawn.cells.0));

        Ok(ObjectTiles {
            bounds: grid.bounds,
            masks,
            glass,
            slabs: if span == 1 {
                solid_slabs(grid, opaque)
            } else {
                Vec::new()
            },
            span,
            shift,
            casters: CasterTiles::new(grid, opaque)?,
            places,
            glass_places,
        })
    }

    /// The tiles along each axis of the grid.
    pub fn counts(&self) -> TyVector3U32 {
        (self.bounds + TILE - 1) / TILE
    }

    /// The one of [`masks`](Self::masks)' tiles at `tile` among the grid's
    /// tiles, or `None` for a tile with no opaque cell.
    pub fn tile_at(&self, tile: TyVector3U32) -> Option<UsizeId<BObjectTile>> {
        held(&self.places, raster_index(tile, self.counts()))
    }

    /// The one of [`glass`](Self::glass)' tiles at `tile` among the grid's
    /// tiles, or `None` for a tile with no surface between materials that
    /// pass light.
    pub fn glass_at(&self, tile: TyVector3U32) -> Option<UsizeId<BObjectTile>> {
        held(&self.glass_places, raster_index(tile, self.counts()))
    }

    /// The opaque cell at `cell` of the grid, as the one of
    /// [`masks`](Self::masks)' tiles that holds it and its index in the
    /// tile. `None` for any other cell and outside the grid.
    pub fn opaque_at(&self, cell: [i64; 3]) -> Option<(UsizeId<BObjectTile>, u16)> {
        let inside =
            (0..3).all(|axis| cell[axis] >= 0 && cell[axis] < i64::from(self.bounds[axis]));

        if !inside {
            return None;
        }

        let [x, y, z] = cell.map(|coordinate| coordinate as u32);
        let place = self.tile_at(TyVector3U32::new(x, y, z) / TILE)?;
        let row = y % TILE * TILE + x % TILE;
        let bits = self.masks.words[(self.masks.tiles[place].offset + row) as usize];

        (bits >> (z % TILE) & 1 == 1).then_some((place, (row * TILE + z % TILE) as u16))
    }

    /// The material of the record cell of index `index` in `tile` of
    /// [`masks`](Self::masks)' tiles. `None` for a cell with no record,
    /// which has no surface.
    pub fn material_of(&self, tile: UsizeId<BObjectTile>, index: u16) -> Option<u32> {
        let tile = &self.masks.tiles[tile];
        let record = tile.recorded.binary_search(&index).ok()?;

        Some(self.masks.palette(tile.offset)[usize::from(tile.entries[record])])
    }
}

/// The tile whose place plus one `places` holds at `at`, if it holds one.
fn held(places: &[u32], at: usize) -> Option<UsizeId<BObjectTile>> {
    let place = places[at].checked_sub(1)?;

    Some(UsizeId::from_usize(place as usize))
}

#[cfg(test)]
mod tests {
    use crate::{CellGrid, ObjectTiles};
    use branded_id::UsizeId;
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
        let shift = TyVector3U32::ZERO;
        let tiles = ObjectTiles::new(&grid, &|material| material == 2, None, 1, shift).unwrap();

        let (tile, index) = tiles.opaque_at([17, 0, 2]).unwrap();

        assert_eq!((tile, index), (UsizeId::from_usize(1), 16 + 2));
        assert_eq!(tiles.material_of(tile, index), Some(2));
        assert_eq!(tiles.opaque_at([17, 1, 2]), None);
        assert_eq!(tiles.opaque_at([17, 2, 2]), None);
        assert_eq!(tiles.opaque_at([20, 0, 0]), None);
        assert_eq!(tiles.opaque_at([0, -1, 0]), None);
    }
}
