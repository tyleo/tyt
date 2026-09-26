use crate::operations::voxelize::{VoxelFrame, VoxelScale};
use branded_id::U32Id;
use meshdoc::BMeshObject;
use std::ops::Range;
use ty_math::{TyBoundsF64, TyTransformF64, TyVector3F64};

/// One placement of a mesh object by a hierarchy node, the unit that becomes
/// one voxel object.
pub(crate) struct PlacedObject {
    /// The mesh object placed.
    pub mesh_object_id: U32Id<BMeshObject>,

    /// The mesh object's name, empty when it has none.
    pub object_name: String,

    /// The placing node's name, empty when it has none.
    pub node_name: String,

    /// The placement's triangles within the mesh's triangle list, in the
    /// grid frame.
    pub range: Range<usize>,

    /// The placing node's world transform.
    pub world: TyTransformF64,

    /// The bounds of the placement's geometry in world space, or `None` when
    /// it has no triangles.
    pub world_bounds: Option<TyBoundsF64>,
}

impl PlacedObject {
    /// The voxel object's name: the mesh object's, else the placing node's,
    /// else `fallback`, else empty.
    pub fn name<'s>(&'s self, fallback: Option<&'s str>) -> &'s str {
        [
            self.object_name.as_str(),
            self.node_name.as_str(),
            fallback.unwrap_or_default(),
        ]
        .into_iter()
        .find(|name| !name.is_empty())
        .unwrap_or_default()
    }

    /// The length of one grid-frame unit on each axis, in the units the voxel
    /// size is given in. One except in a local frame with the scale baked,
    /// where the grid is in local units and the voxel size in world units.
    pub fn grid_unit(&self, frame: VoxelFrame, scale: VoxelScale) -> TyVector3F64 {
        match (frame, scale) {
            (VoxelFrame::Local, VoxelScale::Bake) => self.world.scale.abs(),
            _ => TyVector3F64::ONE,
        }
    }

    /// The transform of the node placing a grid of `voxel_size` cubes built
    /// in the grid frame.
    pub fn node_transform(
        &self,
        voxel_size: f64,
        frame: VoxelFrame,
        scale: VoxelScale,
    ) -> TyTransformF64 {
        let size = TyVector3F64::splat(voxel_size);

        let scale = match scale {
            VoxelScale::Bake => size * self.world.scale.signum(),
            VoxelScale::Keep => size * self.world.scale,
        };

        match frame {
            VoxelFrame::World => TyTransformF64 {
                scale,
                ..TyTransformF64::IDENTITY
            },
            VoxelFrame::Local => TyTransformF64 {
                position: self.world.position,
                rotation: self.world.rotation,
                scale,
            },
        }
    }
}
