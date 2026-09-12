use crate::{Error, QbtExtNode, QbtVoxMain, Result, face_mask};
use branded_id::U32Id;
use qbcl::qbt::{
    QbtColor, QbtCompound, QbtFile, QbtMatrix, QbtModel, QbtNode, QbtUnknownNode, QbtVoxel,
};
use ty_math::TyVector3I32;
use voxcore::{
    BVoxHierarchyNode, VoxHierarchyNode, VoxObject, color::resolve_cell_color_or_transparent,
};

/// Writes a [`QbtVoxMain`] to a decoded Qubicle Binary Tree [`QbtFile`], the
/// inverse of [`from_qbt_file`](crate::from_qbt_file). The scene tree is
/// walked from the single root. Each node is named for its hierarchy node at
/// its translation rounded to whole voxels. Each matrix or compound object
/// emits its grid with colors from the palette and, per solid voxel, the
/// mask of the faces its neighbors leave uncovered. The entry supplies what
/// the scene cannot derive. A state from
/// [`to_qbt_vox_main`](crate::to_qbt_vox_main) writes as a file synthesized
/// from the scene.
///
/// Errors if:
///
/// 1. the ext's node entries do not line up with the hierarchy
/// 2. the state does not have exactly one root
/// 3. a matrix or compound entry's node places no object and lists no child
///    node
/// 4. an unknown entry's node places an object or lists a child node
/// 5. an object's `baseColor` draws from a non-color value pool
pub fn to_qbt_file(main: &QbtVoxMain) -> Result<QbtFile> {
    let ext = main.ext();

    let node_count = main.hierarchy_node_count();
    if node_count != ext.nodes.len() {
        return Err(Error::invalid(format!(
            "qbt ext has {} nodes but the state has {node_count} hierarchy nodes",
            ext.nodes.len()
        )));
    }

    let root_ids = main.root_hierarchy_node_ids();
    let [root_id] = root_ids else {
        return Err(Error::invalid(format!(
            "a Qubicle .qbt file needs exactly one root, but the state has {}",
            root_ids.len()
        )));
    };

    let root = rebuild_node(*root_id, main)?;

    Ok(QbtFile {
        version: ext.version,
        global_scale: ext.global_scale,
        color_map: ext
            .color_map
            .iter()
            .map(|color| QbtColor::new(color[0], color[1], color[2], color[3]))
            .collect(),
        root,
    })
}

/// Rebuilds one scene node and its subtree from the hierarchy node `node_id`
/// and its entry. The node takes the shape a synthesized node would from its
/// objects and children. The entry supplies the provenance that fits the
/// shape. A model entry holds nothing, so its node takes a synthesized node's
/// provenance. An unknown entry fits only an unknown node.
fn rebuild_node(node_id: U32Id<BVoxHierarchyNode>, main: &QbtVoxMain) -> Result<QbtNode> {
    let hierarchy = main
        .hierarchy_node(node_id)
        .expect("a hierarchy id from the state resolves");
    let Some(entry) = main.ext().nodes.get(&node_id) else {
        return Err(Error::invalid(format!(
            "qbt ext has no entry for node {}",
            node_id.to_u32()
        )));
    };

    // The reader made each position its node's translation, so the
    // translation goes back as the position. The synthesizer's summing has
    // no place here because the ancestors carry their positions in their
    // entries.
    let position = rounded_translation(hierarchy).to_array();
    let objects = placed_objects(hierarchy, main);
    let children = rebuild_children(hierarchy, main)?;

    let node = match entry {
        QbtExtNode::Model => {
            synthesized_node(main, hierarchy.name.clone(), position, &objects, children)?
        }
        QbtExtNode::Matrix { local_scale, pivot } => entry_node(
            main,
            node_id,
            placed_matrix(&hierarchy.name, position, *local_scale, *pivot),
            &objects,
            children,
            false,
        )?,
        QbtExtNode::Compound { local_scale, pivot } => entry_node(
            main,
            node_id,
            placed_matrix(&hierarchy.name, position, *local_scale, *pivot),
            &objects,
            children,
            true,
        )?,
        QbtExtNode::Unknown { type_id, data } => {
            // Opaque bytes have no place for a grid or a child.
            if !objects.is_empty() || !children.is_empty() {
                return Err(Error::invalid(format!(
                    "node {} has an unknown entry but places an object or lists a child node",
                    node_id.to_u32()
                )));
            }
            QbtNode::Unknown(QbtUnknownNode {
                type_id: *type_id,
                data: data.clone(),
            })
        }
    };

    Ok(node)
}

