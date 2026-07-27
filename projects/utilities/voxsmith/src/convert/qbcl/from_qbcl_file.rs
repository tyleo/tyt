use crate::{
    BASE_COLOR_FACTOR, Error, QubicleQbclExt, QubicleQbclExtWrapper, QubicleQbclMetadata,
    QubicleQbclNode, QubicleQbclNodeBody, QubicleQbclThumbnail, Result, to_vox_value,
};
use branded_id::U32Id;
use qbcl::qbcl::{QbclFile, QbclMatrix, QbclMetadata, QbclNode, QbclNodeBody};
use std::collections::{HashMap, HashSet};
use ty_math::{TySrgbU8, TyTransformF64, TyVector3I32, TyVector3U32};
use voxcore::{
    BVoxHierarchyNode, BVoxMaterial, BVoxPalette, VoxHierarchyNode, VoxMain, VoxObject, VoxPalette,
    VoxValuePool,
};

/// Loads a decoded Qubicle Construction Library [`QbclFile`] into a
/// [`VoxMain`].
///
/// Matrix and compound grids become objects sharing one `baseColorFactor`
/// palette, and the scene tree becomes the hierarchy nodes. The state with no
/// native voxcore home, such as the per-voxel visibility masks, node names and
/// editor flags, the model transform chunks, matrix placements and pivots, the
/// thumbnail, the metadata strings, the guid, and the versions, rides in a
/// `qubicle-qbcl` ext so the file can be written back exactly.
///
/// Errors on a matrix grid that exceeds the dense limit, or if
/// [`VoxMain::validate`](voxcore::VoxMain::validate) rejects the result.
pub fn from_qbcl_file(file: &QbclFile) -> Result<VoxMain> {
    let mut state = VoxMain::default();

    let (palette, materials) = build_palette(&mut state, &file.root);
    let palette_id = state.add_palette(palette);

    let mut nodes = Vec::new();
    let root_id = build_node(&file.root, &mut state, palette_id, &materials, &mut nodes)?;
    state.set_root_hierarchy_nodes(vec![root_id]);

    let ext = QubicleQbclExtWrapper {
        qubicle_qbcl: QubicleQbclExt {
            program_version: file.program_version,
            file_version: file.file_version,
            thumbnail: QubicleQbclThumbnail {
                width: file.thumbnail.width,
                height: file.thumbnail.height,
                pixels: file
                    .thumbnail
                    .pixels
                    .iter()
                    .map(|pixel| [pixel.r, pixel.g, pixel.b, pixel.a])
                    .collect(),
            },
            metadata: metadata_provenance(&file.metadata),
            guid: file.guid,
            nodes,
        },
    };
    state.set_ext(Some(to_vox_value(&ext)?));

    state.validate()?;
    Ok(state)
}

