use crate::{
    Error, Result, VoxelMaxExt, VoxelMaxExtWrapper, VoxelMaxNode, VoxelMaxObjectState,
    VoxelMaxPalette, to_vox_value,
};
use branded_id::U32Id;
use std::collections::HashMap;
use ty_math::{TyQuaternionF64, TyTransformF64, TyVector3F64, TyVector3U32};
use vmax::{
    VMaxCodecContentsVmaxbFile, VMaxCodecFile, VMaxCodecVoxel, VMaxGroup, VMaxObject,
    VMaxSceneJsonFile,
};
use voxcore::{
    BVoxHierarchyNode, BVoxObject, BVoxPalette, BVoxPaletteCell, VoxHierarchyNode, VoxObject,
    VoxPalette, VoxState, VoxValue,
};

/// Number of cells in a Voxel Max color palette. A `palette*.png` is 256x1 RGBA,
/// and a placeholder palette covers every color index.
const COLOR_CELLS: usize = 256;

/// The color used for every cell of a placeholder palette when an object's
/// colors are missing, so the color indices are still preserved.
const PLACEHOLDER_COLOR: &str = "#FFFFFFFF";

/// Loads a fully decoded Voxel Max document into a [`VoxState`].
///
/// Geometry, palettes, and hierarchy become native voxcore entities. The Voxel
/// Max state with no native voxcore home rides in a `voxel-max` ext so the
/// document can be written back exactly.
///
/// Errors on malformed geometry or if
/// [`VoxState::validate`](voxcore::VoxState::validate) rejects the result.
pub fn vox_state_from_vmax_codec_file(codec: &VMaxCodecFile) -> Result<VoxState> {
    let scene = &codec.scene_json_file;
    let mut state = VoxState::default();

    // Palettes are shared across objects and deduped by source filename. Their
    // names, aligned by palette id, feed the ext.
    let mut palette_id_by_source: HashMap<String, usize> = HashMap::new();
    let mut palette_names: Vec<Option<String>> = Vec::new();

    // One voxcore object per distinct geometry; instances of one geometry
    // collapse to a single object placed by several nodes.
    let mut object_transforms: Vec<TyTransformF64> = Vec::new();
    let mut object_refs: Vec<usize> = Vec::new();
    let mut object_data: Vec<Option<String>> = Vec::new();
    let mut instances: HashMap<InstanceKey, usize> = HashMap::new();
    for object in &scene.objects {
        let key = instance_key(object);
        if let Some(&existing) = key.as_ref().and_then(|key| instances.get(key)) {
            let (box_min, _) = authored_box(object).expect("instance key implies authored bounds");
            object_transforms.push(object_transform(object, box_min));
            object_refs.push(existing);
            continue;
        }
        let (vox_object, data, transform) = build_object(
            codec,
            object,
            &mut state,
            &mut palette_id_by_source,
            &mut palette_names,
        )?;
        let object_id = state.add_object(vox_object).to_u32() as usize;
        object_data.push(data);
        object_transforms.push(transform);
        object_refs.push(object_id);
        if let Some(key) = key {
            instances.insert(key, object_id);
        }
    }

    let (nodes, roots) = build_hierarchy(scene, &object_transforms, &object_refs);
    for node in nodes {
        state.add_hierarchy_node(node);
    }
    state.set_root_hierarchy_nodes(roots);

    // Each object's preserved editor state, aligned by object id with the
    // objects, read off the decoded contents files.
    let object_states: Vec<Option<VoxelMaxObjectState>> = object_data
        .iter()
        .map(|data| {
            data.as_deref()
                .and_then(|data| codec.contents_files.get(data))
                .map(object_state_from_contents)
        })
        .collect();

    let voxel_max = voxel_max_ext(scene, &palette_names, &object_states);
    let ext = to_vox_value(&VoxelMaxExtWrapper { voxel_max })?;
    state.set_ext(Some(ext));

    state.validate()?;
    Ok(state)
}

/// Captures the editor state of a decoded contents file for the ext.
fn object_state_from_contents(data: &VMaxCodecContentsVmaxbFile) -> VoxelMaxObjectState {
    VoxelMaxObjectState {
        uuid: data.uuid.clone(),
        v: data.v,
        tools: data.tools.clone(),
        brush: data.brush.clone(),
        cam: data.cam.clone(),
    }
}

