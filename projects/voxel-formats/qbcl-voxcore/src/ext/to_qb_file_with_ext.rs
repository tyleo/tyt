use crate::{Result, ext::QbVoxMain, write_qb};
use qbcl::qb::QbFile;

/// Writes a [`QbVoxMain`] back to a decoded Qubicle Binary [`QbFile`], the
/// inverse of [`from_qb_file_with_ext`](crate::ext::from_qb_file_with_ext)
/// and the typed form of [`to_qb_file`](crate::to_qb_file). A loaded file
/// writes back exactly through its ext. An object retained after the load is
/// written as a synthesized matrix. A state carrying no ext writes a
/// synthesized file.
///
/// Errors if:
///
/// 1. the ext's matrix entries do not line up with the objects
/// 2. a visibility list does not match its object
/// 3. the header encodes visibility masks and an object has no entry
pub fn to_qb_file_with_ext(state: &QbVoxMain) -> Result<QbFile> {
    write_qb(state, state.ext().as_ref())
}

#[cfg(test)]
mod tests {
    use crate::ext::{QbVoxMain, from_qb_file_with_ext, to_qb_file_with_ext};
    use branded_id::U32Id;
    use qbcl::qb::{QbColorFormat, QbFile, QbMatrix, QbVoxel, QbZAxisOrientation};
    use ty_math::{TyTransformF64, TyVector3F64, TyVector3U32};
    use voxcore::{
        BVoxHierarchyNode, BVoxMaterial, BVoxObject, BVoxPalette, VoxHierarchyNode, VoxObject,
    };

    /// A file with two matrices: a `[2, 1, 1]` grid with two solid voxels, one
    /// carrying a non-default visibility mask, and a single-voxel grid.
    fn sample_file() -> QbFile {
        QbFile {
            version: 257,
            color_format: QbColorFormat::Bgra,
            z_axis_orientation: QbZAxisOrientation::RightHanded,
            compressed: true,
            visibility_mask_encoded: true,
            matrices: vec![
                QbMatrix {
                    name: "m0".to_owned(),
                    size: [2, 1, 1],
                    position: [1, 2, 3],
                    voxels: vec![
                        QbVoxel::new(10, 20, 30),
                        QbVoxel {
                            r: 1,
                            g: 2,
                            b: 3,
                            visibility: 0x3f,
                        },
                    ],
                },
                QbMatrix {
                    name: "m1".to_owned(),
                    size: [1, 1, 1],
                    position: [-1, -1, -1],
                    voxels: vec![QbVoxel::new(40, 50, 60)],
                },
            ],
        }
    }

    #[test]
    fn round_trips_through_vox_state() {
        let file = sample_file();
        let state = from_qb_file_with_ext(&file).unwrap();
        assert_eq!(to_qb_file_with_ext(&state).unwrap(), file);
    }

    #[test]
    fn round_trips_the_default_file() {
        let file = QbFile::default();
        let state = from_qb_file_with_ext(&file).unwrap();
        assert_eq!(to_qb_file_with_ext(&state).unwrap(), file);
    }

    /// A state carrying no ext writes a synthesized file.
    #[test]
    fn synthesizes_without_an_ext() {
        let file = to_qb_file_with_ext(&QbVoxMain::default()).unwrap();
        assert!(file.matrices.is_empty());
    }

    /// Retains a one-voxel object of the sample's second color under a root
    /// node named `placed` at a fractional translation.
    fn retain_placed_object(state: &mut QbVoxMain) {
        let mut object = VoxObject::new("added".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        object.retain_layer(
            U32Id::<BVoxPalette>::from_u32(0),
            U32Id::<BVoxMaterial>::from_u32(0),
        );
        let voxel_id = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
        object
            .retain_voxel(voxel_id, &[U32Id::from_u32(1)])
            .unwrap();
        let object_id = state.retain_object(object).unwrap();

        let node_id = state
            .retain_hierarchy_node(VoxHierarchyNode {
                name: "placed".to_owned(),
                child_node_ids: Vec::new(),
                child_object_ids: vec![object_id],
                transform: TyTransformF64::from_translation(TyVector3F64::new(3.4, -2.6, 16.0)),
            })
            .unwrap();
        state.push_root_hierarchy_node_id(node_id).unwrap();
    }

    /// The mutations `vxl to` makes for a selection. The survivor's entry
    /// stays aligned.
    #[test]
    fn a_released_object_leaves_the_survivors_provenance_aligned() {
        let file = sample_file();
        let mut state = from_qb_file_with_ext(&file).unwrap();
        let node_id = U32Id::<BVoxHierarchyNode>::from_u32(0);

        state
            .set_root_hierarchy_node_ids(vec![U32Id::from_u32(1)])
            .unwrap();
        let mut emptied = state.hierarchy_node(node_id).unwrap().clone();
        emptied.child_object_ids.clear();
        state.set_hierarchy_node(node_id, emptied).unwrap();
        state.release_hierarchy_node(node_id).unwrap();
        state
            .release_object(U32Id::<BVoxObject>::from_u32(0))
            .unwrap();
        state.gc();

        let ext = state.ext().as_ref().unwrap();
        assert_eq!(ext.matrices.len(), 1);
        assert_eq!(ext.matrices[0].as_ref().unwrap().name, "m1");

        let mut want = file;
        want.matrices.remove(0);
        assert_eq!(to_qb_file_with_ext(&state).unwrap(), want);
    }

    /// An object retained after the load has no entry. The writer emits a
    /// matrix like a synthesized one.
    #[test]
    fn an_object_retained_after_the_load_writes_a_synthesized_matrix() {
        let mut file = sample_file();
        file.visibility_mask_encoded = false;
        let mut state = from_qb_file_with_ext(&file).unwrap();

        retain_placed_object(&mut state);

        assert_eq!(state.ext().as_ref().unwrap().matrices[2], None);

        let mut want = file;
        want.matrices.push(QbMatrix {
            name: "placed".to_owned(),
            size: [1, 1, 1],
            position: [3, -3, 16],
            voxels: vec![QbVoxel::new(1, 2, 3)],
        });
        assert_eq!(to_qb_file_with_ext(&state).unwrap(), want);
    }

    /// A moved object carries its entry along.
    #[test]
    fn a_moved_object_keeps_its_matrix() {
        let file = sample_file();
        let mut state = from_qb_file_with_ext(&file).unwrap();

        state
            .move_object(U32Id::<BVoxObject>::from_u32(1), 0)
            .unwrap();

        let mut want = file;
        want.matrices.swap(0, 1);
        assert_eq!(to_qb_file_with_ext(&state).unwrap(), want);
    }

    /// An ext out of step with the objects errors instead of writing a guess.
    #[test]
    fn an_ext_out_of_step_with_its_objects_errors() {
        let file = sample_file();

        let mut state = from_qb_file_with_ext(&file).unwrap();
        let mut ext = state.ext().clone().unwrap();
        ext.matrices.pop();
        state.set_ext(Some(ext));
        assert!(to_qb_file_with_ext(&state).is_err());

        let mut state = from_qb_file_with_ext(&file).unwrap();
        let mut ext = state.ext().clone().unwrap();
        ext.matrices[0].as_mut().unwrap().visibility.pop();
        state.set_ext(Some(ext));
        assert!(to_qb_file_with_ext(&state).is_err());

        // The sample encodes visibility masks. A retained object has none.
        let mut state = from_qb_file_with_ext(&file).unwrap();
        retain_placed_object(&mut state);
        assert!(to_qb_file_with_ext(&state).is_err());
    }
}
