use crate::{QbclExtSource, Result, ext::QbclVoxMain};
use qbcl::qbcl::QbclFile;

/// Writes a [`QbclVoxMain`] back to a decoded Qubicle Construction Library
/// [`QbclFile`], the inverse of
/// [`from_qbcl_file_with_ext`](crate::ext::from_qbcl_file_with_ext) and the
/// typed form of [`to_qbcl_file`](crate::to_qbcl_file). A loaded file writes
/// back exactly through its ext. A node retained after the load is written as
/// a synthesized node.
///
/// Errors if:
///
/// 1. the ext's node entries do not line up with the hierarchy
/// 2. the state does not have exactly one root
/// 3. a mask list does not match its object
/// 4. a model entry's node places an object under a transform chunk other
///    than the default
pub fn to_qbcl_file_with_ext(state: &QbclVoxMain) -> Result<QbclFile> {
    QbclExtSource::write_qbcl(state)
}

#[cfg(test)]
mod tests {
    use crate::{
        SOLID_MASK,
        ext::{
            QbclExtNode, QbclExtNodeBody, QbclVoxMain, from_qbcl_file_with_ext,
            to_qbcl_file_with_ext,
        },
    };
    use branded_id::U32Id;
    use qbcl::qbcl::{
        QbclColor, QbclCompound, QbclFile, QbclMatrix, QbclMetadata, QbclModel, QbclNode,
        QbclNodeBody, QbclThumbnail, QbclVoxel,
    };
    use ty_math::{TyTransformF64, TyVector3F64, TyVector3U32};
    use voxcore::{
        BVoxHierarchyNode, BVoxMaterial, BVoxObject, BVoxPalette, VoxHierarchyNode, VoxObject,
    };

    /// A matrix node with two solid voxels in a `[2, 1, 1]` grid.
    fn matrix_node() -> QbclNode {
        QbclNode {
            name: "matrix".to_owned(),
            visible: true,
            locked: false,
            body: QbclNodeBody::Matrix(QbclMatrix {
                size: [2, 1, 1],
                position: [1, 2, 3],
                pivot: [0.5, 0.0, 0.0],
                voxels: vec![
                    QbclVoxel::new(10, 20, 30, 0x7e),
                    QbclVoxel::new(1, 2, 3, 0x01),
                ],
            }),
        }
    }

    /// The empty model inside the compound.
    fn leaf_node() -> QbclNode {
        QbclNode {
            name: "leaf".to_owned(),
            visible: true,
            locked: false,
            body: QbclNodeBody::Model(QbclModel::default()),
        }
    }

    /// A one-voxel matrix node inside the compound.
    fn inner_node() -> QbclNode {
        QbclNode {
            name: "inner".to_owned(),
            visible: true,
            locked: false,
            body: QbclNodeBody::Matrix(QbclMatrix {
                size: [1, 1, 1],
                position: [0, 1, 0],
                pivot: [0.0, 0.0, 0.0],
                voxels: vec![QbclVoxel::new(1, 2, 3, 0x7e)],
            }),
        }
    }

    /// A compound node carrying one baked voxel, an empty model child, and a
    /// matrix child.
    fn compound_node() -> QbclNode {
        QbclNode {
            name: "compound".to_owned(),
            visible: false,
            locked: true,
            body: QbclNodeBody::Compound(QbclCompound {
                matrix: QbclMatrix {
                    size: [1, 1, 1],
                    position: [-1, -2, -3],
                    pivot: [0.0, 0.0, 0.0],
                    voxels: vec![QbclVoxel::new(40, 50, 60, 0xff)],
                },
                children: vec![leaf_node(), inner_node()],
            }),
        }
    }

    /// A file exercising a model root grouping a matrix and a compound, with a
    /// thumbnail, metadata, and a guid. The load numbers the nodes:
    ///
    /// 0. `matrix`
    /// 1. `leaf`
    /// 2. `inner`
    /// 3. `compound`
    /// 4. `root`
    ///
    /// The objects load as 0 `matrix`, 1 `inner`, and 2 `compound`.
    fn sample_file() -> QbclFile {
        QbclFile {
            program_version: 0x0102_0304,
            file_version: 2,
            thumbnail: QbclThumbnail {
                width: 2,
                height: 1,
                pixels: vec![QbclColor::new(1, 2, 3, 4), QbclColor::new(5, 6, 7, 8)],
            },
            metadata: QbclMetadata {
                title: "Title".to_owned(),
                description: "Desc".to_owned(),
                tags: String::new(),
                author: "Author".to_owned(),
                company: String::new(),
                website: String::new(),
                copyright: "2026".to_owned(),
            },
            guid: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
            root: QbclNode {
                name: "root".to_owned(),
                visible: true,
                locked: false,
                body: QbclNodeBody::Model(QbclModel {
                    transform: QbclModel::DEFAULT_TRANSFORM,
                    children: vec![matrix_node(), compound_node()],
                }),
            },
        }
    }

