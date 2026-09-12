use crate::{Error, QbclExtNodeBody, QbclVoxMain, Result, face_mask};
use branded_id::U32Id;
use qbcl::qbcl::{
    QbclColor, QbclCompound, QbclFile, QbclMatrix, QbclMetadata, QbclModel, QbclNode, QbclNodeBody,
    QbclThumbnail, QbclVoxel,
};
use ty_math::TyVector3I32;
use voxcore::{
    BVoxHierarchyNode, VoxHierarchyNode, VoxObject, color::resolve_cell_color_or_transparent,
};

/// Writes a [`QbclVoxMain`] to a decoded Qubicle Construction Library
/// [`QbclFile`], the inverse of [`from_qbcl_file`](crate::from_qbcl_file).
/// The scene tree is walked from the single root. Each node is named for its
/// hierarchy node at its translation rounded to whole voxels. Each matrix or
/// compound object emits its grid with colors from the palette and, per
/// solid voxel, the mask of the faces its neighbors leave uncovered. The
/// entry supplies what the scene cannot derive. A state from
/// [`to_qbcl_vox_main`](crate::to_qbcl_vox_main) writes as a file
/// synthesized from the scene.
///
/// Errors if:
///
/// 1. the ext's node entries do not line up with the hierarchy
/// 2. the state does not have exactly one root
/// 3. a matrix or compound entry's node places no object and lists no child
///    node
/// 4. a model entry's node places an object under a transform chunk other
///    than the default
/// 5. an object's `baseColor` draws from a non-color value pool
pub fn to_qbcl_file(main: &QbclVoxMain) -> Result<QbclFile> {
    let ext = main.ext();

    let node_count = main.hierarchy_node_count();
    if node_count != ext.nodes.len() {
        return Err(Error::invalid(format!(
            "qbcl ext has {} nodes but the state has {node_count} hierarchy nodes",
            ext.nodes.len()
        )));
    }

    let root_ids = main.root_hierarchy_node_ids();
    let [root_id] = root_ids else {
        return Err(Error::invalid(format!(
            "a Qubicle .qbcl file needs exactly one root, but the state has {}",
            root_ids.len()
        )));
    };

    let root = rebuild_node(*root_id, main)?;

    Ok(QbclFile {
        program_version: ext.program_version,
        file_version: ext.file_version,
        thumbnail: QbclThumbnail {
            width: ext.thumbnail.width,
            height: ext.thumbnail.height,
            pixels: ext
                .thumbnail
                .pixels
                .iter()
                .map(|pixel| QbclColor::new(pixel[0], pixel[1], pixel[2], pixel[3]))
                .collect(),
        },
        metadata: QbclMetadata {
            title: ext.metadata.title.clone(),
            description: ext.metadata.description.clone(),
            tags: ext.metadata.tags.clone(),
            author: ext.metadata.author.clone(),
            company: ext.metadata.company.clone(),
            website: ext.metadata.website.clone(),
            copyright: ext.metadata.copyright.clone(),
        },
        guid: ext.guid,
        root,
    })
}

