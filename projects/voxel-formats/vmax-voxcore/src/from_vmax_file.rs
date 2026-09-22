use crate::{
    InstanceKey, Result, VMaxExtPalette, VMaxVoxMain, authored_box, build_hierarchy, build_object,
    object_transform, pivot_origin, vmax_ext_from_file,
};
use branded_id::U32Id;
use std::collections::{BTreeMap, HashMap};
use ty_math::TyTransformF64;
use vmax::VMaxFile;
use voxcore::{BVoxObject, BVoxPalette, VoxMain};

/// Loads a Voxel Max document into a [`VMaxVoxMain`], the inverse of
/// [`to_vmax_file`](crate::to_vmax_file). Geometry, palettes, and hierarchy
/// become native voxcore entities. Voxel Max shares voxcore's Y-up right-handed
/// axes, so geometry and transforms copy straight. The rest of the Voxel Max
/// state becomes the ext. Voxel snapshots are decoded to voxels on the fly and
/// palette color tables unpacked as needed. Color indices are 1-based in Voxel
/// Max, so a voxel's color cell is `color_idx - 1`. The material byte is
/// 0-based and used directly.
///
/// Errors on malformed geometry or on a cross-reference the checked insertions
/// reject.
pub fn from_vmax_file(serde: &VMaxFile) -> Result<VMaxVoxMain> {
    let scene = &serde.scene_json_file;
    let mut main = VoxMain::default();

    // One palette per distinct object, by palette id.
    let mut palette_provenance: BTreeMap<U32Id<BVoxPalette>, VMaxExtPalette> = BTreeMap::new();

    // One voxcore object per distinct geometry; instances of one geometry
    // collapse to a single object placed by several nodes.
    let mut object_transforms: Vec<TyTransformF64> = Vec::new();
    let mut object_data: Vec<(U32Id<BVoxObject>, Option<String>)> = Vec::new();
    let mut object_ids: Vec<usize> = Vec::new();
    let mut instances: HashMap<InstanceKey, usize> = HashMap::new();
    for object in &scene.objects {
        let key = InstanceKey::of(object);
        if let Some(&existing) = key.as_ref().and_then(|key| instances.get(key)) {
            // An instance shares the geometry it re-places, so it re-derives
            // only the placing transform from its own content box and pivot.
            let box_min = authored_box(object).map_or([0, 0, 0], |(box_min, _)| box_min);
            let origin = pivot_origin(box_min, object.center);
            object_transforms.push(object_transform(object, box_min, origin));
            object_ids.push(existing);
            continue;
        }
        let (vox_object, data, transform) =
            build_object(serde, object, &mut main, &mut palette_provenance)?;
        let object_id = main.retain_object(vox_object)?;
        object_data.push((object_id, data));
        object_transforms.push(transform);
        object_ids.push(object_id.to_u32() as usize);
        if let Some(key) = key {
            instances.insert(key, object_id.to_u32() as usize);
        }
    }

    // The scene nodes land as one batch: a group's children may sit after
    // it in the listing.
    let (nodes, roots) = build_hierarchy(scene, &object_transforms, &object_ids);
    let node_ids = main.retain_hierarchy_nodes(nodes)?;
    main.set_root_hierarchy_node_ids(roots)?;

    let ext = vmax_ext_from_file(serde, &main, &node_ids, palette_provenance, object_data);
    Ok(main.put_ext(ext))
}

#[cfg(test)]
mod tests {
    use crate::from_vmax_file;
    use branded_id::U32Id;
    use std::collections::BTreeMap;
    use ty_math::TyVector3U32;
    use vmax::{
        VMaxContentsVmaxbFile, VMaxFile, VMaxObject, VMaxSceneJsonFile, VMaxTools, VMaxViewBox,
        snapshots::encode_vmax_snapshots,
    };
    use voxcore::BVoxObject;

    /// An empty object (zero voxels) with no authored content box
    /// (`e_mi`/`e_ma` absent) but a `tools.vp` build volume away from the
    /// origin. Voxel Max opens such a file, so the loader must too: seating
    /// `box_min` at `vp.min` keeps the edit grid containing the runtime grid.
    /// Previously `box_min` fell back to `[0, 0, 0]`, the edit grid was offset
    /// off the runtime point, and the containment validator rejected the load.
    fn empty_object_with_view_box_only() -> VMaxFile {
        let object = VMaxObject {
            name: "empty".to_owned(),
            data: "contents1.vmaxb".to_owned(),
            palette: String::new(),
            history: String::new(),
            id: "o".to_owned(),
            parent_id: None,
            hidden: None,
            position: [0.0, 0.0, 0.0],
            rotation: [0.0, 0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
            ind: [0, 0, 0],
            s: None,
            t_al: String::new(),
            t_pa: String::new(),
            t_pf: String::new(),
            t_po: None,
            center: [128.0, 128.0, 16.0],
            bounds_min: None,
            bounds_max: None,
        };
        let contents = VMaxContentsVmaxbFile {
            snapshots: encode_vmax_snapshots(&[]),
            uuid: "u".to_owned(),
            v: 4,
            tools: Some(VMaxTools {
                vp: Some(VMaxViewBox {
                    min: [112, 112, 0],
                    max: [143, 143, 31],
                }),
                ..Default::default()
            }),
            brush: None,
            cam: None,
            pal: None,
        };
        let mut contents_files = BTreeMap::new();
        contents_files.insert("contents1.vmaxb".to_owned(), contents);
        VMaxFile {
            scene_json_file: VMaxSceneJsonFile {
                v: 4,
                objects: vec![object],
                ..Default::default()
            },
            contents_files,
            palette_settings_files: BTreeMap::new(),
            palette_png_files: BTreeMap::new(),
            history_vmaxhb_files: BTreeMap::new(),
            history_vmaxhvsb_files: BTreeMap::new(),
            history_vmaxhvsc_files: BTreeMap::new(),
            selection_vmaxb_files: BTreeMap::new(),
            thumbnail_png: None,
            contents_vmax_pngs: BTreeMap::new(),
            group_pngs: BTreeMap::new(),
        }
    }

    #[test]
    fn empty_object_without_content_box_loads_from_its_view_box() {
        let main = from_vmax_file(&empty_object_with_view_box_only())
            .expect("an empty object with only a build volume must load");
        let object_id = U32Id::<BVoxObject>::from_u32(0);
        let object = main.object(object_id).expect("the one object");
        // The object's grid is the build volume (the 32^3 `vp`); it has no live
        // voxels, so its derived runtime extent is empty.
        assert_eq!(object.bounds(), TyVector3U32::new(32, 32, 32));
        assert_eq!(object.live_extent(), None);
    }
}
