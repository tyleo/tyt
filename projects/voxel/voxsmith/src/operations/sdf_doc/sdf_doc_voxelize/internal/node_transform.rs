use crate::operations::sdf_doc::SdfPlace;
use ty_math::{TyTransformF64, TyVector3F64};

/// The transform of the node of `place` among `places` on the lattice of cells
/// `voxel_size` across.
pub fn node_transform(places: &[SdfPlace], voxel_size: f64, place: &SdfPlace) -> TyTransformF64 {
    match place.parent {
        Some(parent) => TyTransformF64 {
            position: (place.pivot - places[parent].pivot) / voxel_size,
            rotation: place.rotation,
            ..TyTransformF64::IDENTITY
        },

        None => TyTransformF64 {
            position: place.pivot,
            rotation: place.rotation,
            scale: TyVector3F64::splat(voxel_size),
        },
    }
}
