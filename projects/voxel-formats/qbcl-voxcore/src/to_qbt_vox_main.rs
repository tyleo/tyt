use crate::{QbtVoxMain, Result, fold_under_root, qbt_ext_from_file, synthesized_qbt_ext_node};
use qbcl::qbt::QbtFile;
use voxcore::VoxMain;

/// Gives a bare state a synthesized [`QbtExt`](crate::QbtExt), the state
/// [`to_qbt_file`](crate::to_qbt_file) writes as a file synthesized from the
/// scene. The hierarchy first takes the shape of a Qubicle scene tree: one
/// synthetic root model over every root, a node cloned per extra path to it,
/// each node at its world position, and an object no node places under a
/// node of its own at the origin. Each node then takes the entry a retained
/// node takes. The header takes the defaults.
///
/// Lossy where Qubicle cannot represent the source. A model carries no name
/// or translation, so a group's name drops and its placement folds into its
/// descendants. Node rotation and scale drop. A color's alpha drops because a
/// Qubicle voxel stores none.
pub fn to_qbt_vox_main(mut main: VoxMain<()>) -> Result<QbtVoxMain> {
    fold_under_root(&mut main, "")?;

    let nodes = main
        .iter_hierarchy_nodes()
        .map(|(node_id, node)| (node_id, synthesized_qbt_ext_node(node)))
        .collect();

    Ok(main.put_ext(qbt_ext_from_file(&QbtFile::default(), nodes)))
}

#[cfg(test)]
mod tests {
    use crate::{QbtExtNode, from_qbt_file, to_qbt_file, to_qbt_vox_main};
    use branded_id::U32Id;
    use qbcl::qbt::{QbtCompound, QbtFile, QbtMatrix, QbtModel, QbtNode, QbtVoxel};
    use std::collections::BTreeSet;
    use ty_math::{TyHexColor, TySrgbaU8, TyTransformF64, TyVector3F64, TyVector3U32};
    use voxcore::{
        BVoxHierarchyNode, BVoxMaterial, BVoxObject, VoxHierarchyNode, VoxMain, VoxObject,
        VoxPalette, VoxValuePool, color::lin_srgba_f64_from_srgba_u8, material::BASE_COLOR,
    };

    /// The linear-light components of a `#RRGGBB` hex string.
    fn linear_rgb(hex: &str) -> [f64; 3] {
        let linear =
            lin_srgba_f64_from_srgba_u8(TySrgbaU8::from_hex(hex).expect("a valid hex color"));
        [linear.red, linear.green, linear.blue]
    }

    /// A bare state built straight from voxcore: a red-green object and a
    /// blue object sharing one `baseColor` palette, placed by a hierarchy of
    /// a nested group and two roots. This is the cross-format synthesis input.
    fn source_state() -> VoxMain<()> {
        let mut main = VoxMain::default();

        // One baseColor palette: red, green, blue.
        let value_pool_id = main.retain_value_pool(
            VoxValuePool::vec_3_float(
                ["#FF0000", "#00FF00", "#0000FF"]
                    .iter()
                    .map(|hex| linear_rgb(hex))
                    .collect(),
            )
            .unwrap(),
        );
        let mut palette = VoxPalette::default();
        palette
            .retain_property(BASE_COLOR.to_owned(), value_pool_id, U32Id::from_u32(0))
            .unwrap();
        for index in 0..3 {
            palette
                .retain_material(vec![U32Id::from_u32(index)])
                .expect("one value id for the one property");
        }
        let palette_id = main.retain_palette(palette).unwrap();
        let material_id = |index: u32| U32Id::<BVoxMaterial>::from_u32(index);

        // Object 0: a red then a green voxel along x.
        let mut wide = VoxObject::new(String::new(), TyVector3U32::new(2, 1, 1))
            .expect("a 2x1x1 grid is within the dense limit");
        wide.retain_layer(palette_id, material_id(0));
        for (x, material_index) in [(0u32, 0u32), (1, 1)] {
            let voxel_id = wide
                .voxel_id(TyVector3U32::new(x, 0, 0))
                .expect("a position within the grid");
            wide.retain_voxel(voxel_id, &[material_id(material_index)])
                .expect("one sample for the one layer");
        }
        main.retain_object(wide).unwrap();

        // Object 1: a single blue voxel.
        let mut unit = VoxObject::new(String::new(), TyVector3U32::new(1, 1, 1))
            .expect("a 1x1x1 grid is within the dense limit");
        unit.retain_layer(palette_id, material_id(0));
        let voxel_id = unit
            .voxel_id(TyVector3U32::new(0, 0, 0))
            .expect("a position within the grid");
        unit.retain_voxel(voxel_id, &[material_id(2)])
            .expect("one sample for the one layer");
        main.retain_object(unit).unwrap();

        let object_id = |index: u32| U32Id::<BVoxObject>::from_u32(index);
        let node_id = |index: u32| U32Id::<BVoxHierarchyNode>::from_u32(index);
        let placed_at =
            |x: f64, y: f64, z: f64| TyTransformF64::from_translation(TyVector3F64::new(x, y, z));

        // node 0 groups node 1, which places object 0 at +5x; node 2 places
        // object 1 at +3y. Nodes 0 and 2 are the roots.
        main.retain_hierarchy_nodes(vec![
            VoxHierarchyNode {
                name: "group".to_owned(),
                child_node_ids: vec![node_id(1)],
                child_object_ids: Vec::new(),
                transform: TyTransformF64::default(),
            },
            VoxHierarchyNode {
                name: "wide".to_owned(),
                child_node_ids: Vec::new(),
                child_object_ids: vec![object_id(0)],
                transform: placed_at(5.0, 0.0, 0.0),
            },
            VoxHierarchyNode {
                name: "unit".to_owned(),
                child_node_ids: Vec::new(),
                child_object_ids: vec![object_id(1)],
                transform: placed_at(0.0, 3.0, 0.0),
            },
        ])
        .unwrap();
        main.set_root_hierarchy_node_ids(vec![node_id(0), node_id(2)])
            .unwrap();

        main.validate().expect("a well-formed source state");
        main
    }