/// Rebuilds one scene node and its subtree from the hierarchy node `node_id`
/// and its entry. The node takes the shape a synthesized node would from its
/// objects and children. The entry supplies the provenance that fits the
/// shape.
fn rebuild_node(node_id: U32Id<BVoxHierarchyNode>, main: &QbclVoxMain) -> Result<QbclNode> {
    let hierarchy = main
        .hierarchy_node(node_id)
        .expect("a hierarchy id from the state resolves");
    let Some(entry) = main.ext().nodes.get(&node_id) else {
        return Err(Error::invalid(format!(
            "qbcl ext has no entry for node {}",
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

    let body = match &entry.body {
        QbclExtNodeBody::Model { transform } => {
            if objects.is_empty() {
                QbclNodeBody::Model(QbclModel {
                    transform: model_transform(transform)?,
                    children,
                })
            } else {
                // The node now carries a grid. A matrix or compound has no
                // place for the transform chunk. Dropping the default chunk
                // loses nothing.
                if *transform != QbclModel::DEFAULT_TRANSFORM {
                    return Err(Error::invalid(format!(
                        "node {} has a model entry with a transform chunk other than the default but places an object",
                        node_id.to_u32()
                    )));
                }
                synthesized_body(main, position, &objects, children)?
            }
        }
        QbclExtNodeBody::Matrix { pivot } => entry_body(
            main,
            node_id,
            placed_matrix(position, *pivot),
            &objects,
            children,
            false,
        )?,
        QbclExtNodeBody::Compound { pivot } => entry_body(
            main,
            node_id,
            placed_matrix(position, *pivot),
            &objects,
            children,
            true,
        )?,
    };

    Ok(QbclNode {
        name: hierarchy.name.clone(),
        visible: entry.visible,
        locked: entry.locked,
        body,
    })
}

/// Rebuilds the child nodes of a hierarchy node, in stored order.
fn rebuild_children(hierarchy: &VoxHierarchyNode, main: &QbclVoxMain) -> Result<Vec<QbclNode>> {
    hierarchy
        .child_node_ids
        .iter()
        .map(|&child_id| rebuild_node(child_id, main))
        .collect()
}

/// A matrix carrying an entry's placement and an empty grid.
fn placed_matrix(position: [i32; 3], pivot: [f32; 3]) -> QbclMatrix {
    QbclMatrix {
        position,
        pivot,
        ..QbclMatrix::default()
    }
}

/// The body a matrix or compound entry writes. The node's first object fills
/// `placement`. A node placing no object keeps the placement around an empty
/// grid for its children, and errors when it has no children either.
fn entry_body(
    main: &QbclVoxMain,
    node_id: U32Id<BVoxHierarchyNode>,
    placement: QbclMatrix,
    objects: &[&VoxObject],
    children: Vec<QbclNode>,
    compound: bool,
) -> Result<QbclNodeBody> {
    let [first, extras @ ..] = objects else {
        if children.is_empty() {
            return Err(Error::invalid(format!(
                "node {} has a grid entry but places no object and lists no child node",
                node_id.to_u32()
            )));
        }

        // A selection dropped the grid and kept the node for its children.
        return grid_body(main, placement, &[], children, compound);
    };
    let grid = fill_grid(main, first, placement)?;

    grid_body(main, grid, extras, children, compound)
}

/// Converts a stored model-transform chunk into its fixed 36-byte array.
fn model_transform(bytes: &[u8]) -> Result<[u8; 36]> {
    <[u8; 36]>::try_from(bytes).map_err(|_| {
        Error::invalid(format!(
            "qbcl model transform is {} bytes, expected 36",
            bytes.len()
        ))
    })
}

/// The body a node writes around `grid`. Qubicle has no node placing several
/// grids, so further objects become child matrices named for the object.
/// `compound` keeps a compound entry's shape when nothing else calls for one.
fn grid_body(
    main: &QbclVoxMain,
    grid: QbclMatrix,
    extras: &[&VoxObject],
    children: Vec<QbclNode>,
    compound: bool,
) -> Result<QbclNodeBody> {
    let mut all_children: Vec<QbclNode> = extras
        .iter()
        .map(|object| synthesized_matrix_node(main, object, grid.position))
        .collect::<Result<_>>()?;
    all_children.extend(children);

    let body = if all_children.is_empty() && !compound {
        QbclNodeBody::Matrix(grid)
    } else {
        QbclNodeBody::Compound(QbclCompound {
            matrix: grid,
            children: all_children,
        })
    };

    Ok(body)
}

/// Fills `matrix` with `object`'s grid in `.qbcl` storage order. Each solid
/// voxel's color comes from the object's `baseColor` layer and its mask from
/// the faces its neighbors leave uncovered.
fn fill_grid(main: &QbclVoxMain, object: &VoxObject, mut matrix: QbclMatrix) -> Result<QbclMatrix> {
    let [size_x, size_y, size_z] = object.bounds().to_array();
    let volume = size_x as usize * size_y as usize * size_z as usize;
    let mut voxels = vec![QbclVoxel::default(); volume];

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
        voxels[index] = QbclVoxel::new(r, g, b, mask);
    }

    matrix.size = [size_x, size_y, size_z];
    matrix.voxels = voxels;

    Ok(matrix)
}

/// The body a synthesized node takes from its objects and children.
fn synthesized_body(
    main: &QbclVoxMain,
    position: [i32; 3],
    objects: &[&VoxObject],
    children: Vec<QbclNode>,
) -> Result<QbclNodeBody> {
    let [first, extras @ ..] = objects else {
        return Ok(QbclNodeBody::Model(QbclModel {
            transform: QbclModel::DEFAULT_TRANSFORM,
            children,
        }));
    };
    let grid = synthesized_matrix(main, first, position)?;

    grid_body(main, grid, extras, children, false)
}

/// A matrix node named for `object` around its synthesized grid.
fn synthesized_matrix_node(
    main: &QbclVoxMain,
    object: &VoxObject,
    position: [i32; 3],
) -> Result<QbclNode> {
    Ok(QbclNode {
        name: object.name().to_owned(),
        body: QbclNodeBody::Matrix(synthesized_matrix(main, object, position)?),
        ..QbclNode::default()
    })
}

/// `object`'s grid as a matrix with no provenance.
fn synthesized_matrix(
    main: &QbclVoxMain,
    object: &VoxObject,
    position: [i32; 3],
) -> Result<QbclMatrix> {
    let matrix = QbclMatrix {
        position,
        ..QbclMatrix::default()
    };

    fill_grid(main, object, matrix)
}

/// The objects a hierarchy node places, in order.
fn placed_objects<'a>(hierarchy: &VoxHierarchyNode, main: &'a QbclVoxMain) -> Vec<&'a VoxObject> {
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
    use crate::{QbclExtNode, QbclExtNodeBody, QbclVoxMain, from_qbcl_file, to_qbcl_file};
    use branded_id::U32Id;
    use qbcl::qbcl::{
        QbclColor, QbclCompound, QbclFile, QbclMatrix, QbclMetadata, QbclModel, QbclNode,
        QbclNodeBody, QbclThumbnail, QbclVoxel,
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
                    QbclVoxel::new(10, 20, 30, LONE & !4),
                    QbclVoxel::new(1, 2, 3, LONE & !2),
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
                voxels: vec![QbclVoxel::new(1, 2, 3, LONE)],
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
                    voxels: vec![QbclVoxel::new(40, 50, 60, LONE)],
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
        main: &mut QbclVoxMain,
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
    fn retain_added_object(main: &mut QbclVoxMain) -> U32Id<BVoxObject> {
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
                voxels: vec![QbclVoxel::new(1, 2, 3, LONE)],
            }),
        }
    }

    #[test]
    fn round_trips_through_vox_main() {
        let file = sample_file();
        let main = from_qbcl_file(&file).unwrap();
        assert_eq!(to_qbcl_file(&main).unwrap(), file);
    }

    #[test]
    fn round_trips_the_default_file() {
        let file = QbclFile::default();
        let main = from_qbcl_file(&file).unwrap();
        assert_eq!(to_qbcl_file(&main).unwrap(), file);
    }

    /// A file whose root is a matrix rather than the conventional model.
    #[test]
    fn round_trips_a_matrix_root() {
        let file = QbclFile {
            root: matrix_node(),
            ..Default::default()
        };
        let main = from_qbcl_file(&file).unwrap();
        assert_eq!(to_qbcl_file(&main).unwrap(), file);
    }

    /// The mutations `vxl to` makes to keep `matrix`. The survivors' entries
    /// follow their ids through the holes the releases leave and the gc.
    #[test]
    fn a_released_node_leaves_the_survivors_provenance() {
        let file = sample_file();
        let mut main = from_qbcl_file(&file).unwrap();

        set_children(&mut main, node(4), vec![node(0)], Vec::new());
        set_children(&mut main, node(3), Vec::new(), Vec::new());
        for index in [1, 2, 3] {
            main.release_hierarchy_node(node(index)).unwrap();
        }
        for index in [1, 2] {
            main.release_object(object(index)).unwrap();
        }

        let mut want = file;
        root_children(&mut want).truncate(1);
        assert_eq!(to_qbcl_file(&main).unwrap(), want);

        main.gc().unwrap();

        let ext = main.ext();
        assert_eq!(ext.nodes.len(), 2);
        assert_eq!(
            ext.nodes[&node(0)],
            QbclExtNode {
                visible: true,
                locked: false,
                body: QbclExtNodeBody::Matrix {
                    pivot: [0.5, 0.0, 0.0],
                },
            }
        );
        assert_eq!(to_qbcl_file(&main).unwrap(), want);
    }

    /// The mutations `vxl to` makes to keep `inner`. The compound loses its
    /// grid and keeps its placement for its child.
    #[test]
    fn a_compound_whose_grid_is_released_keeps_its_placement() {
        let file = sample_file();
        let mut main = from_qbcl_file(&file).unwrap();

        set_children(&mut main, node(4), vec![node(3)], Vec::new());
        set_children(&mut main, node(3), vec![node(2)], Vec::new());
        for index in [0, 1] {
            main.release_hierarchy_node(node(index)).unwrap();
        }
        for index in [0, 2] {
            main.release_object(object(index)).unwrap();
        }
        main.gc().unwrap();

        let mut want = file;
        let compound = compound(&mut want);
        compound.matrix.size = [0, 0, 0];
        compound.matrix.voxels = Vec::new();
        compound.children = vec![inner_node()];
        root_children(&mut want).remove(0);
        assert_eq!(to_qbcl_file(&main).unwrap(), want);
    }

    /// A grid entry whose node keeps neither an object nor a child has
    /// nothing to write, so the write errors.
    #[test]
    fn a_grid_entry_whose_node_empties_errors() {
        let file = sample_file();
        let mut main = from_qbcl_file(&file).unwrap();

        set_children(&mut main, node(0), Vec::new(), Vec::new());

        assert!(to_qbcl_file(&main).is_err());
    }

    /// A node retained after the load takes the entry synthesis would give
    /// it, and writes from the scene.
    #[test]
    fn a_node_retained_after_the_load_takes_a_synthesized_entry() {
        let file = sample_file();
        let mut main = from_qbcl_file(&file).unwrap();

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
            node(4),
            vec![node(0), node(3), node_id],
            Vec::new(),
        );

        assert_eq!(
            main.ext().nodes[&node_id],
            QbclExtNode {
                visible: true,
                locked: false,
                body: QbclExtNodeBody::Matrix {
                    pivot: [0.0, 0.0, 0.0],
                },
            }
        );

        let mut want = file;
        root_children(&mut want).push(added_node("placed", [3, -3, 16]));
        assert_eq!(to_qbcl_file(&main).unwrap(), want);
    }

    /// A model entry's node takes the shape a synthesized node would. The
    /// entry's flags carry over and the name follows the hierarchy.
    #[test]
    fn a_model_entry_takes_the_shape_of_its_node() {
        let file = sample_file();
        let mut main = from_qbcl_file(&file).unwrap();

        let object_id = retain_added_object(&mut main);
        set_children(&mut main, node(1), Vec::new(), vec![object_id]);

        let mut want = file;
        compound(&mut want).children[0] = added_node("leaf", [0, 0, 0]);
        assert_eq!(to_qbcl_file(&main).unwrap(), want);
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
        let mut main = from_qbcl_file(&file).unwrap();
        assert_eq!(to_qbcl_file(&main).unwrap(), file);

        let object_id = retain_added_object(&mut main);
        set_children(&mut main, node(1), Vec::new(), vec![object_id]);
        assert!(to_qbcl_file(&main).is_err());
    }

    /// A matrix entry's node that gains children or further objects writes a
    /// compound around its grid.
    #[test]
    fn a_matrix_entry_whose_node_gains_children_writes_a_compound() {
        let file = sample_file();
        let mut main = from_qbcl_file(&file).unwrap();

        let object_id = retain_added_object(&mut main);
        set_children(&mut main, node(3), vec![node(2)], vec![object(2)]);
        set_children(
            &mut main,
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
        assert_eq!(to_qbcl_file(&main).unwrap(), want);
    }

    /// An ext out of step with the hierarchy errors instead of writing a
    /// guess, and a hook on a node it does not know refuses the mutation.
    #[test]
    fn an_ext_out_of_step_with_its_nodes_errors() {
        let file = sample_file();

        let mut main = from_qbcl_file(&file).unwrap();
        main.ext_mut().nodes.remove(&node(1));
        assert!(to_qbcl_file(&main).is_err());

        set_children(&mut main, node(3), vec![node(2)], vec![object(2)]);
        assert_eq!(
            main.release_hierarchy_node(node(1)),
            Err(VoxError::Ext {
                reason: "qbcl ext has no entry for node 1".to_owned(),
            })
        );
        assert!(main.hierarchy_node(node(1)).is_some());
    }
}
