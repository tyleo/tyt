use branded_id::U32Id;
use ty_math::TyVector3U32;
use voxcore::{BVoxVoxel, VoxObject};

/// A dense grid of cells the surface algorithms read.
pub trait SurfaceGrid {
    /// A solid cell's handle. The mesher keys faces by it and records it per
    /// face.
    type Cell: Copy;

    /// The grid's size in cells along each axis.
    fn bounds(&self) -> TyVector3U32;

    /// The handle of the solid cell at `position`, or `None` where the cell
    /// is empty. `position` is within [`bounds`](Self::bounds).
    fn cell(&self, position: TyVector3U32) -> Option<Self::Cell>;

    /// Whether `cell` passes no light. An opaque cell hides the faces against
    /// it and closes the occlusion at the corners it touches.
    fn is_opaque(&self, cell: Self::Cell) -> bool;

    /// Whether `cell` and `other` are of one material. The face between two
    /// such cells is a seam inside one body.
    fn shares_material(&self, cell: Self::Cell, other: Self::Cell) -> bool;

    /// The handle of the solid cell at the signed `position`, or `None` where
    /// the cell is empty or outside the grid.
    fn cell_at(&self, position: [i64; 3]) -> Option<Self::Cell> {
        let bounds = self.bounds().to_array();

        if position
            .iter()
            .zip(bounds)
            .any(|(&component, bound)| component < 0 || component >= i64::from(bound))
        {
            return None;
        }

        let position = TyVector3U32::from_array(position.map(|component| {
            u32::try_from(component).expect("the check above keeps the position within the grid")
        }));

        self.cell(position)
    }

    /// Whether `position` lies within the grid and holds a solid cell.
    fn is_solid(&self, position: [i64; 3]) -> bool {
        self.cell_at(position).is_some()
    }

    /// Whether `position` lies within the grid and holds an opaque cell.
    fn is_opaque_at(&self, position: [i64; 3]) -> bool {
        self.cell_at(position)
            .is_some_and(|cell| self.is_opaque(cell))
    }

    /// Whether `neighbor` hides the face of `cell` against it. An opaque
    /// neighbor does, and so does one of the cell's material.
    fn hides(&self, cell: Self::Cell, neighbor: Self::Cell) -> bool {
        self.is_opaque(neighbor) || self.shares_material(cell, neighbor)
    }
}

/// A cell is solid where its voxel is live. An object read alone has no
/// palette to say otherwise, so it is opaque throughout. Two voxels are of
/// one material where they sample the same material in every layer.
impl SurfaceGrid for VoxObject {
    type Cell = U32Id<BVoxVoxel>;

    fn bounds(&self) -> TyVector3U32 {
        self.bounds()
    }

    fn cell(&self, position: TyVector3U32) -> Option<Self::Cell> {
        self.voxel_id(position)
            .filter(|&voxel_id| self.is_live(voxel_id))
    }

    fn is_opaque(&self, _cell: Self::Cell) -> bool {
        true
    }

    fn shares_material(&self, cell: Self::Cell, other: Self::Cell) -> bool {
        self.iter_layers().all(|(layer_id, _)| {
            self.voxel_material(cell, layer_id) == self.voxel_material(other, layer_id)
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{SurfaceGrid, test_utilities::live_object};
    use branded_id::U32Id;
    use ty_math::TyVector3U32;
    use voxcore::VoxObject;

    /// A 3x1x1 bar over two layers, each voxel sampling `materials[x]`, one
    /// material per layer.
    fn two_layer_bar(materials: [[u32; 2]; 3]) -> VoxObject {
        let mut object = VoxObject::new("o".to_owned(), TyVector3U32::new(3, 1, 1)).unwrap();
        for _ in 0..2 {
            object.retain_layer(U32Id::from_u32(0)).unwrap();
        }
        for (x, materials) in (0..).zip(materials) {
            let voxel_id = object.voxel_id(TyVector3U32::new(x, 0, 0)).unwrap();
            object
                .retain_voxel(voxel_id, &materials.map(U32Id::from_u32))
                .unwrap();
        }
        object
    }

    #[test]
    fn a_live_voxel_is_a_solid_cell_and_the_outside_is_empty() {
        let object = live_object([2, 1, 1], &[[1, 0, 0]]);

        let voxel_id = object.voxel_id(TyVector3U32::new(1, 0, 0)).unwrap();

        assert_eq!(SurfaceGrid::bounds(&object), TyVector3U32::new(2, 1, 1));
        assert_eq!(object.cell(TyVector3U32::new(1, 0, 0)), Some(voxel_id));
        assert_eq!(object.cell(TyVector3U32::new(0, 0, 0)), None);

        assert!(object.is_solid([1, 0, 0]));
        assert!(!object.is_solid([0, 0, 0]));
        assert!(!object.is_solid([2, 0, 0]));
        assert!(!object.is_solid([1, -1, 0]));

        assert!(object.is_opaque(voxel_id));
        assert!(object.is_opaque_at([1, 0, 0]));
        assert!(!object.is_opaque_at([0, 0, 0]));
    }

    #[test]
    fn two_voxels_share_a_material_when_every_layer_agrees() {
        let object = two_layer_bar([[0, 0], [0, 1], [0, 0]]);
        let voxel = |x| object.voxel_id(TyVector3U32::new(x, 0, 0)).unwrap();

        assert!(object.shares_material(voxel(0), voxel(2)));
        assert!(!object.shares_material(voxel(0), voxel(1)));
        assert!(object.hides(voxel(0), voxel(1)));
    }
}
