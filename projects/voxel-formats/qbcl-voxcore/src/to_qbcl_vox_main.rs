use crate::{
    QbclExtNode, QbclExtNodeBody, QbclVoxMain, Result, SOLID_MASK, fold_under_root,
    qbcl_ext_from_file,
};
use qbcl::qbcl::{QbclFile, QbclMatrix, QbclModel, QbclNode};
use voxcore::{VoxHierarchyNode, VoxMain};

/// Gives a bare state a synthesized [`QbclExt`](crate::QbclExt), the state
/// [`to_qbcl_file`](crate::to_qbcl_file) writes as a file synthesized from
/// the scene. The hierarchy first takes the shape of a Qubicle scene tree:
/// one synthetic root model named `root` over every root, a node cloned per
/// extra path to it, each node at its world position, and an object no node
/// places under a node of its own at the origin. Each node then takes an
/// entry named for it with the editor defaults. A node placing an object
/// takes a matrix body, or a compound body when it also lists children or
/// further objects, at its position, pivoted at the grid origin, with the
/// solid mask per voxel. Any other node takes a model body with the default
/// transform chunk. The header takes the defaults.
///
/// Lossy where Qubicle cannot represent the source. A model's transform
/// chunk carries no translation, so a group's placement folds into its
/// descendants. Node rotation and scale drop. A color's alpha drops because
/// a Qubicle voxel stores none.
pub fn to_qbcl_vox_main(mut state: VoxMain<()>) -> Result<QbclVoxMain> {
    fold_under_root(&mut state, "root")?;

    let nodes = state
        .iter_hierarchy_nodes()
        .map(|(_, node)| Some(synthesized_entry(&state, node)))
        .collect();

    Ok(state.put_ext(qbcl_ext_from_file(&QbclFile::default(), nodes)))
}

/// The entry a folded node takes.
fn synthesized_entry(state: &VoxMain<()>, node: &VoxHierarchyNode) -> QbclExtNode {
    let defaults = QbclNode::default();
    let body = match node.child_object_ids.as_slice() {
        [] => QbclExtNodeBody::Model {
            transform: QbclModel::DEFAULT_TRANSFORM.to_vec(),
        },
        [first, extras @ ..] => {
            let object = state
                .object(*first)
                .expect("a placed object is one of the state's");
            let position = node.transform.position.round().as_ivec3().to_array();
            let pivot = QbclMatrix::default().pivot;
            let masks = vec![SOLID_MASK; object.live_count()];

            if extras.is_empty() && node.child_node_ids.is_empty() {
                QbclExtNodeBody::Matrix {
                    position,
                    pivot,
                    masks,
                }
            } else {
                QbclExtNodeBody::Compound {
                    position,
                    pivot,
                    masks,
                }
            }
        }
    };

    QbclExtNode {
        name: node.name.clone(),
        visible: defaults.visible,
        locked: defaults.locked,
        body,
    }
}

#[cfg(test)]
mod tests {
    use crate::{QbclExtNodeBody, SOLID_MASK, from_qbcl_file, to_qbcl_file, to_qbcl_vox_main};
    use branded_id::U32Id;
    use qbcl::qbcl::{QbclFile, QbclMatrix, QbclModel, QbclNode, QbclNodeBody, QbclVoxel};
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
        let mut state = VoxMain::default();

