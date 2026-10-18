use crate::is_opaque;
use branded_id::U32Id;
use ty_math::TyVector3U32;
use voxcore::BVoxVoxel;
use voxrender::{RenderMaterial, RenderObject, RenderScene};
use voxsurface::SurfaceGrid;

/// A render object's grid read with its scene's materials: a cell is opaque
/// where its material's pass is zero, and two cells are of one material
/// where they sample one material id.
pub struct RenderGrid<'a> {
    scene: &'a RenderScene,

    object: &'a RenderObject,
}

impl<'a> RenderGrid<'a> {
    pub fn new(scene: &'a RenderScene, object: &'a RenderObject) -> Self {
        RenderGrid { scene, object }
    }

    fn material(&self, cell: U32Id<BVoxVoxel>) -> &RenderMaterial {
        let material_id = self
            .object
            .voxel_material(cell)
            .expect("a solid cell holds a material");

        self.scene
            .material(material_id)
            .expect("a voxel samples one of the scene's materials")
    }
}

impl SurfaceGrid for RenderGrid<'_> {
    type Cell = U32Id<BVoxVoxel>;

    fn bounds(&self) -> TyVector3U32 {
        self.object.bounds()
    }

    fn cell(&self, position: TyVector3U32) -> Option<Self::Cell> {
        self.object
            .voxel_id(position)
            .filter(|&voxel_id| self.object.is_live(voxel_id))
    }

    fn is_opaque(&self, cell: Self::Cell) -> bool {
        is_opaque(self.material(cell))
    }

    fn shares_material(&self, cell: Self::Cell, other: Self::Cell) -> bool {
        self.object.voxel_material(cell) == self.object.voxel_material(other)
    }
}

#[cfg(test)]
mod tests {
    use crate::RenderGrid;
    use branded_id::U32Id;
    use ty_math::{TyLinSrgbaF64, TyVector3U32};
    use voxrender::{RenderMaterial, RenderObject, RenderScene};
    use voxsurface::SurfaceGrid;

    #[test]
    fn a_cell_is_opaque_when_its_pass_is_zero_and_shares_its_material_id() {
        let mut scene = RenderScene::default();
        let opaque_id = scene.retain_material(RenderMaterial::default()).unwrap();
        let glass_id = scene
            .retain_material(RenderMaterial {
                metallic: 0.0,
                transmission: 1.0,
                ..RenderMaterial::default()
            })
            .unwrap();
        let half_id = scene
            .retain_material(RenderMaterial {
                base_color: TyLinSrgbaF64::new(1.0, 1.0, 1.0, 0.5),
                ..RenderMaterial::default()
            })
            .unwrap();

        let mut object = RenderObject::new("o".to_owned(), TyVector3U32::new(5, 1, 1)).unwrap();
        for (x, material_id) in [(0, opaque_id), (1, glass_id), (2, half_id), (3, glass_id)] {
            let voxel_id = object.voxel_id(TyVector3U32::new(x, 0, 0)).unwrap();
            object
                .set_voxel_material(voxel_id, Some(material_id))
                .unwrap();
        }
        let object_id = U32Id::from_u32(0);
        scene.retain_object(object_id, object).unwrap();
        let object = scene.object(object_id).unwrap();

        let grid = RenderGrid::new(&scene, object);
        let cell = |x| grid.cell(TyVector3U32::new(x, 0, 0)).unwrap();

        assert_eq!(SurfaceGrid::bounds(&grid), TyVector3U32::new(5, 1, 1));
        assert_eq!(grid.cell(TyVector3U32::new(4, 0, 0)), None);
        assert!(grid.is_solid([1, 0, 0]));
        assert!(!grid.is_solid([4, 0, 0]));
        assert!(!grid.is_solid([5, 0, 0]));

        assert!(grid.is_opaque(cell(0)));
        assert!(!grid.is_opaque(cell(1)));
        assert!(!grid.is_opaque(cell(2)));
        assert!(grid.is_opaque_at([0, 0, 0]));
        assert!(!grid.is_opaque_at([1, 0, 0]));
        assert!(!grid.is_opaque_at([4, 0, 0]));

        assert!(grid.shares_material(cell(1), cell(3)));
        assert!(!grid.shares_material(cell(1), cell(2)));

        // An opaque neighbor hides a face, a seam inside one glass hides it,
        // and another glass does not.
        assert!(grid.hides(cell(1), cell(0)));
        assert!(grid.hides(cell(1), cell(3)));
        assert!(!grid.hides(cell(1), cell(2)));
        assert!(!grid.hides(cell(0), cell(1)));
    }
}