/// Rebuilds the child nodes of a hierarchy node, in stored order.
fn rebuild_children(hierarchy: &VoxHierarchyNode, main: &QbtVoxMain) -> Result<Vec<QbtNode>> {
    hierarchy
        .child_node_ids
        .iter()
        .map(|&child_id| rebuild_node(child_id, main))
        .collect()
}

/// A matrix carrying an entry's provenance and an empty grid.
fn placed_matrix(
    name: &str,
    position: [i32; 3],
    local_scale: [u32; 3],
    pivot: [f32; 3],
) -> QbtMatrix {
    QbtMatrix {
        name: name.to_owned(),
        position,
        local_scale,
        pivot,
        ..QbtMatrix::default()
    }
}

/// The node a matrix or compound entry writes. The node's first object fills
/// `placement`. A node placing no object keeps the placement around an empty
/// grid for its children, and errors when it has no children either.
fn entry_node(
    main: &QbtVoxMain,
    node_id: U32Id<BVoxHierarchyNode>,
    placement: QbtMatrix,
    objects: &[&VoxObject],
    children: Vec<QbtNode>,
    compound: bool,
) -> Result<QbtNode> {
    let [first, extras @ ..] = objects else {
        if children.is_empty() {
            return Err(Error::invalid(format!(
                "node {} has a grid entry but places no object and lists no child node",
                node_id.to_u32()
            )));
        }

        // A selection dropped the grid and kept the node for its children.
        return grid_node(main, placement, &[], children, compound);
    };
    let grid = fill_grid(main, first, placement)?;

    grid_node(main, grid, extras, children, compound)
}

/// The node written around `grid`. Qubicle has no node placing several grids,
/// so further objects become child matrices named for the object. `compound`
/// keeps a compound entry's shape when nothing else calls for one.
fn grid_node(
    main: &QbtVoxMain,
    grid: QbtMatrix,
    extras: &[&VoxObject],
    children: Vec<QbtNode>,
    compound: bool,
) -> Result<QbtNode> {
    let mut all_children: Vec<QbtNode> = extras
        .iter()
        .map(|object| synthesized_matrix_node(main, object, grid.position))
        .collect::<Result<_>>()?;
    all_children.extend(children);

    let node = if all_children.is_empty() && !compound {
        QbtNode::Matrix(grid)
    } else {
        QbtNode::Compound(QbtCompound {
            matrix: grid,
            children: all_children,
        })
    };

    Ok(node)
}

/// Fills `matrix` with `object`'s grid in `.qbt` storage order. Each solid
/// voxel's color comes from the object's `baseColor` layer and its mask from
/// the faces its neighbors leave uncovered.
fn fill_grid(main: &QbtVoxMain, object: &VoxObject, mut matrix: QbtMatrix) -> Result<QbtMatrix> {
    let [size_x, size_y, size_z] = object.bounds().to_array();
    let volume = size_x as usize * size_y as usize * size_z as usize;
    let mut voxels = vec![QbtVoxel::default(); volume];

    let cell_color = resolve_cell_color_or_transparent(main, object)?;
    for voxel_id in object.iter_live() {
        let position = object
            .voxel_position(voxel_id)
            .expect("a live voxel is within the grid");
        // A Qubicle voxel stores no alpha, so the sampled color's alpha is
        // dropped.
        let [r, g, b, _] = cell_color.color(voxel_id);
        let mask = face_mask(object, voxel_id);
        // Storage order: index = y + size_y * (z + size_z * x).
        let index = position.y as usize
            + size_y as usize * (position.z as usize + size_z as usize * position.x as usize);
        voxels[index] = QbtVoxel::new(r, g, b, mask);
    }

    matrix.size = [size_x, size_y, size_z];
    matrix.voxels = voxels;

    Ok(matrix)
}

