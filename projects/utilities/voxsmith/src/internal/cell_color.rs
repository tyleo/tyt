use branded_id::{IdVec, U32Id};
use voxcore::{BVoxLayer, BVoxMaterial, BVoxVoxel, VoxObject};

/// A per-voxel color read, resolved once per object by
/// [`resolve_cell_color`](crate::resolve_cell_color) so voxel loops carry no
/// palette machinery.
pub struct CellColor<'a> {
    /// The object the colors read through.
    object: &'a VoxObject,

    /// The winning layer, or `None` when no layer supplies a color.
    layer_id: Option<U32Id<BVoxLayer>>,

    /// The decoded color for each material.
    colors: IdVec<BVoxMaterial, [u8; 4]>,
}

impl<'a> CellColor<'a> {
    /// A read over the winning `layer_id`, taking each voxel's color from
    /// `colors` at the material it samples there.
    pub fn new(
        object: &'a VoxObject,
        layer_id: U32Id<BVoxLayer>,
        colors: IdVec<BVoxMaterial, [u8; 4]>,
    ) -> Self {
        Self {
            object,
            layer_id: Some(layer_id),
            colors,
        }
    }

    /// A read yielding transparent black for every voxel, when no layer
    /// supplies a color.
    pub fn transparent(object: &'a VoxObject) -> Self {
        Self {
            object,
            layer_id: None,
            colors: IdVec::default(),
        }
    }

    /// The color of the live voxel at `voxel_id`.
    pub fn color(&self, voxel_id: U32Id<BVoxVoxel>) -> [u8; 4] {
        let Some(layer_id) = self.layer_id else {
            return [0, 0, 0, 0];
        };
        let material_id = self
            .object
            .voxel_material(voxel_id, layer_id)
            .expect("a live voxel samples the winning layer");
        self.colors[material_id.to_usize_id()]
    }
}