/// Builds the voxcore object for one scene object, adding any palettes it
/// introduces to `state`. Returns the object, its backing `data` filename, and
/// the transform of the node that places it.
fn build_object(
    codec: &VMaxCodecFile,
    object: &VMaxObject,
    state: &mut VoxState,
    palette_id_by_source: &mut HashMap<String, usize>,
    palette_names: &mut Vec<Option<String>>,
) -> Result<(VoxObject, Option<String>, TyTransformF64)> {
    let voxels: &[VMaxCodecVoxel] = if object.data.is_empty() {
        &[]
    } else {
        &codec.contents_files[&object.data].voxels
    };

    let (box_min, bounds) = object_box(object, voxels, min_corner(voxels))?;
    let transform = object_transform(object, box_min);
    let data = (!object.data.is_empty()).then(|| object.data.clone());

    let [size_x, size_y, size_z] = bounds;
    let mut vox_object = VoxObject::new(
        object.name.clone(),
        TyVector3U32::new(size_x, size_y, size_z),
    )
    .ok_or_else(|| {
        invalid(format!(
            "object \"{}\" grid {size_x}x{size_y}x{size_z} exceeds the dense limit of {} cells",
            object.name,
            VoxObject::MAX_GRID_CELLS
        ))
    })?;

    if voxels.is_empty() {
        return Ok((vox_object, data, transform));
    }

    let color_palette = color_palette(codec, object, state, palette_id_by_source, palette_names);
    let material_palette =
        material_palette(codec, object, state, palette_id_by_source, palette_names);

    // Back-fill each reference with cell 0; the live voxels overwrite theirs.
    let filler = U32Id::<BVoxPaletteCell>::from_u32(0);
    if let Some(id) = color_palette {
        vox_object.add_palette_ref(U32Id::<BVoxPalette>::from_u32(id as u32), filler);
    }
    if let Some(id) = material_palette {
        vox_object.add_palette_ref(U32Id::<BVoxPalette>::from_u32(id as u32), filler);
    }

    for voxel in voxels {
        let position = TyVector3U32::new(
            (voxel.position[0] - box_min[0]) as u32,
            (voxel.position[1] - box_min[1]) as u32,
            (voxel.position[2] - box_min[2]) as u32,
        );
        let voxel_id = vox_object.voxel_id(position).ok_or_else(|| {
            invalid(format!(
                "object \"{}\" voxel ({}, {}, {}) lies outside its bounds",
                object.name, voxel.position[0], voxel.position[1], voxel.position[2]
            ))
        })?;

        // Sample cells follow the reference order: color first, then material.
        let mut cells = Vec::new();
        if color_palette.is_some() {
            cells.push(U32Id::<BVoxPaletteCell>::from_u32(voxel.color_idx as u32));
        }
        if material_palette.is_some() {
            cells.push(U32Id::<BVoxPaletteCell>::from_u32(
                voxel.material_idx as u32,
            ));
        }
        vox_object.retain_voxel(voxel_id, &cells).ok_or_else(|| {
            invalid(format!(
                "object \"{}\" has a malformed voxel sample",
                object.name
            ))
        })?;
    }

    Ok((vox_object, data, transform))
}

/// Returns the shared color palette id for an object, adding it to `state` on
/// first use. The cells come from the `palette*.png` pixels when present, else
/// the material sidecar's `colors` table, and finally a uniform placeholder.
/// `None` only when the object names no palette.
fn color_palette(
    codec: &VMaxCodecFile,
    object: &VMaxObject,
    state: &mut VoxState,
    palette_id_by_source: &mut HashMap<String, usize>,
    palette_names: &mut Vec<Option<String>>,
) -> Option<usize> {
    if object.palette.is_empty() {
        return None;
    }
    if let Some(&id) = palette_id_by_source.get(&object.palette) {
        return Some(id);
    }
    let mut palette = VoxPalette::default();
    palette.add_attribute("rgba".to_owned());
    for hex in color_cells(codec, object) {
        palette
            .add_cell(vec![VoxValue::Text(hex)])
            .expect("one value for one attribute");
    }
    let id = state.add_palette(palette).to_u32() as usize;
    palette_id_by_source.insert(object.palette.clone(), id);
    palette_names.push(None);
    Some(id)
}