    /// A solid voxel in world space: `x`, `y`, `z`, and an `rgb` color.
    type WorldVoxel = (i32, i32, i32, (u8, u8, u8));

    /// The world voxels a file places: each matrix or compound grid's solid
    /// cells decoded to world coordinates and color, order-independent.
    /// Synthesis bakes the world position onto each matrix and leaves models at
    /// identity, so a cell's world coordinate is its grid coordinate plus the
    /// matrix position.
    fn world_voxels(file: &QbtFile) -> BTreeSet<WorldVoxel> {
        let mut set = BTreeSet::new();
        collect_world_voxels(&file.root, &mut set);
        set
    }

    /// Adds a node's solid world voxels to `set`, recursing into child nodes.
    fn collect_world_voxels(node: &QbtNode, set: &mut BTreeSet<WorldVoxel>) {
        match node {
            QbtNode::Matrix(matrix) => collect_matrix_voxels(matrix, set),
            QbtNode::Model(model) => {
                for child in &model.children {
                    collect_world_voxels(child, set);
                }
            }
            QbtNode::Compound(compound) => {
                collect_matrix_voxels(&compound.matrix, set);
                for child in &compound.children {
                    collect_world_voxels(child, set);
                }
            }
            QbtNode::Unknown(_) => {}
        }
    }

    /// Adds one matrix's solid world voxels to `set`, decoding the storage
    /// index `y + size_y * (z + size_z * x)` back to a grid coordinate.
    fn collect_matrix_voxels(matrix: &QbtMatrix, set: &mut BTreeSet<WorldVoxel>) {
        let [_, size_y, size_z] = matrix.size;
        for (index, voxel) in matrix.voxels.iter().enumerate() {
            if voxel.is_empty() {
                continue;
            }
            let index = index as u32;
            let y = index % size_y;
            let zx_index = index / size_y;
            let z = zx_index % size_z;
            let x = zx_index / size_z;
            set.insert((
                matrix.position[0] + x as i32,
                matrix.position[1] + y as i32,
                matrix.position[2] + z as i32,
                (voxel.r, voxel.g, voxel.b),
            ));
        }
    }

    /// A default state has no objects, so the writer synthesizes an empty
    /// file rooted at a childless model.
    #[test]
    fn synthesizes_an_empty_state_without_an_ext() {
        let main = to_qbt_vox_main(VoxMain::default()).unwrap();
        let file = to_qbt_file(&main).unwrap();
        let QbtNode::Model(model) = &file.root else {
            panic!("synthesis roots under a model");
        };
        assert!(model.children.is_empty());
    }

    /// A bare state, such as one cross-loaded from another format, synthesizes
    /// a file: the hierarchy maps to a Qubicle scene tree whose world voxels
    /// and colors match the source, and the file reads back into a valid state
    /// with both objects.
    #[test]
    fn synthesizes_a_file_without_an_ext() {
        let main = to_qbt_vox_main(source_state()).unwrap();
        let file = to_qbt_file(&main).unwrap();

        let red = (0xFF, 0, 0);
        let green = (0, 0xFF, 0);
        let blue = (0, 0, 0xFF);

        // Object 0 is placed at +5x under a group, object 1 at +3y.
        assert_eq!(
            world_voxels(&file),
            BTreeSet::from([(5, 0, 0, red), (6, 0, 0, green), (0, 3, 0, blue)])
        );

        let reloaded = from_qbt_file(&file).unwrap();
        assert_eq!(reloaded.object_count(), 2);
    }