/// Builds the hierarchy node for one scene node and its subtree, adding it and
/// any objects to the state and appending its provenance to `nodes` so the ext
/// entry at each index lines up with the hierarchy node id. Returns the new
/// node's id.
fn build_node(
    node: &QbclNode,
    state: &mut VoxMain,
    palette: U32Id<BVoxPalette>,
    materials: &HashMap<[u8; 3], U32Id<BVoxMaterial>>,
    nodes: &mut Vec<QubicleQbclNode>,
) -> Result<U32Id<BVoxHierarchyNode>> {
    let id = match &node.body {
        QbclNodeBody::Matrix(matrix) => {
            // The matrix grid becomes the object's build volume directly; it
            // may carry empty margin. The masks are read from that same grid.
            let object = build_object(matrix, palette, materials)?;
            let masks = masks_of(&object, matrix);
            let object_id = state.add_object(object);
            let hierarchy = VoxHierarchyNode {
                name: node.name.clone(),
                child_nodes: Vec::new(),
                child_objects: vec![object_id],
                transform: translation(matrix.position),
            };
            let id = state.add_hierarchy_node(hierarchy);
            nodes.push(node_provenance(
                node,
                QubicleQbclNodeBody::Matrix {
                    position: matrix.position,
                    pivot: matrix.pivot,
                    masks,
                },
            ));
            id
        }
        QbclNodeBody::Model(model) => {
            let mut child_nodes = Vec::with_capacity(model.children.len());
            for child in &model.children {
                child_nodes.push(build_node(child, state, palette, materials, nodes)?);
            }
            let hierarchy = VoxHierarchyNode {
                name: node.name.clone(),
                child_nodes,
                child_objects: Vec::new(),
                transform: TyTransformF64::default(),
            };
            let id = state.add_hierarchy_node(hierarchy);
            nodes.push(node_provenance(
                node,
                QubicleQbclNodeBody::Model {
                    transform: model.transform.to_vec(),
                },
            ));
            id
        }
        QbclNodeBody::Compound(compound) => {
            let mut child_nodes = Vec::with_capacity(compound.children.len());
            for child in &compound.children {
                child_nodes.push(build_node(child, state, palette, materials, nodes)?);
            }
            // The compound grid becomes the object's build volume directly; it
            // may carry empty margin. The masks are read from that same grid.
            let object = build_object(&compound.matrix, palette, materials)?;
            let masks = masks_of(&object, &compound.matrix);
            let object_id = state.add_object(object);
            let hierarchy = VoxHierarchyNode {
                name: node.name.clone(),
                child_nodes,
                child_objects: vec![object_id],
                transform: translation(compound.matrix.position),
            };
            let id = state.add_hierarchy_node(hierarchy);
            nodes.push(node_provenance(
                node,
                QubicleQbclNodeBody::Compound {
                    position: compound.matrix.position,
                    pivot: compound.matrix.pivot,
                    masks,
                },
            ));
            id
        }
    };
    Ok(id)
}

/// Builds the one shared palette: a color pool of one entry per distinct color
/// across every matrix and compound voxel in the tree, bound to
/// `baseColorFactor`, with one material per color and a map from a color to its
/// material. The pool is added to `state`. A tree with no solid voxels gets a
/// single placeholder color so objects have a default material to sample.
fn build_palette(
    state: &mut VoxMain,
    root: &QbclNode,
) -> (VoxPalette, HashMap<[u8; 3], U32Id<BVoxMaterial>>) {
    let mut order: Vec<[u8; 3]> = Vec::new();
    let mut seen: HashSet<[u8; 3]> = HashSet::new();
    collect_colors(root, &mut order, &mut seen);
    if order.is_empty() {
        order.push([0, 0, 0]);
    }

    // A Qubicle voxel carries no alpha, so colors ride in a shared sRGB pool as
    // float components in `[0, 1]`; each material draws one value id into it.
    let pool = state.add_value_pool(
        VoxValuePool::srgb(order.iter().map(|&color| color_floats(color)).collect())
            .expect("byte-derived components are in range and the list is non-empty"),
    );

    let mut palette = VoxPalette::default();
    palette
        .add_property(BASE_COLOR_FACTOR.to_owned(), pool)
        .expect("the property names are distinct");
    let mut materials = HashMap::with_capacity(order.len());
    for (index, color) in order.iter().enumerate() {
        let material = palette
            .add_material(vec![U32Id::from_u32(index as u32)])
            .expect("one value id for the one property");
        materials.insert(*color, material);
    }

    (palette, materials)
}

/// Collects the distinct colors of a node and its subtree, in first-seen order.
fn collect_colors(node: &QbclNode, order: &mut Vec<[u8; 3]>, seen: &mut HashSet<[u8; 3]>) {
    match &node.body {
        QbclNodeBody::Matrix(matrix) => collect_matrix(matrix, order, seen),
        QbclNodeBody::Model(model) => {
            for child in &model.children {
                collect_colors(child, order, seen);
            }
        }
        QbclNodeBody::Compound(compound) => {
            collect_matrix(&compound.matrix, order, seen);
            for child in &compound.children {
                collect_colors(child, order, seen);
            }
        }
    }
}