/// The `#RRGGBBAA` cells for an object's color palette.
fn color_cells(codec: &VMaxCodecFile, object: &VMaxObject) -> Vec<String> {
    if let Some(png) = codec.palette_png_files.get(&object.palette) {
        return strip_color_terminator(png.0.clone())
            .iter()
            .map(hex)
            .collect();
    }
    if let Some(stem) = object.palette.strip_suffix(".png") {
        let sidecar = format!("{stem}.settings.vmaxpsb");
        if let Some(palette) = codec.palette_settings_files.get(&sidecar)
            && !palette.colors.is_empty()
        {
            return strip_color_terminator(palette.colors.clone())
                .iter()
                .map(hex)
                .collect();
        }
    }
    (0..COLOR_CELLS)
        .map(|_| PLACEHOLDER_COLOR.to_owned())
        .collect()
}

/// Returns the shared material palette id for an object's settings sidecar,
/// adding it to `state` on first use, or `None` when the sidecar is absent or
/// carries no materials. The palette's display name is recorded for the ext.
fn material_palette(
    codec: &VMaxCodecFile,
    object: &VMaxObject,
    state: &mut VoxState,
    palette_id_by_source: &mut HashMap<String, usize>,
    palette_names: &mut Vec<Option<String>>,
) -> Option<usize> {
    let stem = object.palette.strip_suffix(".png")?;
    let sidecar = format!("{stem}.settings.vmaxpsb");
    if let Some(&id) = palette_id_by_source.get(&sidecar) {
        return Some(id);
    }
    let settings = codec.palette_settings_files.get(&sidecar)?;
    if settings.materials.is_empty() {
        return None;
    }

    // Dispersion columns are added only when some slot carries an `md` block, so
    // palettes without dispersion are unchanged. Slots lacking `md` fill those
    // columns with null so every row spans every attribute.
    let has_dispersion = settings.materials.iter().any(|m| m.dispersion.is_some());
    let mut palette = VoxPalette::default();
    for attribute in ["metallic", "roughness", "emissive", "shadows"] {
        palette.add_attribute(attribute.to_owned());
    }
    if has_dispersion {
        for attribute in ["ior", "transmission", "absorption"] {
            palette.add_attribute(attribute.to_owned());
        }
    }
    for material in &settings.materials {
        let mut row = vec![
            VoxValue::Number(material.metalness),
            VoxValue::Number(material.roughness),
            VoxValue::Number(material.emission),
            VoxValue::Bool(material.enable_shadows),
        ];
        if has_dispersion {
            match material.dispersion {
                Some(dispersion) => row.extend([
                    VoxValue::Number(dispersion.ior),
                    VoxValue::Number(dispersion.transmission),
                    VoxValue::Number(dispersion.absorption),
                ]),
                None => row.extend([VoxValue::Null, VoxValue::Null, VoxValue::Null]),
            }
        }
        palette.add_cell(row).expect("one value per attribute");
    }
    let id = state.add_palette(palette).to_u32() as usize;
    palette_id_by_source.insert(sidecar, id);
    palette_names.push(Some(settings.name.clone()));
    Some(id)
}

/// Builds the `voxel-max` ext payload: the scene-level state, the per-node
/// provenance aligned with the hierarchy nodes, the material-palette names, and
/// the per-object editor states.
fn voxel_max_ext(
    scene: &VMaxSceneJsonFile,
    palette_names: &[Option<String>],
    object_states: &[Option<VoxelMaxObjectState>],
) -> VoxelMaxExt {
    let mut scene_block = scene.clone();
    scene_block.groups = Vec::new();
    scene_block.objects = Vec::new();

    // Aligned with the hierarchy nodes: groups first, then objects.
    let mut hierarchy_nodes: Vec<VoxelMaxNode> = scene.groups.iter().map(node_from_group).collect();
    hierarchy_nodes.extend(scene.objects.iter().map(node_from_object));

    let palettes = palette_names
        .iter()
        .map(|name| name.clone().map(|name| VoxelMaxPalette { name }))
        .collect();

    VoxelMaxExt {
        scene: scene_block,
        hierarchy_nodes,
        palettes,
        object_states: object_states.to_vec(),
    }
}

/// The per-node provenance for a scene object.
fn node_from_object(object: &VMaxObject) -> VoxelMaxNode {
    VoxelMaxNode {
        id: object.id.clone(),
        parent_id: object.parent_id.clone(),
        index: Some(object.ind),
        rotation: Some(object.rotation),
        center: Some(object.center),
        bounds_min: object.bounds_min,
        bounds_max: object.bounds_max,
        alignment: Some(object.t_al.clone()),
        pivot_face: Some(object.t_pf.clone()),
        pivot_align: Some(object.t_pa.clone()),
        selected: object.s,
    }
}

