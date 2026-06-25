use crate::{
    Error, QubicleQbclExt, QubicleQbclExtWrapper, QubicleQbclMetadata, QubicleQbclNode,
    QubicleQbclNodeBody, QubicleQbclThumbnail, Result, to_vox_value,
};
use branded_id::U32Id;
use qbcl::qbcl::{QbclFile, QbclMatrix, QbclMetadata, QbclNode, QbclNodeBody};
use std::collections::{HashMap, HashSet};
use ty_math::{TyQuaternionF64, TyTransformF64, TyVector3F64, TyVector3U32};
use voxcore::{
    BVoxHierarchyNode, BVoxPalette, BVoxPaletteCell, VoxHierarchyNode, VoxObject, VoxPalette,
    VoxState, VoxValue,
};

/// Loads a decoded Qubicle Construction Library [`QbclFile`] into a [`VoxState`].
///
/// Matrix and compound grids become objects sharing one `rgb` palette, and the
/// scene tree becomes the hierarchy nodes. The state with no native voxcore home,
/// such as the per-voxel visibility masks, node names and editor flags, the
/// model transform chunks, matrix placements and pivots, the thumbnail, the
/// metadata strings, the guid, and the versions, rides in a `qubicle-qbcl` ext so
/// the file can be written back exactly.
///
/// Errors on a matrix grid that exceeds the dense limit, or if
/// [`VoxState::validate`](voxcore::VoxState::validate) rejects the result.
pub fn from_qbcl_file(file: &QbclFile) -> Result<VoxState> {
    let mut state = VoxState::default();

    let (palette, cells) = build_palette(&file.root);
    let palette_id = state.add_palette(palette);

    let mut nodes = Vec::new();
    let root_id = build_node(&file.root, &mut state, palette_id, &cells, &mut nodes)?;
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

/// Builds the hierarchy node for one scene node and its subtree, adding it and any
/// objects to the state and appending its provenance to `nodes` so the ext entry
/// at each index lines up with the hierarchy node id. Returns the new node's id.
fn build_node(
    node: &QbclNode,
    state: &mut VoxState,
    palette: U32Id<BVoxPalette>,
    cells: &HashMap<[u8; 3], U32Id<BVoxPaletteCell>>,
    nodes: &mut Vec<QubicleQbclNode>,
) -> Result<U32Id<BVoxHierarchyNode>> {
    let id = match &node.body {
        QbclNodeBody::Matrix(matrix) => {
            let object = build_object(matrix, palette, cells)?;
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
                child_nodes.push(build_node(child, state, palette, cells, nodes)?);
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
                child_nodes.push(build_node(child, state, palette, cells, nodes)?);
            }
            let object = build_object(&compound.matrix, palette, cells)?;
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

/// Builds the one shared palette: an `rgb` cell per distinct color across every
/// matrix and compound voxel in the tree, plus a map from a color to its cell. A
/// tree with no solid voxels gets a single placeholder cell so objects have a
/// default sample to reference.
fn build_palette(root: &QbclNode) -> (VoxPalette, HashMap<[u8; 3], U32Id<BVoxPaletteCell>>) {
    let mut order: Vec<[u8; 3]> = Vec::new();
    let mut seen: HashSet<[u8; 3]> = HashSet::new();
    collect_colors(root, &mut order, &mut seen);
    if order.is_empty() {
        order.push([0, 0, 0]);
    }

    let mut palette = VoxPalette::default();
    palette.add_attribute("rgb".to_owned());
    let mut cells = HashMap::with_capacity(order.len());
    for color in &order {
        let cell = palette
            .add_cell(vec![VoxValue::Text(hex(*color))])
            .expect("one value per palette attribute");
        cells.insert(*color, cell);
    }

    (palette, cells)
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

/// Builds an object from a matrix: a dense grid sized by the matrix, referencing
/// the shared palette, each solid voxel sampling its color cell. Errors on an
/// oversized grid.
fn build_object(
    matrix: &QbclMatrix,
    palette: U32Id<BVoxPalette>,
    cells: &HashMap<[u8; 3], U32Id<BVoxPaletteCell>>,
) -> Result<VoxObject> {
    let [size_x, size_y, size_z] = matrix.size;
    let mut object = VoxObject::new(String::new(), TyVector3U32::new(size_x, size_y, size_z))
        .ok_or_else(|| {
            Error::invalid(format!(
                "matrix grid {size_x}x{size_y}x{size_z} exceeds the dense limit of {} cells",
                VoxObject::MAX_GRID_CELLS
            ))
        })?;

    object.add_palette_ref(palette, U32Id::<BVoxPaletteCell>::from_u32(0));

    for x in 0..size_x {
        for y in 0..size_y {
            for z in 0..size_z {
                let Some(voxel) = matrix.voxel(x, y, z) else {
                    continue;
                };
                if voxel.is_empty() {
                    continue;
                }
                let cell = cells
                    .get(&[voxel.r, voxel.g, voxel.b])
                    .copied()
                    .expect("every solid color is in the palette");
                let id = object
                    .voxel_id(TyVector3U32::new(x, y, z))
                    .expect("a coordinate inside the matrix is inside the grid");
                object
                    .retain_voxel(id, &[cell])
                    .expect("one sample for the one palette reference");
            }
        }
    }

    Ok(object)
}

/// The visibility masks of an object's solid voxels, in live-voxel raster order,
/// read back from the matrix the object was built from.
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
    TyTransformF64::new(
        TyVector3F64::new(position[0] as f64, position[1] as f64, position[2] as f64),
        TyQuaternionF64::identity(),
        TyVector3F64::new(1.0, 1.0, 1.0),
    )
}

/// One `#RRGGBB` string for an `[r, g, b]` color.
fn hex(color: [u8; 3]) -> String {
    format!("#{:02X}{:02X}{:02X}", color[0], color[1], color[2])
}

#[cfg(test)]
mod tests {
    use crate::{from_qbcl_bytes, from_qbcl_file, to_qbcl_bytes, to_qbcl_file};
    use qbcl::qbcl::{
        QbclColor, QbclCompound, QbclFile, QbclMatrix, QbclMetadata, QbclModel, QbclNode,
        QbclNodeBody, QbclThumbnail, QbclVoxel,
    };
    use voxcore::VoxState;

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

    #[test]
    fn errors_without_qubicle_qbcl_ext() {
        let state = VoxState::default();
        assert!(to_qbcl_file(&state).is_err());
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