/// The Qubicle node a hierarchy node synthesizes to.
fn synthesized_node(
    main: &QbtVoxMain,
    name: String,
    position: [i32; 3],
    objects: &[&VoxObject],
    children: Vec<QbtNode>,
) -> Result<QbtNode> {
    let [first, extras @ ..] = objects else {
        return Ok(QbtNode::Model(QbtModel { children }));
    };
    let grid = synthesized_matrix(main, first, name, position)?;

    grid_node(main, grid, extras, children, false)
}

/// A matrix node named for `object` around its synthesized grid.
fn synthesized_matrix_node(
    main: &QbtVoxMain,
    object: &VoxObject,
    position: [i32; 3],
) -> Result<QbtNode> {
    synthesized_matrix(main, object, object.name().to_owned(), position).map(QbtNode::Matrix)
}

/// `object`'s grid as a matrix with no provenance.
fn synthesized_matrix(
    main: &QbtVoxMain,
    object: &VoxObject,
    name: String,
    position: [i32; 3],
) -> Result<QbtMatrix> {
    let matrix = QbtMatrix {
        name,
        position,
        local_scale: [1, 1, 1],
        ..QbtMatrix::default()
    };

    fill_grid(main, object, matrix)
}

/// The objects a hierarchy node places, in order.
fn placed_objects<'a>(hierarchy: &VoxHierarchyNode, main: &'a QbtVoxMain) -> Vec<&'a VoxObject> {
    hierarchy
        .child_object_ids
        .iter()
        .map(|&object_id| {
            main.object(object_id)
                .expect("a placed object is one of the state's")
        })
        .collect()
}

/// A node's translation rounded to a Qubicle position's whole voxels.
fn rounded_translation(node: &VoxHierarchyNode) -> TyVector3I32 {
    node.transform.position.round().as_ivec3()
}

#[cfg(test)]
mod tests {
    use crate::{QbtExtNode, QbtVoxMain, from_qbt_file, to_qbt_file};
    use branded_id::U32Id;
    use qbcl::qbt::{
        QbtColor, QbtCompound, QbtFile, QbtMatrix, QbtModel, QbtNode, QbtUnknownNode, QbtVoxel,
    };
    use ty_math::{TyTransformF64, TyVector3F64, TyVector3U32};
    use voxcore::{
        BVoxHierarchyNode, BVoxMaterial, BVoxObject, BVoxPalette, Error as VoxError,
        VoxHierarchyNode, VoxObject,
    };

    /// The mask of a voxel nothing covers.
    const LONE: u8 = 0x7e;

    /// A matrix node with two solid voxels covering each other along `x` in a
    /// `[2, 1, 1]` grid.
    fn matrix_node() -> QbtNode {
        QbtNode::Matrix(QbtMatrix {
            name: "matrix".to_owned(),
            position: [1, 2, 3],
            local_scale: [1, 1, 1],
            pivot: [0.5, 0.0, 0.0],
            size: [2, 1, 1],
            voxels: vec![
                QbtVoxel::new(10, 20, 30, LONE & !4),
                QbtVoxel::new(1, 2, 3, LONE & !2),
            ],
        })
    }

    /// A one-voxel matrix node inside the compound.
    fn inner_node() -> QbtNode {
        QbtNode::Matrix(QbtMatrix {
            name: "inner".to_owned(),
            position: [0, 1, 0],
            local_scale: [1, 1, 1],
            pivot: [0.0, 0.0, 0.0],
            size: [1, 1, 1],
            voxels: vec![QbtVoxel::new(1, 2, 3, LONE)],
        })
    }