/// Collects the distinct colors of one matrix's solid voxels.
fn collect_matrix(matrix: &QbclMatrix, order: &mut Vec<[u8; 3]>, seen: &mut HashSet<[u8; 3]>) {
    for voxel in &matrix.voxels {
        if voxel.is_empty() {
            continue;
        }
        let color = [voxel.r, voxel.g, voxel.b];
        if seen.insert(color) {
            order.push(color);
        }
    }
}

/// Builds an object from a matrix: a dense grid sized by the matrix,
/// referencing the shared palette on one layer, each solid voxel sampling its
/// color material. Errors on an oversized grid.
fn build_object(
    matrix: &QbclMatrix,
    palette: U32Id<BVoxPalette>,
    materials: &HashMap<[u8; 3], U32Id<BVoxMaterial>>,
) -> Result<VoxObject> {
    let [size_x, size_y, size_z] = matrix.size;
    let mut object = VoxObject::new(String::new(), TyVector3U32::new(size_x, size_y, size_z))
        .ok_or_else(|| {
            Error::invalid(format!(
                "matrix grid {size_x}x{size_y}x{size_z} exceeds the dense limit of {} cells",
                VoxObject::MAX_GRID_CELLS
            ))
        })?;

    object.add_layer(palette, U32Id::<BVoxMaterial>::from_u32(0));

    for x in 0..size_x {
        for y in 0..size_y {
            for z in 0..size_z {
                let Some(voxel) = matrix.voxel(x, y, z) else {
                    continue;
                };
                if voxel.is_empty() {
                    continue;
                }
                let material = materials
                    .get(&[voxel.r, voxel.g, voxel.b])
                    .copied()
                    .expect("every solid color is in the palette");
                let id = object
                    .voxel_id(TyVector3U32::new(x, y, z))
                    .expect("a coordinate inside the matrix is inside the grid");
                object
                    .retain_voxel(id, &[material])
                    .expect("one sample for the one layer");
            }
        }
    }

    Ok(object)
}

/// The visibility masks of an object's solid voxels, in live-voxel raster
/// order, read back from the matrix the object was built from.
fn masks_of(object: &VoxObject, matrix: &QbclMatrix) -> Vec<u8> {
    object
        .iter_live()
        .map(|id| {
            let position = object
                .voxel_position(id)
                .expect("a live voxel is within the grid");
            matrix
                .voxel(position.x, position.y, position.z)
                .map_or(0, |voxel| voxel.mask)
        })
        .collect()
}

/// The ext provenance for one node: its name, editor flags, and per-kind body.
fn node_provenance(node: &QbclNode, body: QubicleQbclNodeBody) -> QubicleQbclNode {
    QubicleQbclNode {
        name: node.name.clone(),
        visible: node.visible,
        locked: node.locked,
        body,
    }
}

/// The ext provenance for the metadata strings.
fn metadata_provenance(metadata: &QbclMetadata) -> QubicleQbclMetadata {
    QubicleQbclMetadata {
        title: metadata.title.clone(),
        description: metadata.description.clone(),
        tags: metadata.tags.clone(),
        author: metadata.author.clone(),
        company: metadata.company.clone(),
        website: metadata.website.clone(),
        copyright: metadata.copyright.clone(),
    }
}

/// A translation-only transform from a scene position.
fn translation(position: [i32; 3]) -> TyTransformF64 {
    TyTransformF64::from_translation(TyVector3I32::from_array(position).as_dvec3())
}

/// The float sRGB components in `[0, 1]` of an `[r, g, b]` byte color.
fn color_floats(color: [u8; 3]) -> [f64; 3] {
    TySrgbU8::from(color).into_format::<f64>().into()
}

