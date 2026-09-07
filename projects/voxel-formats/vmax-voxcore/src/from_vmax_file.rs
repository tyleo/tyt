use crate::{Result, read_vmax};
use vmax::VMaxFile;
use voxcore::VoxMain;

/// Loads a Voxel Max document into a bare [`VoxMain`]. Geometry, palettes,
/// and hierarchy become native voxcore entities. The rest of the Voxel Max
/// state is dropped, so [`to_vmax_file`](crate::to_vmax_file) writes the
/// state back as a synthesized document. The `ext` feature's
/// `ext::from_vmax_file_with_ext` keeps that state instead.
///
/// Errors on malformed geometry or on a cross-reference the checked
/// insertions reject.
pub fn from_vmax_file(serde: &VMaxFile) -> Result<VoxMain<()>> {
    let (state, _) = read_vmax(serde)?;

    Ok(state)
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
        let state = from_vmax_file(&empty_object_with_view_box_only())
            .expect("an empty object with only a build volume must load");
        let object_id = U32Id::<BVoxObject>::from_u32(0);
        let object = state.object(object_id).expect("the one object");
        // The object's grid is the build volume (the 32^3 `vp`); it has no live
        // voxels, so its derived runtime extent is empty.
        assert_eq!(object.bounds(), TyVector3U32::new(32, 32, 32));
        assert_eq!(object.live_extent(), None);
    }
}