/// The per-node provenance for a scene group.
fn node_from_group(group: &VMaxGroup) -> VoxelMaxNode {
    VoxelMaxNode {
        id: group.id.clone(),
        parent_id: group.parent_id.clone(),
        index: Some(group.ind),
        rotation: Some(group.rotation),
        center: Some(group.center),
        bounds_min: group.bounds_min,
        bounds_max: group.bounds_max,
        alignment: Some(group.t_al.clone()),
        pivot_face: Some(group.t_pf.clone()),
        pivot_align: Some(group.t_pa.clone()),
        selected: group.s,
    }
}

/// Drops the trailing transparent terminator Voxel Max appends to fill a color
/// table to [`COLOR_CELLS`]. That reserved slot is referenced by no voxel, so
/// keeping it would pad the model with a non-color.
fn strip_color_terminator(mut colors: Vec<[u8; 4]>) -> Vec<[u8; 4]> {
    if colors.len() == COLOR_CELLS && colors.last() == Some(&[0, 0, 0, 0]) {
        colors.pop();
    }
    colors
}

/// One `#RRGGBBAA` string for an RGBA color.
fn hex(color: &[u8; 4]) -> String {
    let [r, g, b, a] = color;
    format!("#{r:02X}{g:02X}{b:02X}{a:02X}")
}

/// The minimum `[x, y, z]` corner over `voxels`, or `None` when empty.
fn min_corner(voxels: &[VMaxCodecVoxel]) -> Option<[i32; 3]> {
    voxels.iter().fold(None, |acc, v| {
        let acc = acc.unwrap_or(v.position);
        Some([
            acc[0].min(v.position[0]),
            acc[1].min(v.position[1]),
            acc[2].min(v.position[2]),
        ])
    })
}

/// The `[X, Y, Z]` bounds: the per-axis extent of `voxels` relative to `box_min`.
fn object_bounds(voxels: &[VMaxCodecVoxel], box_min: [i32; 3]) -> [u32; 3] {
    let mut bounds = [1u32; 3];
    for v in voxels {
        bounds[0] = bounds[0].max((v.position[0] - box_min[0] + 1) as u32);
        bounds[1] = bounds[1].max((v.position[1] - box_min[1] + 1) as u32);
        bounds[2] = bounds[2].max((v.position[2] - box_min[2] + 1) as u32);
    }
    bounds
}

/// Identifies scene objects that place the same geometry more than once. Voxel
/// Max instances a model by reusing a `contents*.vmaxb` and its palette across
/// objects, so objects sharing the `data`/`palette` filenames and the same
/// authored box decode to one identical object.
type InstanceKey = (String, String, [i32; 3], [u32; 3]);

/// The [`InstanceKey`] for an object, or `None` when it cannot be instanced.
fn instance_key(object: &VMaxObject) -> Option<InstanceKey> {
    if object.data.is_empty() {
        return None;
    }
    let (box_min, size) = authored_box(object)?;
    Some((object.data.clone(), object.palette.clone(), box_min, size))
}

/// The re-basing origin `round(center + bounds_min)` and `[X, Y, Z]` size from an
/// object's authored Voxel Max bounds, or `None` when it has none.
fn authored_box(object: &VMaxObject) -> Option<([i32; 3], [u32; 3])> {
    let (min, max) = (object.bounds_min?, object.bounds_max?);
    let box_min = [
        (object.center[0] + min[0]).round() as i32,
        (object.center[1] + min[1]).round() as i32,
        (object.center[2] + min[2]).round() as i32,
    ];
    let size = [
        (max[0] - min[0]).round().max(0.0) as u32,
        (max[1] - min[1]).round().max(0.0) as u32,
        (max[2] - min[2]).round().max(0.0) as u32,
    ];
    Some((box_min, size))
}