    /// A compound node carrying one baked voxel, an empty model child, and a
    /// matrix child.
    fn compound_node() -> QbtNode {
        QbtNode::Compound(QbtCompound {
            matrix: QbtMatrix {
                name: "compound".to_owned(),
                position: [-1, -2, -3],
                local_scale: [2, 2, 2],
                pivot: [0.0, 0.0, 0.0],
                size: [1, 1, 1],
                voxels: vec![QbtVoxel::new(40, 50, 60, LONE)],
            },
            children: vec![QbtNode::Model(QbtModel::default()), inner_node()],
        })
    }

    /// A file exercising a model root grouping a matrix, a compound, and an
    /// unknown node, with a color map and a global scale. The load numbers the
    /// nodes:
    ///
    /// 0. `matrix`
    /// 1. the empty model
    /// 2. `inner`
    /// 3. `compound`
    /// 4. the unknown node
    /// 5. the root
    ///
    /// The objects load as 0 `matrix`, 1 `inner`, and 2 `compound`.
    fn sample_file() -> QbtFile {
        QbtFile {
            version: (1, 0),
            global_scale: [1.0, 2.0, 0.5],
            color_map: vec![QbtColor::new(10, 20, 30, 255), QbtColor::new(1, 2, 3, 255)],
            root: QbtNode::Model(QbtModel {
                children: vec![
                    matrix_node(),
                    compound_node(),
                    QbtNode::Unknown(QbtUnknownNode {
                        type_id: 99,
                        data: vec![9, 8, 7],
                    }),
                ],
            }),
        }
    }

    /// The root model's children of `file`.
    fn root_children(file: &mut QbtFile) -> &mut Vec<QbtNode> {
        let QbtNode::Model(root) = &mut file.root else {
            panic!("the sample roots under a model");
        };
        &mut root.children
    }

    fn node(index: u32) -> U32Id<BVoxHierarchyNode> {
        U32Id::from_u32(index)
    }

    fn object(index: u32) -> U32Id<BVoxObject> {
        U32Id::from_u32(index)
    }

    /// Replaces `node_id`'s children. The name and transform stay.
    fn set_children(
        main: &mut QbtVoxMain,
        node_id: U32Id<BVoxHierarchyNode>,
        child_node_ids: Vec<U32Id<BVoxHierarchyNode>>,
        child_object_ids: Vec<U32Id<BVoxObject>>,
    ) {
        let mut node = main.hierarchy_node(node_id).unwrap().clone();
        node.child_node_ids = child_node_ids;
        node.child_object_ids = child_object_ids;
        main.set_hierarchy_node(node_id, node).unwrap();
    }

    /// Retains a one-voxel object of the sample's second color.
    fn retain_added_object(main: &mut QbtVoxMain) -> U32Id<BVoxObject> {
        let mut object = VoxObject::new("added".to_owned(), TyVector3U32::new(1, 1, 1)).unwrap();
        object.retain_layer(
            U32Id::<BVoxPalette>::from_u32(0),
            U32Id::<BVoxMaterial>::from_u32(0),
        );
        let voxel_id = object.voxel_id(TyVector3U32::new(0, 0, 0)).unwrap();
        object
            .retain_voxel(voxel_id, &[U32Id::from_u32(1)])
            .unwrap();

        main.retain_object(object).unwrap()
    }

    /// The synthesized matrix of `retain_added_object`'s object.
    fn added_matrix(name: &str, position: [i32; 3]) -> QbtNode {
        QbtNode::Matrix(QbtMatrix {
            name: name.to_owned(),
            position,
            local_scale: [1, 1, 1],
            pivot: [0.0, 0.0, 0.0],
            size: [1, 1, 1],
            voxels: vec![QbtVoxel::new(1, 2, 3, LONE)],
        })
    }

