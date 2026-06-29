use branded_id::U32Id;
use ty_math::{TyVector3I32, TyVector3U32};
use voxcore::{BVoxPaletteCell, VoxEditObject, VoxObject};

/// The tight live-voxel extent of `object` as `(min_corner, [X, Y, Z] size)`, or
/// `None` when it has no live voxels.
fn live_extent(object: &VoxObject) -> Option<([u32; 3], [u32; 3])> {
    let mut live = object.iter_live().map(|id| {
        let p = object
            .voxel_position(id)
            .expect("a live voxel is within the grid");
        [p.x, p.y, p.z]
    });
    let first = live.next()?;
    let (mut min, mut max) = (first, first);
    for p in live {
        for axis in 0..3 {
            min[axis] = min[axis].min(p[axis]);
            max[axis] = max[axis].max(p[axis]);
        }
    }
    Some((
        min,
        [
            max[0] - min[0] + 1,
            max[1] - min[1] + 1,
            max[2] - min[2] + 1,
        ],
    ))
}

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

/// Re-bases an object to the tight extent of its live voxels, returning the tight
/// object and the edit grid that keeps its original grid as the build volume.
///
/// The world placement is unchanged: the tight object's `origin` is the extent's
/// min corner, so the placing node composes its voxels exactly where the source
/// grid placed them. An object with no live voxels yields a degenerate `[0, 0, 0]`
/// runtime grid. Used by the `from_<format>` importers to satisfy the exactly-tight
/// `bounds` invariant while preserving the author's grid in the edit state.
pub(crate) fn tighten(object: &VoxObject) -> (VoxObject, VoxEditObject) {
    let edit = VoxEditObject {
        bounds: object.bounds(),
        origin: TyVector3I32::new(0, 0, 0),
    };
    let Some((min, size)) = live_extent(object) else {
        let mut tight = VoxObject::new(object.name().to_owned(), TyVector3U32::new(0, 0, 0))
            .expect("an empty grid is within the dense limit");
        copy_palette_refs(object, &mut tight);
        return (tight, edit);
    };
    let mut tight = VoxObject::new(
        object.name().to_owned(),
        TyVector3U32::new(size[0], size[1], size[2]),
    )
    .expect("a sub-grid of an existing grid is within the dense limit");
    tight.set_origin(TyVector3I32::new(
        min[0] as i32,
        min[1] as i32,
        min[2] as i32,
    ));
    copy_palette_refs(object, &mut tight);
    copy_voxels(
        object,
        &mut tight,
        [-(min[0] as i32), -(min[1] as i32), -(min[2] as i32)],
    );
    (tight, edit)
}

/// Restores an object to its source grid (the build volume in `edit`), undoing
/// [`tighten`]. The result has the edit grid's bounds with each voxel back at its
/// source position and a zero `origin`, so a `to_<format>` writer that reads
/// `bounds` and voxel positions reproduces the source grid without consulting
/// `origin`.
pub(crate) fn untighten(object: &VoxObject, edit: VoxEditObject) -> VoxObject {
    let origin = object.origin();
    let offset = [
        origin.x - edit.origin.x,
        origin.y - edit.origin.y,
        origin.z - edit.origin.z,
    ];
    let mut source = VoxObject::new(object.name().to_owned(), edit.bounds)
        .expect("the source grid was a valid grid");
    copy_palette_refs(object, &mut source);
    copy_voxels(object, &mut source, offset);
    source
}
