use branded_id::U32Id;
use ty_math::{TyVector3I32, TyVector3U32};
use voxcore::{BVoxPaletteCell, VoxObject};

/// Mirrors `from`'s palette references onto `to`, back-filling every voxel with
/// cell `0`. The filler is overwritten when a voxel is re-lived and is never read
/// for an empty voxel, so a uniform filler suffices.
fn copy_palette_refs(from: &VoxObject, to: &mut VoxObject) {
    for (_, palette) in from.iter_palette_refs() {
        to.add_palette_ref(palette, U32Id::<BVoxPaletteCell>::from_u32(0));
    }
}

/// Re-lives `from`'s live voxels on `to`, each shifted by `offset` and carrying
/// its samples. `to` must already share `from`'s palette references and contain
/// every shifted position.
fn copy_voxels(from: &VoxObject, to: &mut VoxObject, offset: [i32; 3]) {
    let references: Vec<_> = from
        .iter_palette_refs()
        .map(|(reference, _)| reference)
        .collect();
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
        let samples: Vec<_> = references
            .iter()
            .map(|&reference| {
                from.voxel_cell(voxel, reference)
                    .expect("a live voxel samples every reference")
            })
            .collect();
        to.retain_voxel(id, &samples)
            .expect("one sample per reference");
    }
}

/// Re-bases a build-volume object to the tight extent of its live voxels,
/// returning the tight object and the original build volume as
/// `(bounds, origin)`.
///
/// The world placement is unchanged: the tight object's `origin` is the build
/// volume's `origin` shifted to the extent's min corner, so the placing node
/// composes its voxels exactly where the build volume placed them. An object with
/// no live voxels yields a degenerate `[0, 0, 0]` grid seated at the build-volume
/// origin. Used by the vmax writer, whose placement math wants the tight runtime
/// grid alongside the build volume the author worked in.
pub(crate) fn tighten(object: &VoxObject) -> (VoxObject, (TyVector3U32, TyVector3I32)) {
    let origin = object.origin();
    let build_volume = (object.bounds(), origin);
    let Some((min, size)) = object.live_extent() else {
        let mut tight = VoxObject::new(object.name().to_owned(), TyVector3U32::new(0, 0, 0))
            .expect("an empty grid is within the dense limit");
        tight.set_origin(origin);
        copy_palette_refs(object, &mut tight);
        return (tight, build_volume);
    };
    let mut tight = VoxObject::new(object.name().to_owned(), size)
        .expect("a sub-grid of an existing grid is within the dense limit");
    tight.set_origin(TyVector3I32::new(
        origin.x + min.x as i32,
        origin.y + min.y as i32,
        origin.z + min.z as i32,
    ));
    copy_palette_refs(object, &mut tight);
    copy_voxels(
        object,
        &mut tight,
        [-(min.x as i32), -(min.y as i32), -(min.z as i32)],
    );
    (tight, build_volume)
}