#[cfg(test)]
mod tests {
    use crate::{BASE_COLOR_FACTOR, from_qbcl_bytes, from_qbcl_file, to_qbcl_bytes, to_qbcl_file};
    use branded_id::U32Id;
    use qbcl::qbcl::{
        QbclColor, QbclCompound, QbclFile, QbclMatrix, QbclMetadata, QbclModel, QbclNode,
        QbclNodeBody, QbclThumbnail, QbclVoxel,
    };
    use std::collections::BTreeSet;
    use ty_math::{TyHexColor, TySrgbaU8, TyTransformF64, TyVector3F64, TyVector3U32};
    use voxcore::{
        BVoxHierarchyNode, BVoxMaterial, BVoxObject, VoxHierarchyNode, VoxMain, VoxMap, VoxObject,
        VoxPalette, VoxValue, VoxValuePool,
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

    /// A compound node carrying one baked voxel and one empty model child.
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
                children: vec![QbclNode {
                    name: "leaf".to_owned(),
                    visible: true,
                    locked: false,
                    body: QbclNodeBody::Model(QbclModel::default()),
                }],
            }),
        }
    }

    /// A file exercising a model root grouping a matrix and a compound, with a
    /// thumbnail, metadata, and a guid.
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

    /// The float sRGB components in `[0, 1]` of a `#RRGGBB` hex string.
    fn srgb(hex: &str) -> [f64; 3] {
        TySrgbaU8::from_hex(hex)
            .expect("a valid hex color")
            .into_format::<f64, f64>()
            .color
            .into()
    }

    /// A state carrying no format ext, built straight from voxcore: a red-green
    /// object and a blue object sharing one `baseColorFactor` palette, placed by
    /// a hierarchy of a nested group and two roots. This is the cross-format
    /// synthesis input.
    fn source_state() -> VoxMain {
        let mut state = VoxMain::default();

        // One baseColorFactor palette: red, green, blue.
        let pool = state.add_value_pool(
            VoxValuePool::srgb(
                ["#FF0000", "#00FF00", "#0000FF"]
                    .iter()
                    .map(|hex| srgb(hex))
                    .collect(),
            )
            .unwrap(),
        );
        let mut palette = VoxPalette::default();
        palette
            .add_property(BASE_COLOR_FACTOR.to_owned(), pool)
            .unwrap();
        for index in 0..3 {
            palette
                .add_material(vec![U32Id::from_u32(index)])
                .expect("one value id for the one property");
        }
        let palette_id = state.add_palette(palette);
        let material = |index: u32| U32Id::<BVoxMaterial>::from_u32(index);

        // Object 0: a red then a green voxel along x.
        let mut wide = VoxObject::new(String::new(), TyVector3U32::new(2, 1, 1))
            .expect("a 2x1x1 grid is within the dense limit");
        wide.add_layer(palette_id, material(0));
        for (x, color) in [(0u32, 0u32), (1, 1)] {
            let voxel = wide
                .voxel_id(TyVector3U32::new(x, 0, 0))
                .expect("a position within the grid");
            wide.retain_voxel(voxel, &[material(color)])
                .expect("one sample for the one layer");
        }
        state.add_object(wide);

        // Object 1: a single blue voxel.
        let mut unit = VoxObject::new(String::new(), TyVector3U32::new(1, 1, 1))
            .expect("a 1x1x1 grid is within the dense limit");
        unit.add_layer(palette_id, material(0));
        let voxel = unit
            .voxel_id(TyVector3U32::new(0, 0, 0))
            .expect("a position within the grid");
        unit.retain_voxel(voxel, &[material(2)])
            .expect("one sample for the one layer");
        state.add_object(unit);

        let object = |index: u32| U32Id::<BVoxObject>::from_u32(index);
        let node = |index: u32| U32Id::<BVoxHierarchyNode>::from_u32(index);
        let placed_at =
            |x: f64, y: f64, z: f64| TyTransformF64::from_translation(TyVector3F64::new(x, y, z));

        // node 0 groups node 1, which places object 0 at +5x; node 2 places
        // object 1 at +3y. Nodes 0 and 2 are the roots.
        state.add_hierarchy_node(VoxHierarchyNode {
            name: "group".to_owned(),
            child_nodes: vec![node(1)],
            child_objects: Vec::new(),
            transform: TyTransformF64::default(),
        });
        state.add_hierarchy_node(VoxHierarchyNode {
            name: "wide".to_owned(),
            child_nodes: Vec::new(),
            child_objects: vec![object(0)],
            transform: placed_at(5.0, 0.0, 0.0),
        });
        state.add_hierarchy_node(VoxHierarchyNode {
            name: "unit".to_owned(),
            child_nodes: Vec::new(),
            child_objects: vec![object(1)],
            transform: placed_at(0.0, 3.0, 0.0),
        });
        state.set_root_hierarchy_nodes(vec![node(0), node(2)]);

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
            let layer = index / size_y;
            let z = layer % size_z;
            let x = layer / size_z;
            set.insert((
                matrix.position[0] + x as i32,
                matrix.position[1] + y as i32,
                matrix.position[2] + z as i32,
                (voxel.r, voxel.g, voxel.b),
            ));
        }
    }

    #[test]
    fn round_trips_through_vox_state() {
        let file = sample_file();
        let state = from_qbcl_file(&file).unwrap();
        assert_eq!(to_qbcl_file(&state).unwrap(), file);
    }

    #[test]
    fn round_trips_through_qbcl_bytes() {
        let file = sample_file();
        let state = from_qbcl_file(&file).unwrap();
        let bytes = to_qbcl_bytes(&state).unwrap();
        let reloaded = from_qbcl_bytes(&bytes).unwrap();
        assert_eq!(to_qbcl_file(&reloaded).unwrap(), file);
    }

    #[test]
    fn round_trips_the_default_file() {
        let file = QbclFile::default();
        let state = from_qbcl_file(&file).unwrap();
        assert_eq!(to_qbcl_file(&state).unwrap(), file);
    }

    /// A file whose root is a matrix rather than the conventional model.
    #[test]
    fn round_trips_a_matrix_root() {
        let file = QbclFile {
            root: matrix_node(),
            ..Default::default()
        };
        let state = from_qbcl_file(&file).unwrap();
        assert_eq!(to_qbcl_file(&state).unwrap(), file);
    }

    /// A default state has no objects and no ext, so the writer synthesizes an
    /// empty file rooted at a childless model rather than erroring on the
    /// missing ext.
    #[test]
    fn synthesizes_an_empty_state_without_an_ext() {
        let state = VoxMain::default();
        let file = to_qbcl_file(&state).unwrap();
        let QbclNodeBody::Model(model) = &file.root.body else {
            panic!("synthesis roots under a model");
        };
        assert!(model.children.is_empty());
    }

    /// A state with no `qubicle-qbcl` ext, such as one cross-loaded from
    /// another format, synthesizes a file: the hierarchy maps to a Qubicle
    /// scene tree whose world voxels and colors match the source, and the file
    /// reads back into a valid state with both objects.
    #[test]
    fn synthesizes_a_file_without_an_ext() {
        let state = source_state();
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

    /// A foreign ext, here `voxel-max`, is ignored by the Qubicle writer: it
    /// synthesizes from the scene rather than failing to parse the wrong ext.
    #[test]
    fn synthesizes_past_a_foreign_ext() {
        let mut state = source_state();
        state.set_ext(Some(VoxValue::Object(VoxMap(vec![(
            "voxel-max".to_owned(),
            VoxValue::Null,
        )]))));
        let file = to_qbcl_file(&state).unwrap();
        assert_eq!(world_voxels(&file).len(), 3);
    }

    #[test]
    fn rejects_an_oversized_matrix() {
        let file = QbclFile {
            root: QbclNode {
                body: QbclNodeBody::Matrix(QbclMatrix {
                    size: [2048, 2048, 2048],
                    position: [0, 0, 0],
                    pivot: [0.0, 0.0, 0.0],
                    voxels: Vec::new(),
                }),
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(from_qbcl_file(&file).is_err());
    }
}