    #[test]
    fn round_trips_through_vox_main() {
        let file = sample_file();
        let main = from_qbt_file(&file).unwrap();
        assert_eq!(to_qbt_file(&main).unwrap(), file);
    }

    #[test]
    fn round_trips_the_default_file() {
        let file = QbtFile::default();
        let main = from_qbt_file(&file).unwrap();
        assert_eq!(to_qbt_file(&main).unwrap(), file);
    }

    /// A file whose root is a matrix rather than the conventional model.
    #[test]
    fn round_trips_a_matrix_root() {
        let file = QbtFile {
            root: matrix_node(),
            ..Default::default()
        };
        let main = from_qbt_file(&file).unwrap();
        assert_eq!(to_qbt_file(&main).unwrap(), file);
    }

    /// The mutations `vxl to` makes to keep `matrix`. The survivors' entries
    /// follow their ids through the holes the releases leave and the gc.
    #[test]
    fn a_released_node_leaves_the_survivors_provenance() {
        let file = sample_file();
        let mut main = from_qbt_file(&file).unwrap();

        set_children(&mut main, node(5), vec![node(0)], Vec::new());
        set_children(&mut main, node(3), Vec::new(), Vec::new());
        for index in [1, 2, 3, 4] {
            main.release_hierarchy_node(node(index)).unwrap();
        }
        for index in [1, 2] {
            main.release_object(object(index)).unwrap();
        }

        let mut want = file;
        root_children(&mut want).truncate(1);
        assert_eq!(to_qbt_file(&main).unwrap(), want);

        main.gc().unwrap();

        let ext = main.ext();
        assert_eq!(ext.nodes.len(), 2);
        assert_eq!(
            ext.nodes[&node(0)],
            QbtExtNode::Matrix {
                local_scale: [1, 1, 1],
                pivot: [0.5, 0.0, 0.0],
            }
        );
        assert_eq!(to_qbt_file(&main).unwrap(), want);
    }

    /// The mutations `vxl to` makes to keep `inner`. The compound loses its
    /// grid and keeps its placement for its child.
    #[test]
    fn a_compound_whose_grid_is_released_keeps_its_placement() {
        let file = sample_file();
        let mut main = from_qbt_file(&file).unwrap();

        set_children(&mut main, node(5), vec![node(3)], Vec::new());
        set_children(&mut main, node(3), vec![node(2)], Vec::new());
        for index in [0, 1, 4] {
            main.release_hierarchy_node(node(index)).unwrap();
        }
        for index in [0, 2] {
            main.release_object(object(index)).unwrap();
        }
        main.gc().unwrap();

        let mut want = file;
        let QbtNode::Compound(compound) = compound_node() else {
            panic!("the sample's compound is a compound");
        };
        *root_children(&mut want) = vec![QbtNode::Compound(QbtCompound {
            matrix: QbtMatrix {
                size: [0, 0, 0],
                voxels: Vec::new(),
                ..compound.matrix
            },
            children: vec![inner_node()],
        })];
        assert_eq!(to_qbt_file(&main).unwrap(), want);
    }

    /// A grid entry whose node keeps neither an object nor a child has
    /// nothing to write, so the write errors.
    #[test]
    fn a_grid_entry_whose_node_empties_errors() {
        let file = sample_file();
        let mut main = from_qbt_file(&file).unwrap();

        set_children(&mut main, node(0), Vec::new(), Vec::new());

        assert!(to_qbt_file(&main).is_err());
    }

