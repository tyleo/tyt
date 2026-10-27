use crate::operations::object::Swatches;
use branded_id::U32Id;
use ty_math::TyVector3U32;
use vox_value_language::BSwatch;
use voxcore::{BVoxVoxel, VoxObject};
use voxsurface::SurfaceGrid;

/// An object's grid read through its swatches: a cell is opaque where its
/// swatch passes no light, and two cells are of one material where they
/// share a swatch.
pub struct SwatchGrid<'a> {
    object: &'a VoxObject,

    swatches: &'a Swatches<'a>,
}

impl<'a> SwatchGrid<'a> {
    pub fn new(object: &'a VoxObject, swatches: &'a Swatches<'a>) -> Self {
        SwatchGrid { object, swatches }
    }

    fn swatch_id(&self, voxel_id: U32Id<BVoxVoxel>) -> U32Id<BSwatch> {
        let entry_id = self.swatches.voxel_entry_id(voxel_id);

        self.swatches.voxel_swatch_ids()[entry_id.to_usize_id()]
    }
}

impl SurfaceGrid for SwatchGrid<'_> {
    type Cell = U32Id<BVoxVoxel>;

    fn bounds(&self) -> TyVector3U32 {
        self.object.bounds()
    }

    fn cell(&self, position: TyVector3U32) -> Option<Self::Cell> {
        self.object.cell(position)
    }

    fn is_opaque(&self, cell: Self::Cell) -> bool {
        self.swatches.is_opaque(self.swatch_id(cell))
    }

    fn shares_material(&self, cell: Self::Cell, other: Self::Cell) -> bool {
        self.swatch_id(cell) == self.swatch_id(other)
    }
}

#[cfg(test)]
mod tests {
    use crate::operations::object::{SwatchGrid, Swatches};
    use branded_id::U32Id;
    use ty_math::TyVector3U32;
    use voxcore::{
        VoxMain, VoxObject, VoxPalette, VoxValuePool,
        material::{BASE_COLOR, METALLIC, TRANSMISSION},
    };
    use voxsurface::SurfaceGrid;

    /// A main holding a 4x1x1 bar: an opaque red metal, two of one blue
    /// glass, and an empty cell.
    fn glazed() -> VoxMain {
        let mut main: VoxMain = VoxMain::default();
        let colors = main.retain_value_pool(
            VoxValuePool::vec_4_float(vec![[1.0, 0.0, 0.0, 1.0], [0.0, 0.0, 1.0, 1.0]]).unwrap(),
        );
        let scalars = main.retain_value_pool(VoxValuePool::float(vec![1.0, 0.0]).unwrap());

        let mut palette = VoxPalette::default();
        palette
            .retain_property(BASE_COLOR.to_owned(), colors)
            .unwrap();
        palette
            .retain_property(METALLIC.to_owned(), scalars)
            .unwrap();
        palette
            .retain_property(TRANSMISSION.to_owned(), scalars)
            .unwrap();
        let red_id = palette
            .retain_material(vec![
                U32Id::from_u32(0),
                U32Id::from_u32(0),
                U32Id::from_u32(1),
            ])
            .unwrap();
        let glass_id = palette
            .retain_material(vec![
                U32Id::from_u32(1),
                U32Id::from_u32(1),
                U32Id::from_u32(0),
            ])
            .unwrap();
        let palette_id = main.retain_palette(palette).unwrap();

        let mut object = VoxObject::new("bar".to_owned(), TyVector3U32::new(4, 1, 1)).unwrap();
        object.retain_layer_filled(palette_id, red_id);
        for (x, material_id) in [(0, red_id), (1, glass_id), (2, glass_id)] {
            let voxel_id = object.voxel_id(TyVector3U32::new(x, 0, 0)).unwrap();
            object.retain_voxel(voxel_id, &[material_id]).unwrap();
        }
        main.retain_object(object).unwrap();

        main
    }

    #[test]
    fn a_cell_is_opaque_by_its_swatch_and_shares_a_material_with_its_swatch_mates() {
        let main = glazed();
        let (_, object) = main.iter_objects().next().unwrap();
        let swatches = Swatches::resolve(&main, object).unwrap();
        let grid = SwatchGrid::new(object, &swatches);
        let cell = |x| grid.cell(TyVector3U32::new(x, 0, 0)).unwrap();

        assert_eq!(SurfaceGrid::bounds(&grid), TyVector3U32::new(4, 1, 1));
        assert_eq!(grid.cell(TyVector3U32::new(3, 0, 0)), None);

        assert!(grid.is_opaque(cell(0)));
        assert!(!grid.is_opaque(cell(1)));
        assert!(!grid.is_opaque_at([2, 0, 0]));
        assert!(!grid.is_opaque_at([3, 0, 0]));

        assert!(grid.shares_material(cell(1), cell(2)));
        assert!(!grid.shares_material(cell(0), cell(1)));
        assert!(grid.hides(cell(1), cell(0)));
        assert!(grid.hides(cell(1), cell(2)));
        assert!(!grid.hides(cell(0), cell(1)));
    }
}
