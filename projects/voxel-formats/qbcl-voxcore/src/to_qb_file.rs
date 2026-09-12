use crate::{Error, QbVoxMain, Result, SOLID_VISIBILITY, face_mask};
use branded_id::U32Id;
use qbcl::qb::{QbColorFormat, QbFile, QbMatrix, QbVoxel, QbZAxisOrientation};
use std::collections::HashSet;
use voxcore::{
    BVoxHierarchyNode, BVoxObject, VoxHierarchyNode, VoxObject,
    color::resolve_cell_color_or_transparent,
};

/// Writes a [`QbVoxMain`] to a decoded Qubicle Binary [`QbFile`], the inverse
/// of [`from_qb_file`](crate::from_qb_file). The scene holds the shape a
/// `.qb` file holds: one root per matrix, each placing one object and no
/// child node, in matrix order. Each root emits one matrix named for the
/// root at the root's translation rounded to whole voxels, with its colors
/// from the palette. Under visibility masks each solid voxel's byte says
/// which faces its neighbors leave uncovered; otherwise every solid voxel
/// takes the plain solid byte. The header comes from the ext. A state from
/// [`to_qb_vox_main`](crate::to_qb_vox_main) has that shape.
///
/// Errors if:
///
/// 1. a root lists a child node or places a number of objects other than one
/// 2. an object is placed by no root or by more than one
/// 3. an object's `baseColor` draws from a non-color value pool
pub fn to_qb_file(main: &QbVoxMain) -> Result<QbFile> {
    let ext = main.ext();

    let mut placed: HashSet<U32Id<BVoxObject>> = HashSet::with_capacity(main.object_count());
    let matrices = main
        .root_hierarchy_node_ids()
        .iter()
        .map(|&root_id| {
            let root = main
                .hierarchy_node(root_id)
                .expect("a root is one of the state's nodes");
            let object_id = placed_object_id(root_id, root)?;

            if !placed.insert(object_id) {
                return Err(Error::invalid(format!(
                    "object {} is placed by more than one root, but a .qb matrix holds one grid",
                    object_id.to_u32()
                )));
            }

            let object = main
                .object(object_id)
                .expect("a placed object is one of the state's");

            matrix_from_object(main, object, root, ext.visibility_mask_encoded)
        })
        .collect::<Result<Vec<_>>>()?;

    if let Some((object_id, _)) = main
        .iter_objects()
        .find(|(object_id, _)| !placed.contains(object_id))
    {
        return Err(Error::invalid(format!(
            "object {} is placed by no root, so it has no .qb matrix",
            object_id.to_u32()
        )));
    }

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

/// The one object a root places. Errors on a child node or on any other
/// number of objects, because a `.qb` matrix is one grid with no children.
fn placed_object_id(
    root_id: U32Id<BVoxHierarchyNode>,
    root: &VoxHierarchyNode,
) -> Result<U32Id<BVoxObject>> {
    if !root.child_node_ids.is_empty() {
        return Err(Error::invalid(format!(
            "root node {} lists child nodes, but a .qb file has no hierarchy",
            root_id.to_u32()
        )));
    }

    let [object_id] = root.child_object_ids.as_slice() else {
        return Err(Error::invalid(format!(
            "root node {} places {} objects, but a .qb matrix holds one grid",
            root_id.to_u32(),
            root.child_object_ids.len()
        )));
    };

    Ok(*object_id)
}

/// Rebuilds a matrix grid from an object in `.qb` storage order, named for
/// `root` at its translation. Each solid voxel's color comes from the object's
/// `baseColor` layer. Under `mask_encoded` its byte is the face mask its
/// neighbors leave; otherwise the plain solid byte.
fn matrix_from_object(
    main: &QbVoxMain,
    object: &VoxObject,
    root: &VoxHierarchyNode,
    mask_encoded: bool,
) -> Result<QbMatrix> {
    let [size_x, size_y, size_z] = object.bounds().to_array();
    let volume = size_x as usize * size_y as usize * size_z as usize;
    let mut voxels = vec![QbVoxel::default(); volume];

    let cell_color = resolve_cell_color_or_transparent(main, object)?;
    for voxel_id in object.iter_live() {
        let cell = object
            .voxel_position(voxel_id)
            .expect("a live voxel is within the grid");
        // A Qubicle voxel stores no alpha, so the sampled color's alpha is
        // dropped.
        let [r, g, b, _] = cell_color.color(voxel_id);
        let visibility = if mask_encoded {
            face_mask(object, voxel_id)
        } else {
            SOLID_VISIBILITY
        };
        // Storage order: index = x + size_x * (y + size_y * z).
        let index = cell.x as usize
            + size_x as usize * (cell.y as usize + size_y as usize * cell.z as usize);
        voxels[index] = QbVoxel {
            r,
            g,
            b,
            visibility,
        };
    }

    Ok(QbMatrix {
        name: root.name.clone(),
        size: [size_x, size_y, size_z],
        position: root.transform.position.round().as_ivec3().to_array(),
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

    /// A solid voxel whose visibility byte is `mask`.
    fn masked(r: u8, g: u8, b: u8, mask: u8) -> QbVoxel {
        QbVoxel {
            r,
            g,
            b,
            visibility: mask,
        }
    }

    /// A file with two matrices under visibility masks: a `[2, 1, 1]` grid
    /// with two solid voxels covering each other along `x`, and a
    /// single-voxel grid. The masks are the ones the grids derive.
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
                    voxels: vec![masked(10, 20, 30, 0x7e & !4), masked(1, 2, 3, 0x7e & !2)],
                },
                QbMatrix {
                    name: "m1".to_owned(),
                    size: [1, 1, 1],
                    position: [-1, -1, -1],
                    voxels: vec![masked(40, 50, 60, 0x7e)],
                },
            ],
        }
    }

    #[test]
    fn round_trips_through_vox_main() {
        let file = sample_file();
        let main = from_qb_file(&file).unwrap();
        assert_eq!(to_qb_file(&main).unwrap(), file);
    }

    #[test]
    fn round_trips_the_default_file() {
        let file = QbFile::default();
        let main = from_qb_file(&file).unwrap();
        assert_eq!(to_qb_file(&main).unwrap(), file);
    }

    /// A plain file writes the solid byte for every voxel.
    #[test]
    fn round_trips_plain_visibility() {
        let mut file = sample_file();
        file.visibility_mask_encoded = false;
        for matrix in &mut file.matrices {
            for voxel in &mut matrix.voxels {
                voxel.visibility = 255;
            }
        }

        let main = from_qb_file(&file).unwrap();
        assert_eq!(to_qb_file(&main).unwrap(), file);
    }

    /// Retains a one-voxel object of the sample's second color under a root
    /// node named `placed` at a fractional translation.
    fn retain_placed_object(main: &mut QbVoxMain) {
        let mut object = VoxObject::new("added".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        object.retain_layer(
            U32Id::<BVoxPalette>::from_u32(0),
            U32Id::<BVoxMaterial>::from_u32(0),
        );
        let voxel_id = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
        object
            .retain_voxel(voxel_id, &[U32Id::from_u32(1)])
            .unwrap();
        let object_id = main.retain_object(object).unwrap();

        let node_id = main
            .retain_hierarchy_node(VoxHierarchyNode {
                name: "placed".to_owned(),
                child_node_ids: Vec::new(),
                child_object_ids: vec![object_id],
                transform: TyTransformF64::from_translation(TyVector3F64::new(3.4, -2.6, 16.0)),
            })
            .unwrap();
        main.push_root_hierarchy_node_id(node_id).unwrap();
    }

    /// The mutations `vxl to` makes for a selection. The survivor writes.
    #[test]
    fn a_released_object_leaves_the_survivor() {
        let file = sample_file();
        let mut main = from_qb_file(&file).unwrap();
        let node_id = U32Id::<BVoxHierarchyNode>::from_u32(0);

        main.set_root_hierarchy_node_ids(vec![U32Id::from_u32(1)])
            .unwrap();
        let mut emptied = main.hierarchy_node(node_id).unwrap().clone();
        emptied.child_object_ids.clear();
        main.set_hierarchy_node(node_id, emptied).unwrap();
        main.release_hierarchy_node(node_id).unwrap();
        main.release_object(U32Id::<BVoxObject>::from_u32(0))
            .unwrap();
        main.gc().unwrap();

        let mut want = file;
        want.matrices.remove(0);
        assert_eq!(to_qb_file(&main).unwrap(), want);
    }

    /// An object retained after the load writes its matrix from the scene,
    /// masks included.
    #[test]
    fn an_object_retained_after_the_load_writes_its_matrix() {
        let file = sample_file();
        let mut main = from_qb_file(&file).unwrap();

        retain_placed_object(&mut main);

        let mut want = file;
        want.matrices.push(QbMatrix {
            name: "placed".to_owned(),
            size: [1, 1, 1],
            position: [3, -3, 16],
            voxels: vec![masked(1, 2, 3, 0x7e)],
        });
        assert_eq!(to_qb_file(&main).unwrap(), want);
    }

    /// The matrices follow the roots. Moving an object in its listing changes
    /// nothing. Reordering the roots reorders the matrices.
    #[test]
    fn the_matrices_follow_the_roots() {
        let file = sample_file();
        let mut main = from_qb_file(&file).unwrap();

        main.move_object(U32Id::<BVoxObject>::from_u32(1), 0)
            .unwrap();
        assert_eq!(to_qb_file(&main).unwrap(), file);

        main.set_root_hierarchy_node_ids(vec![U32Id::from_u32(1), U32Id::from_u32(0)])
            .unwrap();
        let mut want = file;
        want.matrices.swap(0, 1);
        assert_eq!(to_qb_file(&main).unwrap(), want);
    }

    /// A scene outside the `.qb` shape errors instead of writing a guess.
    #[test]
    fn a_scene_outside_the_qb_shape_errors() {
        let file = sample_file();
        let node = |index: u32| U32Id::<BVoxHierarchyNode>::from_u32(index);
        let object = |index: u32| U32Id::<BVoxObject>::from_u32(index);

        // A root with a child node.
        let mut main = from_qb_file(&file).unwrap();
        let mut root = main.hierarchy_node(node(0)).unwrap().clone();
        root.child_node_ids.push(node(1));
        main.set_hierarchy_node(node(0), root).unwrap();
        main.set_root_hierarchy_node_ids(vec![node(0)]).unwrap();
        assert!(to_qb_file(&main).is_err());

        // A root placing two objects.
        let mut main = from_qb_file(&file).unwrap();
        let mut root = main.hierarchy_node(node(0)).unwrap().clone();
        root.child_object_ids.push(object(1));
        main.set_hierarchy_node(node(0), root).unwrap();
        assert!(to_qb_file(&main).is_err());

        // An object placed by two roots.
        let mut main = from_qb_file(&file).unwrap();
        let mut root = main.hierarchy_node(node(1)).unwrap().clone();
        root.child_object_ids = vec![object(0)];
        main.set_hierarchy_node(node(1), root).unwrap();
        assert!(to_qb_file(&main).is_err());

        // An object no root places.
        let mut main = from_qb_file(&file).unwrap();
        main.set_root_hierarchy_node_ids(vec![node(0)]).unwrap();
        assert!(to_qb_file(&main).is_err());
    }
}