    /// A node retained after the load takes the entry synthesis would give
    /// it, and writes from the scene.
    #[test]
    fn a_node_retained_after_the_load_takes_a_synthesized_entry() {
        let file = sample_file();
        let mut main = from_qbt_file(&file).unwrap();

        let object_id = retain_added_object(&mut main);
        let node_id = main
            .retain_hierarchy_node(VoxHierarchyNode {
                name: "placed".to_owned(),
                child_node_ids: Vec::new(),
                child_object_ids: vec![object_id],
                transform: TyTransformF64::from_translation(TyVector3F64::new(3.4, -2.6, 16.0)),
            })
            .unwrap();
        set_children(
            &mut main,
            node(5),
            vec![node(0), node(3), node(4), node_id],
            Vec::new(),
        );

        assert_eq!(
            main.ext().nodes[&node_id],
            QbtExtNode::Matrix {
                local_scale: [1, 1, 1],
                pivot: [0.0, 0.0, 0.0],
            }
        );

        let mut want = file;
        root_children(&mut want).push(added_matrix("placed", [3, -3, 16]));
        assert_eq!(to_qbt_file(&main).unwrap(), want);
    }

    /// A model entry holds nothing. Its node takes the shape a synthesized
    /// node would.
    #[test]
    fn a_model_entry_takes_the_shape_of_its_node() {
        let file = sample_file();
        let mut main = from_qbt_file(&file).unwrap();

        let object_id = retain_added_object(&mut main);
        set_children(&mut main, node(1), Vec::new(), vec![object_id]);

        let mut want = file;
        let QbtNode::Compound(compound) = &mut root_children(&mut want)[1] else {
            panic!("the sample's compound is a compound");
        };
        compound.children[0] = added_matrix("", [0, 0, 0]);
        assert_eq!(to_qbt_file(&main).unwrap(), want);
    }

    /// A matrix entry's node that gains children or further objects writes a
    /// compound around its grid.
    #[test]
    fn a_matrix_entry_whose_node_gains_children_writes_a_compound() {
        let file = sample_file();
        let mut main = from_qbt_file(&file).unwrap();

        let object_id = retain_added_object(&mut main);
        set_children(&mut main, node(3), vec![node(2)], vec![object(2)]);
        set_children(
            &mut main,
            node(0),
            vec![node(1)],
            vec![object(0), object_id],
        );

        let mut want = file;
        let QbtNode::Compound(compound) = &mut root_children(&mut want)[1] else {
            panic!("the sample's compound is a compound");
        };
        compound.children.remove(0);
        let QbtNode::Matrix(grid) = matrix_node() else {
            panic!("the sample's matrix is a matrix");
        };
        root_children(&mut want)[0] = QbtNode::Compound(QbtCompound {
            matrix: grid,
            children: vec![
                added_matrix("added", [1, 2, 3]),
                QbtNode::Model(QbtModel::default()),
            ],
        });
        assert_eq!(to_qbt_file(&main).unwrap(), want);
    }

    /// An unknown node's bytes are opaque. The writer refuses to reshape an
    /// unknown entry whose node gains an object or a child.
    #[test]
    fn an_unknown_entry_refuses_a_grid_or_a_child() {
        let file = sample_file();

        let mut main = from_qbt_file(&file).unwrap();
        let object_id = retain_added_object(&mut main);
        set_children(&mut main, node(4), Vec::new(), vec![object_id]);
        assert!(to_qbt_file(&main).is_err());

        let mut main = from_qbt_file(&file).unwrap();
        set_children(&mut main, node(3), vec![node(2)], vec![object(2)]);
        set_children(&mut main, node(4), vec![node(1)], Vec::new());
        assert!(to_qbt_file(&main).is_err());
    }

    /// An ext out of step with the hierarchy errors instead of writing a
    /// guess, and a hook on a node it does not know refuses the mutation.
    #[test]
    fn an_ext_out_of_step_with_its_nodes_errors() {
        let file = sample_file();

        let mut main = from_qbt_file(&file).unwrap();
        main.ext_mut().nodes.remove(&node(4));
        assert!(to_qbt_file(&main).is_err());

        set_children(&mut main, node(5), vec![node(0), node(3)], Vec::new());
        assert_eq!(
            main.release_hierarchy_node(node(4)),
            Err(VoxError::Ext {
                reason: "qbt ext has no entry for node 4".to_owned(),
            })
        );
        assert!(main.hierarchy_node(node(4)).is_some());
    }
}