    /// The root model's children of `file`.
    fn root_children(file: &mut QbclFile) -> &mut Vec<QbclNode> {
        let QbclNodeBody::Model(root) = &mut file.root.body else {
            panic!("the sample roots under a model");
        };
        &mut root.children
    }

    /// The sample's compound body in `file`.
    fn compound(file: &mut QbclFile) -> &mut QbclCompound {
        let QbclNodeBody::Compound(compound) = &mut root_children(file)[1].body else {
            panic!("the sample's compound is a compound");
        };
        compound
    }

    fn node(index: u32) -> U32Id<BVoxHierarchyNode> {
        U32Id::from_u32(index)
    }

    fn object(index: u32) -> U32Id<BVoxObject> {
        U32Id::from_u32(index)
    }

    /// Replaces `node_id`'s children. The name and transform stay.
    fn set_children(
        state: &mut QbclVoxMain,
        node_id: U32Id<BVoxHierarchyNode>,
        child_node_ids: Vec<U32Id<BVoxHierarchyNode>>,
        child_object_ids: Vec<U32Id<BVoxObject>>,
    ) {
        let mut node = state.hierarchy_node(node_id).unwrap().clone();
        node.child_node_ids = child_node_ids;
        node.child_object_ids = child_object_ids;
        state.set_hierarchy_node(node_id, node).unwrap();
    }