/// The re-basing origin and `[X, Y, Z]` size for an object. Uses the authored
/// Voxel Max bounds when present so the encoded bounds match vmax exactly; an
/// object with no authored bounds falls back to the tight extent of its voxels.
/// Errors if authored bounds do not enclose every voxel.
fn object_box(
    object: &VMaxObject,
    voxels: &[VMaxCodecVoxel],
    tight_min: Option<[i32; 3]>,
) -> Result<([i32; 3], [u32; 3])> {
    let Some((box_min, size)) = authored_box(object) else {
        return Ok(match tight_min {
            Some(tight) => (tight, object_bounds(voxels, tight)),
            None => ([0, 0, 0], [0, 0, 0]),
        });
    };
    for v in voxels {
        let local = [
            v.position[0] - box_min[0],
            v.position[1] - box_min[1],
            v.position[2] - box_min[2],
        ];
        if (0..3).any(|k| local[k] < 0 || local[k] as i64 >= i64::from(size[k])) {
            return Err(invalid(format!(
                "object '{}' has voxel ({}, {}, {}) outside its Voxel Max bounds \
                 (origin {box_min:?}, size {size:?})",
                object.name, v.position[0], v.position[1], v.position[2]
            )));
        }
    }
    Ok((box_min, size))
}

/// The node transform that places an object authored from the origin, undoing
/// Voxel Max's pivot-about-center placement so the voxels sit at `box_min`.
fn object_transform(object: &VMaxObject, box_min: [i32; 3]) -> TyTransformF64 {
    let rotation = TyQuaternionF64::from_axis_angle(
        TyVector3F64::new(object.rotation[0], object.rotation[1], object.rotation[2]),
        object.rotation[3],
    );
    let scale = object.scale;
    let offset = TyVector3F64::new(
        (box_min[0] as f64 - object.center[0]) * scale[0],
        (box_min[1] as f64 - object.center[1]) * scale[1],
        (box_min[2] as f64 - object.center[2]) * scale[2],
    );
    let rotated = rotation.rotate(offset);
    TyTransformF64::new(
        TyVector3F64::new(
            object.position[0] + rotated.x,
            object.position[1] + rotated.y,
            object.position[2] + rotated.z,
        ),
        rotation,
        TyVector3F64::new(scale[0], scale[1], scale[2]),
    )
}

/// The transform for a scene group, placed directly at its authored position.
fn group_transform(group: &VMaxGroup) -> TyTransformF64 {
    let rotation = TyQuaternionF64::from_axis_angle(
        TyVector3F64::new(group.rotation[0], group.rotation[1], group.rotation[2]),
        group.rotation[3],
    );
    TyTransformF64::new(
        TyVector3F64::new(group.position[0], group.position[1], group.position[2]),
        rotation,
        TyVector3F64::new(group.scale[0], group.scale[1], group.scale[2]),
    )
}

/// Builds the voxcore hierarchy: one node per group then one per object, the
/// latter placing its geometry. `object_refs[i]` is the object that scene object
/// `i` places, so instances share a `child_objects` id. Returns the nodes in id
/// order and the root ids.
fn build_hierarchy(
    scene: &VMaxSceneJsonFile,
    object_transforms: &[TyTransformF64],
    object_refs: &[usize],
) -> (Vec<VoxHierarchyNode>, Vec<U32Id<BVoxHierarchyNode>>) {
    let mut nodes: Vec<VoxHierarchyNode> = Vec::new();
    let mut node_of_id: HashMap<&str, usize> = HashMap::new();
    let mut parents: Vec<Option<&str>> = Vec::new();

    for group in &scene.groups {
        node_of_id.insert(&group.id, nodes.len());
        parents.push(group.parent_id.as_deref());
        nodes.push(VoxHierarchyNode {
            name: group.name.clone(),
            child_nodes: Vec::new(),
            child_objects: Vec::new(),
            transform: group_transform(group),
        });
    }
    for (index, object) in scene.objects.iter().enumerate() {
        node_of_id.insert(&object.id, nodes.len());
        parents.push(object.parent_id.as_deref());
        nodes.push(VoxHierarchyNode {
            name: object.name.clone(),
            child_nodes: Vec::new(),
            child_objects: vec![U32Id::<BVoxObject>::from_u32(object_refs[index] as u32)],
            transform: object_transforms[index],
        });
    }

    let mut roots = Vec::new();
    for (node, parent) in parents.iter().enumerate() {
        match parent.and_then(|pid| node_of_id.get(pid)) {
            Some(&parent_node) => nodes[parent_node]
                .child_nodes
                .push(U32Id::from_u32(node as u32)),
            None => roots.push(U32Id::from_u32(node as u32)),
        }
    }

    (nodes, roots)
}

/// Invalid-data error from a message.
fn invalid(message: String) -> Error {
    Error::Invalid(message)
}
