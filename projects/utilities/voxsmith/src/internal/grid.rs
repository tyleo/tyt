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
    tight.set_origin(TyVector3I32::new(
        origin.x + min.x as i32,
        origin.y + min.y as i32,
        origin.z + min.z as i32,
    ));
    copy_layers(object, &mut tight);
    copy_voxels(
        object,
        &mut tight,
        [-(min.x as i32), -(min.y as i32), -(min.z as i32)],
    );
    (tight, build_volume)
}

/// Mirrors `from`'s layers onto `to`, back-filling every voxel with material
/// `0`. The filler is overwritten when a voxel is re-lived and is never read for
/// an empty voxel, so a uniform filler suffices.
fn copy_layers(from: &VoxObject, to: &mut VoxObject) {
    for (_, palette) in from.iter_layers() {
        to.add_layer(palette, U32Id::<BVoxMaterial>::from_u32(0));
    }
}

/// Re-lives `from`'s live voxels on `to`, each shifted by `offset` and carrying
/// its samples. `to` must already share `from`'s layers and contain every
/// shifted position.
fn copy_voxels(from: &VoxObject, to: &mut VoxObject, offset: [i32; 3]) {
    let layers: Vec<_> = from.iter_layers().map(|(layer, _)| layer).collect();
    for voxel in from.iter_live() {
        let p = from
            .voxel_position(voxel)
            .expect("a live voxel is within the grid");
        let position = TyVector3U32::new(
            (p.x as i32 + offset[0]) as u32,
            (p.y as i32 + offset[1]) as u32,
            (p.z as i32 + offset[2]) as u32,
        );
        let id = to
            .voxel_id(position)
            .expect("the shifted voxel is within the target grid");
        let samples: Vec<_> = layers
            .iter()
            .map(|&layer| {
                from.voxel_material(voxel, layer)
                    .expect("a live voxel samples every layer")
            })
            .collect();
        to.retain_voxel(id, &samples).expect("one sample per layer");
    }
}
