use crate::{Error, QbVoxMain, Result, qb_placements};
use qbcl::qb::{QbColorFormat, QbFile, QbMatrix, QbVoxel, QbZAxisOrientation};
use voxcore::{VoxMain, VoxObject, color::resolve_cell_color_or_transparent};

/// Writes a [`QbVoxMain`] to a decoded Qubicle Binary [`QbFile`], the inverse
/// of [`from_qb_file`](crate::from_qb_file). A loaded file writes back
/// exactly through its ext: each object emits one matrix, taking its name,
/// position, and per-voxel visibility from the ext and its colors from the
/// palette. A state [`to_qb_vox_main`](crate::to_qb_vox_main) gave its ext
/// writes as a file synthesized from the scene. An object retained after the
/// load has no entry. It emits a matrix like a synthesized one at its first
/// placement.
///
/// Errors if:
///
/// 1. the ext's matrix entries do not line up with the objects
/// 2. a visibility list does not match its object
/// 3. the header encodes visibility masks and an object has no entry
/// 4. an object's `baseColor` draws from a non-color value pool
pub fn to_qb_file(state: &QbVoxMain) -> Result<QbFile> {
    let ext = state.ext();

    let object_count = state.object_count();
    if object_count != ext.matrices.len() {
        return Err(Error::invalid(format!(
            "qb ext has {} matrices but the state has {object_count} objects",
            ext.matrices.len()
        )));
    }

    // The object is the author's build volume, so the written matrix keeps
    // its dimensions and voxel positions directly.
    let mut flattened = None;
    let matrices = state
        .iter_objects()
        .zip(&ext.matrices)
        .enumerate()
        .map(|(index, ((object_id, object), entry))| match entry {
            Some(provenance) => matrix_from_object(
                state,
                object,
                &provenance.name,
                provenance.position,
                Some(&provenance.visibility),
            ),
            None => {
                // A mask says which faces of a voxel show. Only the plain
                // visibility byte has one solid value.
                if ext.visibility_mask_encoded {
                    return Err(Error::invalid(format!(
                        "qb ext has no entry for object {index} and the header encodes visibility masks"
                    )));
                }

                let placement = flattened
                    .get_or_insert_with(|| qb_placements(state))
                    .iter()
                    .find(|placement| placement.object_id == object_id)
                    .expect("every object has a placement");
                matrix_from_object(state, object, &placement.name, placement.position, None)
            }
        })
        .collect::<Result<_>>()?;

    Ok(QbFile {
        version: ext.version,
        color_format: if ext.bgra {
            QbColorFormat::Bgra
        } else {
            QbColorFormat::Rgba
        },
        z_axis_orientation: if ext.right_handed {
            QbZAxisOrientation::RightHanded
        } else {
            QbZAxisOrientation::LeftHanded
        },
        compressed: ext.compressed,
        visibility_mask_encoded: ext.visibility_mask_encoded,
        matrices,
    })
}

/// Rebuilds a matrix grid from an object in `.qb` storage order. Each solid
/// voxel's color comes from the object's `baseColor` layer. `visibility`
/// supplies each live voxel's byte in raster order. Without it every solid
/// voxel takes the plain solid byte. Errors if the visibility count does not
/// match the object's solid voxels.
fn matrix_from_object<T>(
    state: &VoxMain<T>,
    object: &VoxObject,
    name: &str,
    position: [i32; 3],
    visibility: Option<&[u8]>,
) -> Result<QbMatrix> {
    let live_count = object.live_count();
    if let Some(visibility) = visibility
        && visibility.len() != live_count
    {
        return Err(Error::invalid(format!(
            "qb ext has {} visibility bytes but the object has {live_count} solid voxels",
            visibility.len()
        )));
    }

    let [size_x, size_y, size_z] = object.bounds().to_array();
    let volume = size_x as usize * size_y as usize * size_z as usize;
    let mut voxels = vec![QbVoxel::default(); volume];

    let cell_color = resolve_cell_color_or_transparent(state, object)?;
    for (live_index, voxel_id) in object.iter_live().enumerate() {
        let cell = object
            .voxel_position(voxel_id)
            .expect("a live voxel is within the grid");
        // A Qubicle voxel stores no alpha, so the sampled color's alpha is
        // dropped.
        let [r, g, b, _] = cell_color.color(voxel_id);
        let mut voxel = QbVoxel::new(r, g, b);
        if let Some(visibility) = visibility {
            voxel.visibility = visibility[live_index];
        }
        // Storage order: index = x + size_x * (y + size_y * z).
        let index = cell.x as usize
            + size_x as usize * (cell.y as usize + size_y as usize * cell.z as usize);
        voxels[index] = voxel;
    }

    Ok(QbMatrix {
        name: name.to_owned(),
        size: [size_x, size_y, size_z],
        position,
        voxels,
    })
}

#[cfg(test)]
mod tests {
    use crate::{QbVoxMain, from_qb_file, to_qb_file};
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
        let state = from_qb_file(&file).unwrap();
        assert_eq!(to_qb_file(&state).unwrap(), file);
    }

    #[test]
    fn round_trips_the_default_file() {
        let file = QbFile::default();
        let state = from_qb_file(&file).unwrap();
        assert_eq!(to_qb_file(&state).unwrap(), file);
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
        let mut state = from_qb_file(&file).unwrap();
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

        let ext = state.ext();
        assert_eq!(ext.matrices.len(), 1);
        assert_eq!(ext.matrices[0].as_ref().unwrap().name, "m1");

        let mut want = file;
        want.matrices.remove(0);
        assert_eq!(to_qb_file(&state).unwrap(), want);
    }

    /// An object retained after the load has no entry. The writer emits a
    /// matrix like a synthesized one.
    #[test]
    fn an_object_retained_after_the_load_writes_a_synthesized_matrix() {
        let mut file = sample_file();
        file.visibility_mask_encoded = false;
        let mut state = from_qb_file(&file).unwrap();

        retain_placed_object(&mut state);

        assert_eq!(state.ext().matrices[2], None);

        let mut want = file;
        want.matrices.push(QbMatrix {
            name: "placed".to_owned(),
            size: [1, 1, 1],
            position: [3, -3, 16],
            voxels: vec![QbVoxel::new(1, 2, 3)],
        });
        assert_eq!(to_qb_file(&state).unwrap(), want);
    }

    /// A moved object carries its entry along.
    #[test]
    fn a_moved_object_keeps_its_matrix() {
        let file = sample_file();
        let mut state = from_qb_file(&file).unwrap();

        state
            .move_object(U32Id::<BVoxObject>::from_u32(1), 0)
            .unwrap();

        let mut want = file;
        want.matrices.swap(0, 1);
        assert_eq!(to_qb_file(&state).unwrap(), want);
    }

    /// An ext out of step with the objects errors instead of writing a guess.
    #[test]
    fn an_ext_out_of_step_with_its_objects_errors() {
        let file = sample_file();

        let mut state = from_qb_file(&file).unwrap();
        let ext = state.ext_mut();
        ext.matrices.pop();
        assert!(to_qb_file(&state).is_err());

        let mut state = from_qb_file(&file).unwrap();
        let ext = state.ext_mut();
        ext.matrices[0].as_mut().unwrap().visibility.pop();
        assert!(to_qb_file(&state).is_err());

        // The sample encodes visibility masks. A retained object has none.
        let mut state = from_qb_file(&file).unwrap();
        retain_placed_object(&mut state);
        assert!(to_qb_file(&state).is_err());
    }
}