    /// The scene folds under one root model at world positions, and every
    /// node takes an entry: a model for the root and the group, a matrix for
    /// each object node.
    #[test]
    fn synthesizes_an_entry_per_node() {
        let main = to_qbt_vox_main(source_state()).unwrap();

        let ext = main.ext();

        assert_eq!(ext.version, (1, 0));

        assert_eq!(main.hierarchy_node_count(), 4);

        assert_eq!(ext.nodes.len(), 4);

        let [root_id] = main.root_hierarchy_node_ids() else {
            panic!("one root");
        };

        let root = main.hierarchy_node(*root_id).unwrap();

        assert_eq!(root.child_node_ids.len(), 2);

        let node_id = |index: u32| U32Id::<BVoxHierarchyNode>::from_u32(index);

        let matrix = QbtExtNode::Matrix {
            local_scale: [1, 1, 1],
            pivot: [0.0, 0.0, 0.0],
        };

        assert_eq!(ext.nodes[&node_id(0)], QbtExtNode::Model);

        assert_eq!(ext.nodes[&node_id(1)], matrix);

        assert_eq!(
            main.hierarchy_node(node_id(1)).unwrap().transform,
            TyTransformF64::from_translation(TyVector3F64::new(5.0, 0.0, 0.0))
        );

        assert_eq!(ext.nodes[&node_id(2)], matrix);

        assert_eq!(ext.nodes[&node_id(3)], QbtExtNode::Model);
    }

    /// The written tree pins the synthesized shape: the group becomes a
    /// nameless model, and each object node a matrix at its world position.
    #[test]
    fn writes_the_synthesized_tree() {
        let main = to_qbt_vox_main(source_state()).unwrap();

        let file = to_qbt_file(&main).unwrap();

        let matrix = |name: &str, position: [i32; 3], voxels: Vec<QbtVoxel>| {
            QbtNode::Matrix(QbtMatrix {
                name: name.to_owned(),
                position,
                local_scale: [1, 1, 1],
                pivot: [0.0, 0.0, 0.0],
                size: [voxels.len() as u32, 1, 1],
                voxels,
            })
        };

        assert_eq!(
            file.root,
            QbtNode::Model(QbtModel {
                children: vec![
                    QbtNode::Model(QbtModel {
                        children: vec![matrix(
                            "wide",
                            [5, 0, 0],
                            vec![
                                QbtVoxel::new(0xFF, 0, 0, 0x7e & !4),
                                QbtVoxel::new(0, 0xFF, 0, 0x7e & !2),
                            ],
                        )],
                    }),
                    matrix("unit", [0, 3, 0], vec![QbtVoxel::new(0, 0, 0xFF, 0x7e)]),
                ],
            })
        );
    }

    /// A node placing an object and a child takes a compound entry, and a
    /// subtree placed twice is cloned per path.
    #[test]
    fn synthesizes_a_compound_and_clones_a_shared_subtree() {
        let mut main = source_state();

        // The group also places object 1, and node 1 hangs under both roots.
        let group_id = U32Id::<BVoxHierarchyNode>::from_u32(0);

        let mut group = main.hierarchy_node(group_id).unwrap().clone();

        group.child_object_ids.push(U32Id::from_u32(1));

        main.set_hierarchy_node(group_id, group).unwrap();

        let unit_id = U32Id::<BVoxHierarchyNode>::from_u32(2);

        let mut unit = main.hierarchy_node(unit_id).unwrap().clone();

        unit.child_node_ids.push(U32Id::from_u32(1));

        main.set_hierarchy_node(unit_id, unit).unwrap();

        let main = to_qbt_vox_main(main).unwrap();

        // The group, wide, unit, the clone of wide, and the root.
        assert_eq!(main.hierarchy_node_count(), 5);

        let ext = main.ext();

        assert!(matches!(ext.nodes[&group_id], QbtExtNode::Compound { .. }));

        let clone_id = U32Id::<BVoxHierarchyNode>::from_u32(3);

        assert!(matches!(ext.nodes[&clone_id], QbtExtNode::Matrix { .. }));

        let clone = main.hierarchy_node(clone_id).unwrap();

        assert_eq!(clone.name, "wide");

        assert_eq!(
            clone.transform,
            TyTransformF64::from_translation(TyVector3F64::new(5.0, 3.0, 0.0))
        );

        let file = to_qbt_file(&main).unwrap();

        let QbtNode::Model(root) = &file.root else {
            panic!("a model root");
        };

        let QbtNode::Compound(QbtCompound { matrix, .. }) = &root.children[0] else {
            panic!("a compound for the group");
        };

        assert_eq!(matrix.name, "group");
    }
}