        // One baseColor palette: red, green, blue.
        let value_pool_id = state.retain_value_pool(
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
        let palette_id = state.retain_palette(palette).unwrap();
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
        state.retain_object(wide).unwrap();

        // Object 1: a single blue voxel.
        let mut unit = VoxObject::new(String::new(), TyVector3U32::new(1, 1, 1))
            .expect("a 1x1x1 grid is within the dense limit");
        unit.retain_layer(palette_id, material_id(0));
        let voxel_id = unit
            .voxel_id(TyVector3U32::new(0, 0, 0))
            .expect("a position within the grid");
        unit.retain_voxel(voxel_id, &[material_id(2)])
            .expect("one sample for the one layer");
        state.retain_object(unit).unwrap();

        let object_id = |index: u32| U32Id::<BVoxObject>::from_u32(index);
        let node_id = |index: u32| U32Id::<BVoxHierarchyNode>::from_u32(index);
        let placed_at =
            |x: f64, y: f64, z: f64| TyTransformF64::from_translation(TyVector3F64::new(x, y, z));

        // node 0 groups node 1, which places object 0 at +5x; node 2 places
        // object 1 at +3y. Nodes 0 and 2 are the roots.
        state
            .retain_hierarchy_nodes(vec![
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
        state
            .set_root_hierarchy_node_ids(vec![node_id(0), node_id(2)])
            .unwrap();

        state.validate().expect("a well-formed source state");
        state
    }

    /// A solid voxel in world space: `x`, `y`, `z`, and an `rgb` color.
    type WorldVoxel = (i32, i32, i32, (u8, u8, u8));

    /// The world voxels a file places: each matrix or compound grid's solid
    /// cells decoded to world coordinates and color, order-independent.
    /// Synthesis bakes the world position onto each matrix and leaves models at
    /// identity, so a cell's world coordinate is its grid coordinate plus the
    /// matrix position.
    fn world_voxels(file: &QbclFile) -> BTreeSet<WorldVoxel> {
        let mut set = BTreeSet::new();
        collect_world_voxels(&file.root, &mut set);
        set
    }

    /// Adds a node's solid world voxels to `set`, recursing into child nodes.
    fn collect_world_voxels(node: &QbclNode, set: &mut BTreeSet<WorldVoxel>) {
        match &node.body {
            QbclNodeBody::Matrix(matrix) => collect_matrix_voxels(matrix, set),
            QbclNodeBody::Model(model) => {
                for child in &model.children {
                    collect_world_voxels(child, set);
                }
            }
            QbclNodeBody::Compound(compound) => {
                collect_matrix_voxels(&compound.matrix, set);
                for child in &compound.children {
                    collect_world_voxels(child, set);
                }
            }
        }
    }

    /// Adds one matrix's solid world voxels to `set`, decoding the storage
    /// index `y + size_y * (z + size_z * x)` back to a grid coordinate.
    fn collect_matrix_voxels(matrix: &QbclMatrix, set: &mut BTreeSet<WorldVoxel>) {
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
        let state = to_qbcl_vox_main(VoxMain::default()).unwrap();
        let file = to_qbcl_file(&state).unwrap();
        let QbclNodeBody::Model(model) = &file.root.body else {
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
        let state = to_qbcl_vox_main(source_state()).unwrap();
        let file = to_qbcl_file(&state).unwrap();

        let red = (0xFF, 0, 0);
        let green = (0, 0xFF, 0);
        let blue = (0, 0, 0xFF);

        // Object 0 is placed at +5x under a group, object 1 at +3y.
        assert_eq!(
            world_voxels(&file),
            BTreeSet::from([(5, 0, 0, red), (6, 0, 0, green), (0, 3, 0, blue)])
        );

        let reloaded = from_qbcl_file(&file).unwrap();
        assert_eq!(reloaded.object_count(), 2);
    }

    /// The scene folds under one root model at world positions, and every
    /// node takes an entry named for it: a model body for the root and the
    /// group, a matrix body for each object node.
    #[test]
    fn synthesizes_an_entry_per_node() {
        let state = to_qbcl_vox_main(source_state()).unwrap();

        let ext = state.ext();

        assert_eq!(ext.file_version, 2);

        assert_eq!(state.hierarchy_node_count(), 4);

        assert_eq!(ext.nodes.len(), 4);

        let group = ext.nodes[0].as_ref().unwrap();

        assert_eq!(group.name, "group");

        assert!(group.visible);

        assert_eq!(
            group.body,
            QbclExtNodeBody::Model {
                transform: QbclModel::DEFAULT_TRANSFORM.to_vec(),
            }
        );

        let wide = ext.nodes[1].as_ref().unwrap();

        assert_eq!(wide.name, "wide");

        assert_eq!(
            wide.body,
            QbclExtNodeBody::Matrix {
                position: [5, 0, 0],
                pivot: [0.0, 0.0, 0.0],
                masks: vec![SOLID_MASK; 2],
            }
        );

        let root = ext.nodes[3].as_ref().unwrap();

        assert_eq!(root.name, "root");
    }

    /// The written tree pins the synthesized shape: the group keeps its name
    /// as a model, and each object node is a matrix at its world position.
    #[test]
    fn writes_the_synthesized_tree() {
        let state = to_qbcl_vox_main(source_state()).unwrap();

        let file = to_qbcl_file(&state).unwrap();

        let matrix = |name: &str, position: [i32; 3], voxels: Vec<QbclVoxel>| QbclNode {
            name: name.to_owned(),
            body: QbclNodeBody::Matrix(QbclMatrix {
                size: [voxels.len() as u32, 1, 1],
                position,
                pivot: [0.0, 0.0, 0.0],
                voxels,
            }),
            ..QbclNode::default()
        };

        assert_eq!(
            file.root,
            QbclNode {
                name: "root".to_owned(),
                body: QbclNodeBody::Model(QbclModel {
                    transform: QbclModel::DEFAULT_TRANSFORM,
                    children: vec![
                        QbclNode {
                            name: "group".to_owned(),
                            body: QbclNodeBody::Model(QbclModel {
                                transform: QbclModel::DEFAULT_TRANSFORM,
                                children: vec![matrix(
                                    "wide",
                                    [5, 0, 0],
                                    vec![
                                        QbclVoxel::new(0xFF, 0, 0, SOLID_MASK),
                                        QbclVoxel::new(0, 0xFF, 0, SOLID_MASK),
                                    ],
                                )],
                            }),
                            ..QbclNode::default()
                        },
                        matrix(
                            "unit",
                            [0, 3, 0],
                            vec![QbclVoxel::new(0, 0, 0xFF, SOLID_MASK)]
                        ),
                    ],
                }),
                ..QbclNode::default()
            }
        );
    }

    /// An object no node places gets a node of its own at the origin, after
    /// the roots.
    #[test]
    fn places_an_unplaced_object_under_the_root() {
        let mut state = source_state();

        let unit_id = U32Id::<BVoxHierarchyNode>::from_u32(2);

        let mut unit = state.hierarchy_node(unit_id).unwrap().clone();

        unit.child_object_ids.clear();

        state.set_hierarchy_node(unit_id, unit).unwrap();

        let state = to_qbcl_vox_main(state).unwrap();

        // The group, wide, the emptied unit, the object's node, and the root.
        assert_eq!(state.hierarchy_node_count(), 5);

        let [root_id] = state.root_hierarchy_node_ids() else {
            panic!("one root");
        };

        let root = state.hierarchy_node(*root_id).unwrap();

        assert_eq!(root.child_node_ids.len(), 3);

        let placed = state.hierarchy_node(root.child_node_ids[2]).unwrap();

        assert_eq!(
            placed.child_object_ids,
            vec![U32Id::<BVoxObject>::from_u32(1)]
        );

        assert_eq!(
            state.ext().nodes[3].as_ref().unwrap().body,
            QbclExtNodeBody::Matrix {
                position: [0, 0, 0],
                pivot: [0.0, 0.0, 0.0],
                masks: vec![SOLID_MASK],
            }
        );
    }
}