    /// Retains a one-voxel object of the sample's second color.
    fn retain_added_object(state: &mut QbclVoxMain) -> U32Id<BVoxObject> {
        let mut object = VoxObject::new("added".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        object.retain_layer(
            U32Id::<BVoxPalette>::from_u32(0),
            U32Id::<BVoxMaterial>::from_u32(0),
        );
        let voxel_id = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
        object
            .retain_voxel(voxel_id, &[U32Id::from_u32(1)])
            .unwrap();

        state.retain_object(object).unwrap()
    }

    /// The synthesized matrix node of `retain_added_object`'s object.
    fn added_node(name: &str, position: [i32; 3]) -> QbclNode {
        QbclNode {
            name: name.to_owned(),
            visible: true,
            locked: false,
            body: QbclNodeBody::Matrix(QbclMatrix {
                size: [1, 1, 1],
                position,
                pivot: [0.0, 0.0, 0.0],
                voxels: vec![QbclVoxel::new(1, 2, 3, SOLID_MASK)],
            }),
        }
    }

    #[test]
    fn round_trips_through_vox_state() {
        let file = sample_file();
        let state = from_qbcl_file_with_ext(&file).unwrap();
        assert_eq!(to_qbcl_file_with_ext(&state).unwrap(), file);
    }

    #[test]
    fn round_trips_the_default_file() {
        let file = QbclFile::default();
        let state = from_qbcl_file_with_ext(&file).unwrap();
        assert_eq!(to_qbcl_file_with_ext(&state).unwrap(), file);
    }

    /// A file whose root is a matrix rather than the conventional model.
    #[test]
    fn round_trips_a_matrix_root() {
        let file = QbclFile {
            root: matrix_node(),
            ..Default::default()
        };
        let state = from_qbcl_file_with_ext(&file).unwrap();
        assert_eq!(to_qbcl_file_with_ext(&state).unwrap(), file);
    }

    /// The mutations `vxl to` makes to keep `matrix`. The survivors' entries
    /// stay aligned through the holes the releases leave and after `gc`.
    #[test]
    fn a_released_node_leaves_the_survivors_provenance_aligned() {
        let file = sample_file();
        let mut state = from_qbcl_file_with_ext(&file).unwrap();

        set_children(&mut state, node(4), vec![node(0)], Vec::new());
        set_children(&mut state, node(3), Vec::new(), Vec::new());
        for index in [1, 2, 3] {
            state.release_hierarchy_node(node(index)).unwrap();
        }
        for index in [1, 2] {
            state.release_object(object(index)).unwrap();
        }

        let mut want = file;
        root_children(&mut want).truncate(1);
        assert_eq!(to_qbcl_file_with_ext(&state).unwrap(), want);

        state.gc();

        let ext = state.ext();
        assert_eq!(ext.nodes.len(), 2);
        assert_eq!(ext.nodes[0].as_ref().unwrap().name, "matrix");
        assert_eq!(to_qbcl_file_with_ext(&state).unwrap(), want);
    }

    /// The mutations `vxl to` makes to keep `inner`. The compound loses its
    /// grid and keeps its placement.
    #[test]
    fn a_compound_whose_grid_is_released_keeps_its_placement() {
        let file = sample_file();
        let mut state = from_qbcl_file_with_ext(&file).unwrap();

        set_children(&mut state, node(4), vec![node(3)], Vec::new());
        set_children(&mut state, node(3), vec![node(2)], Vec::new());
        for index in [0, 1] {
            state.release_hierarchy_node(node(index)).unwrap();
        }
        for index in [0, 2] {
            state.release_object(object(index)).unwrap();
        }
        state.gc();

        let mut want = file;
        let compound = compound(&mut want);
        compound.matrix.size = [0, 0, 0];
        compound.matrix.voxels = Vec::new();
        compound.children = vec![inner_node()];
        root_children(&mut want).remove(0);
        assert_eq!(to_qbcl_file_with_ext(&state).unwrap(), want);
    }

    /// A node retained after the load has no entry. The writer emits a node
    /// like a synthesized one.
    #[test]
    fn a_node_retained_after_the_load_writes_a_synthesized_node() {
        let file = sample_file();
        let mut state = from_qbcl_file_with_ext(&file).unwrap();

        let object_id = retain_added_object(&mut state);
        let node_id = state
            .retain_hierarchy_node(VoxHierarchyNode {
                name: "placed".to_owned(),
                child_node_ids: Vec::new(),
                child_object_ids: vec![object_id],
                transform: TyTransformF64::from_translation(TyVector3F64::new(3.4, -2.6, 16.0)),
            })
            .unwrap();
        set_children(
            &mut state,
            node(4),
            vec![node(0), node(3), node_id],
            Vec::new(),
        );

        assert_eq!(state.ext().nodes[5], None);

        let mut want = file;
        root_children(&mut want).push(added_node("placed", [3, -3, 16]));
        assert_eq!(to_qbcl_file_with_ext(&state).unwrap(), want);
    }

    /// A model entry's node takes the shape a synthesized node would. The
    /// entry's name and flags carry over.
    #[test]
    fn a_model_entry_takes_the_shape_of_its_node() {
        let file = sample_file();
        let mut state = from_qbcl_file_with_ext(&file).unwrap();

        let object_id = retain_added_object(&mut state);
        set_children(&mut state, node(1), Vec::new(), vec![object_id]);

        let mut want = file;
        compound(&mut want).children[0] = added_node("leaf", [0, 0, 0]);
        assert_eq!(to_qbcl_file_with_ext(&state).unwrap(), want);
    }

    /// A matrix or compound carries no transform chunk. The writer refuses to
    /// reshape a model entry whose chunk is not the default.
    #[test]
    fn a_model_entry_with_a_transform_chunk_refuses_a_grid() {
        let mut file = sample_file();
        let QbclNodeBody::Model(leaf) = &mut compound(&mut file).children[0].body else {
            panic!("the sample's leaf is a model");
        };
        leaf.transform = [7; 36];
        let mut state = from_qbcl_file_with_ext(&file).unwrap();
        assert_eq!(to_qbcl_file_with_ext(&state).unwrap(), file);

        let object_id = retain_added_object(&mut state);
        set_children(&mut state, node(1), Vec::new(), vec![object_id]);
        assert!(to_qbcl_file_with_ext(&state).is_err());
    }

    /// A matrix entry's node that gains children or further objects writes a
    /// compound around its grid.
    #[test]
    fn a_matrix_entry_whose_node_gains_children_writes_a_compound() {
        let file = sample_file();
        let mut state = from_qbcl_file_with_ext(&file).unwrap();

        let object_id = retain_added_object(&mut state);
        set_children(&mut state, node(3), vec![node(2)], vec![object(2)]);
        set_children(
            &mut state,
            node(0),
            vec![node(1)],
            vec![object(0), object_id],
        );

        let mut want = file;
        compound(&mut want).children.remove(0);
        let QbclNodeBody::Matrix(grid) = matrix_node().body else {
            panic!("the sample's matrix is a matrix");
        };
        root_children(&mut want)[0].body = QbclNodeBody::Compound(QbclCompound {
            matrix: grid,
            children: vec![added_node("added", [1, 2, 3]), leaf_node()],
        });
        assert_eq!(to_qbcl_file_with_ext(&state).unwrap(), want);
    }

    /// An ext out of step with the hierarchy errors instead of writing a
    /// guess.
    #[test]
    fn an_ext_out_of_step_with_its_nodes_errors() {
        let file = sample_file();

        let mut state = from_qbcl_file_with_ext(&file).unwrap();
        let ext = state.ext_mut();
        ext.nodes.pop();
        assert!(to_qbcl_file_with_ext(&state).is_err());

        let mut state = from_qbcl_file_with_ext(&file).unwrap();
        let ext = state.ext_mut();
        let Some(QbclExtNode {
            body: QbclExtNodeBody::Matrix { masks, .. },
            ..
        }) = &mut ext.nodes[0]
        else {
            panic!("node 0 is the matrix");
        };
        masks.pop();
        assert!(to_qbcl_file_with_ext(&state).is_err());
    }
}
