use branded_id::U32Id;
use ty_math::{TyVector3I32, TyVector3U32};
use voxcore::{BVoxMaterial, VoxObject};

/// Re-bases a build-volume object to the tight extent of its live voxels,
/// returning the tight object and the original build volume as
/// `(bounds, origin)`.
///
/// The world placement is unchanged: the tight object's `origin` is the build
/// volume's `origin` shifted to the extent's min corner, so the placing node
/// composes its voxels exactly where the build volume placed them. An object
/// with no live voxels yields a degenerate `[0, 0, 0]` grid seated at the
/// build-volume origin. Used by the vmax writer, whose placement math wants the
/// tight runtime grid alongside the build volume the author worked in.
pub fn tighten(object: &VoxObject) -> (VoxObject, (TyVector3U32, TyVector3I32)) {
    let origin = object.origin();
    let build_volume = (object.bounds(), origin);
    let Some((min, size)) = object.live_extent() else {
        let mut tight = VoxObject::new(object.name().to_owned(), TyVector3U32::new(0, 0, 0))
            .expect("an empty grid is within the dense limit");
        tight.set_origin(origin);
        copy_layers(object, &mut tight);
        return (tight, build_volume);
    };
    let mut tight = VoxObject::new(object.name().to_owned(), size)
        .expect("a sub-grid of an existing grid is within the dense limit");
    tight.set_origin(origin + min.as_ivec3());
    copy_layers(object, &mut tight);
    copy_voxels(object, &mut tight, -min.as_ivec3());
    (tight, build_volume)
}

/// Mirrors `from`'s layers onto `to`, back-filling every voxel with material
/// `0`. The filler is overwritten when a voxel is re-lived and is never read for
/// an empty voxel, so a uniform filler suffices.
fn copy_layers(from: &VoxObject, to: &mut VoxObject) {
    for (_, palette_id) in from.iter_layers() {
        to.retain_layer(palette_id, U32Id::<BVoxMaterial>::from_u32(0));
    }
}

/// Re-lives `from`'s live voxels on `to`, each shifted by `offset` and carrying
/// its samples. `to` must already share `from`'s layers and contain every
/// shifted position.
fn copy_voxels(from: &VoxObject, to: &mut VoxObject, offset: TyVector3I32) {
    let layer_ids: Vec<_> = from.iter_layers().map(|(layer_id, _)| layer_id).collect();
    for voxel_id in from.iter_live() {
        let p = from
            .voxel_position(voxel_id)
            .expect("a live voxel is within the grid");
        let position = (p.as_ivec3() + offset).as_uvec3();
        let shifted_voxel_id = to
            .voxel_id(position)
            .expect("the shifted voxel is within the target grid");
        let samples: Vec<_> = layer_ids
            .iter()
            .map(|&layer_id| {
                from.voxel_material(voxel_id, layer_id)
                    .expect("a live voxel samples every layer")
            })
            .collect();
        to.retain_voxel(shifted_voxel_id, &samples)
            .expect("one sample per layer");
    }
}
