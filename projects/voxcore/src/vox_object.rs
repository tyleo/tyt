use crate::{BVoxPalette, BVoxPaletteCell, BVoxPaletteRef, BVoxVoxel};
use branded_id::{
    U32Id,
    soa::{IdField, IdStruct},
};
use ty_math::TyVector3U32;

/// One object's voxel volume: a dense voxel pool, the palettes it references,
/// and a sample per voxel per palette reference.
///
/// An object is pure geometry placed by a
/// [`VoxHierarchyNode`](crate::VoxHierarchyNode) that references it. Voxels are
/// generated from [`voxel_ids`](Self::voxel_ids).
/// [`palette_refs`](Self::palette_refs) lists the shared
/// [`VoxPalette`](crate::VoxPalette)s it samples, in resolution order;
/// [`samples`](Self::samples) carries, per palette reference, the cell each
/// voxel takes from that palette.
///
/// The columns track liveness through the id pools, so this type does not
/// derive `Clone` or `PartialEq`; its [`Drop`] releases every column.
#[derive(Debug, Default)]
pub struct VoxObject {
    /// The display name of the object.
    pub name: String,

    /// The `[x, y, z]` size in voxels of the dense voxel grid.
    pub bounds: TyVector3U32,

    /// The voxel id pool: every retained id is one voxel in the grid.
    pub voxel_ids: IdStruct<BVoxVoxel>,

    /// The palette reference id pool, shared by
    /// [`palette_refs`](Self::palette_refs) and [`samples`](Self::samples).
    pub palette_ref_ids: IdStruct<BVoxPaletteRef>,

    /// The shared palette each reference points to, in resolution order.
    pub palette_refs: IdField<BVoxPaletteRef, U32Id<BVoxPalette>>,

    /// One column per palette reference: each maps a voxel to the cell it takes
    /// from that reference's palette.
    pub samples: IdField<BVoxPaletteRef, IdField<BVoxVoxel, U32Id<BVoxPaletteCell>>>,
}

impl Drop for VoxObject {
    fn drop(&mut self) {
        // Safety: `palette_refs` and `samples` retain a value for every id in
        // `palette_ref_ids`. `palette_refs` holds `Copy` palette ids and each
        // `samples` column holds `Copy` cell ids, so dropping each value (the
        // fields free their own storage) is all that is needed here.
        unsafe {
            self.palette_refs.release_all(&self.palette_ref_ids);
            self.samples.release_all(&self.palette_ref_ids);
        }
    }
}
