use crate::{
    Error, Result, VMaxExtPalette, authored_box, combo_key, edit_grid, min_corner, object_bounds,
    object_palette, object_transform, pivot_origin, view_box,
};
use branded_id::U32Id;
use std::collections::BTreeMap;
use ty_math::{TyTransformF64, TyVector3I32, TyVector3U32};
use vmax::{
    VMaxFile, VMaxObject,
    snapshots::{VMaxVoxel, decode_vmax_snapshots},
};
use voxcore::{BVoxMaterial, BVoxPalette, VoxMain, VoxObject};

/// Builds the voxcore object for one scene object, adding any palettes it
/// introduces to `state`. The object is built in its build volume (the author's
/// `tools.vp`), so its live voxels keep their authored positions inside it.
/// Returns the object, its backing `data` filename, and the transform of the
/// node that places it.
pub(crate) fn build_object(
    serde: &VMaxFile,
    object: &VMaxObject,
    main: &mut VoxMain<()>,
    palette_provenance: &mut BTreeMap<U32Id<BVoxPalette>, VMaxExtPalette>,
) -> Result<(VoxObject, Option<String>, TyTransformF64)> {
    // Voxels come from decoding the object's snapshot edit-log on the fly.
    let voxels: Vec<VMaxVoxel> = if object.data.is_empty() {
        Vec::new()
    } else {
        decode_vmax_snapshots(&serde.contents_files[&object.data].snapshots)?
    };

    // The runtime grid is exactly tight: the occupied voxel extent, re-based so
    // the voxels fill it from the origin. An empty object is a degenerate [0,
    // 0, 0] grid seated at its content box so the placing node still pivots
    // about the recorded center. `origin` offsets that grid from the node so
    // the node transform pivots about the content center.
    let (box_min, bounds) = match min_corner(&voxels) {
        Some(min) => (min, object_bounds(&voxels, min)),
        // An empty object seats at its content box; lacking one, it seats at
        // the build volume `vp.min` so the edit grid still contains the runtime
        // grid, and only at the world origin when it has neither.
        None => (
            authored_box(object)
                .map(|(box_min, _)| box_min)
                .or_else(|| {
                    view_box(serde, object)
                        .map(|vp| [vp.min[0] as i32, vp.min[1] as i32, vp.min[2] as i32])
                })
                .unwrap_or([0, 0, 0]),
            [0, 0, 0],
        ),
    };
    let origin = pivot_origin(box_min, object.center);
    let transform = object_transform(object, box_min, origin);
    // The build volume is the author's `tools.vp`; its `origin` offsets it from
    // the placing node, and `offset` shifts a runtime-grid voxel into it.
    let (edit_bounds, edit_origin) = edit_grid(view_box(serde, object), box_min, origin, bounds);
    let offset = [
        origin[0] - edit_origin[0],
        origin[1] - edit_origin[1],
        origin[2] - edit_origin[2],
    ];
    let data = (!object.data.is_empty()).then(|| object.data.clone());

    let [size_x, size_y, size_z] = edit_bounds;
    let mut vox_object = VoxObject::new(
        object.name.clone(),
        TyVector3U32::new(size_x, size_y, size_z),
    )
    .map_err(|_| {
        Error::invalid(format!(
            "object \"{}\" grid {size_x}x{size_y}x{size_z} exceeds the dense limit of {} cells",
            object.name,
            VoxObject::MAX_GRID_CELLS
        ))
    })?;
    vox_object.set_origin(TyVector3I32::new(
        edit_origin[0],
        edit_origin[1],
        edit_origin[2],
    ));

    if voxels.is_empty() {
        return Ok((vox_object, data, transform));
    }

    // One palette carries the color table and the material list: each voxel
    // samples a single material carrying both its color and its material
    // coefficients, one per distinct color-and-material combination the voxels
    // use.
    let palette = object_palette(serde, object, &voxels, main)?;
    palette_provenance.insert(palette.palette_id, palette.provenance);

    // Back-fill the layer with material 0; the live voxels overwrite theirs.
    vox_object.retain_layer(palette.palette_id, U32Id::<BVoxMaterial>::from_u32(0));

    for voxel in &voxels {
        // Shift the voxel from its model position into the build volume; an
        // out-of-grid result casts to a huge u32 and is rejected by `voxel_id`.
        let position = TyVector3U32::new(
            (voxel.position[0] - box_min[0] + offset[0]) as u32,
            (voxel.position[1] - box_min[1] + offset[1]) as u32,
            (voxel.position[2] - box_min[2] + offset[2]) as u32,
        );
        let voxel_id = vox_object.voxel_id(position).ok_or_else(|| {
            Error::invalid(format!(
                "object \"{}\" voxel ({}, {}, {}) lies outside its build volume",
                object.name, voxel.position[0], voxel.position[1], voxel.position[2]
            ))
        })?;

        let material_id = palette.combo_material_ids[&combo_key(voxel, palette.has_materials)];
        vox_object
            .retain_voxel(voxel_id, &[material_id])
            .map_err(|_| {
                Error::invalid(format!(
                    "object \"{}\" has a malformed voxel sample",
                    object.name
                ))
            })?;
    }

    Ok((vox_object, data, transform))
}
