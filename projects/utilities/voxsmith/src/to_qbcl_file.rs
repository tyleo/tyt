use crate::{
    Error, QubicleQbclExtWrapper, QubicleQbclNode, QubicleQbclNodeBody, Result, from_vox_value,
};
use branded_id::U32Id;
use qbcl::qbcl::{
    QbclColor, QbclCompound, QbclFile, QbclMatrix, QbclMetadata, QbclModel, QbclNode, QbclNodeBody,
    QbclThumbnail, QbclVoxel,
};
use voxcore::{
    BVoxAttribute, BVoxHierarchyNode, BVoxPaletteRef, BVoxVoxel, VoxHierarchyNode, VoxObject,
    VoxPalette, VoxState, VoxValue,
};

/// Writes a [`VoxState`] back to a decoded Qubicle Construction Library
/// [`QbclFile`], the inverse of [`from_qbcl_file`](crate::from_qbcl_file).
///
/// Requires the `qubicle-qbcl` ext the forward path writes; without it the file
/// cannot be rebuilt. The scene tree is walked from the single root, each matrix
/// or compound object emitting its grid with the visibility masks and color from
/// the ext and the palette.
///
/// Errors if the ext is missing, its node entries do not line up with the
/// hierarchy, the state does not have exactly one root, or a mask list does not
/// match its object.
pub fn to_qbcl_file(state: &VoxState) -> Result<QbclFile> {
    let ext = match state.ext() {
        Some(ext) => from_vox_value::<QubicleQbclExtWrapper>(ext)?.qubicle_qbcl,
        None => {
            return Err(Error::invalid(
                "state has no qubicle-qbcl ext; cannot rebuild a Qubicle .qbcl file",
            ));
        }
    };

    let node_count = state.hierarchy_node_count();
    if node_count != ext.nodes.len() {
        return Err(Error::invalid(format!(
            "qubicle-qbcl ext has {} nodes but the state has {node_count} hierarchy nodes",
            ext.nodes.len()
        )));
    }

    let roots = state.root_hierarchy_nodes();
    let [root] = roots else {
        return Err(Error::invalid(format!(
            "a Qubicle .qbcl file needs exactly one root, but the state has {}",
            roots.len()
        )));
    };

    let palette = state.iter_palettes().next().map(|(_, palette)| palette);
    let root = rebuild_node(*root, state, &ext.nodes, palette)?;

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
            title: ext.metadata.title,
            description: ext.metadata.description,
            tags: ext.metadata.tags,
            author: ext.metadata.author,
            company: ext.metadata.company,
            website: ext.metadata.website,
            copyright: ext.metadata.copyright,
        },
        guid: ext.guid,
        root,
    })
}

/// Rebuilds one scene node and its subtree from the hierarchy node `id` and its
/// aligned ext provenance.
fn rebuild_node(
    id: U32Id<BVoxHierarchyNode>,
    state: &VoxState,
    nodes: &[QubicleQbclNode],
    palette: Option<&VoxPalette>,
) -> Result<QbclNode> {
    let hierarchy = state
        .hierarchy_node(id)
        .ok_or_else(|| Error::invalid(format!("hierarchy node {} does not exist", id.to_u32())))?;
    let provenance = nodes.get(id.to_u32() as usize).ok_or_else(|| {
        Error::invalid(format!(
            "qubicle-qbcl ext has no entry for hierarchy node {}",
            id.to_u32()
        ))
    })?;

    let body = match &provenance.body {
        QubicleQbclNodeBody::Model { transform } => QbclNodeBody::Model(QbclModel {
            transform: model_transform(transform)?,
            children: rebuild_children(hierarchy, state, nodes, palette)?,
        }),
        QubicleQbclNodeBody::Matrix {
            position,
            pivot,
            masks,
        } => QbclNodeBody::Matrix(matrix_from_object(
            matrix_object(hierarchy, state)?,
            palette,
            *position,
            *pivot,
            masks,
        )?),
        QubicleQbclNodeBody::Compound {
            position,
            pivot,
            masks,
        } => {
            let matrix = matrix_from_object(
                matrix_object(hierarchy, state)?,
                palette,
                *position,
                *pivot,
                masks,
            )?;
            QbclNodeBody::Compound(QbclCompound {
                matrix,
                children: rebuild_children(hierarchy, state, nodes, palette)?,
            })
        }
    };

    Ok(QbclNode {
        name: provenance.name.clone(),
        visible: provenance.visible,
        locked: provenance.locked,
        body,
    })
}

/// Rebuilds the child nodes of a hierarchy node, in stored order.
fn rebuild_children(
    hierarchy: &VoxHierarchyNode,
    state: &VoxState,
    nodes: &[QubicleQbclNode],
    palette: Option<&VoxPalette>,
) -> Result<Vec<QbclNode>> {
    hierarchy
        .child_nodes
        .iter()
        .map(|&child| rebuild_node(child, state, nodes, palette))
        .collect()
}

/// The object a matrix or compound node places, or an error if it has none.
fn matrix_object<'a>(hierarchy: &VoxHierarchyNode, state: &'a VoxState) -> Result<&'a VoxObject> {
    let object_id = *hierarchy
        .child_objects
        .first()
        .ok_or_else(|| Error::invalid("a matrix or compound node has no object"))?;
    state
        .object(object_id)
        .ok_or_else(|| Error::invalid(format!("object {} does not exist", object_id.to_u32())))
}

/// Rebuilds a matrix grid from an object: each solid voxel's color comes from the
/// palette and its mask from the aligned ext mask list, placed in `.qbcl` storage
/// order. Errors if the mask count does not match the object's solid voxels.
fn matrix_from_object(
    object: &VoxObject,
    palette: Option<&VoxPalette>,
    position: [i32; 3],
    pivot: [f32; 3],
    masks: &[u8],
) -> Result<QbclMatrix> {
    let bounds = object.bounds();
    let [size_x, size_y, size_z] = [bounds.x, bounds.y, bounds.z];
    let volume = size_x as usize * size_y as usize * size_z as usize;
    let mut voxels = vec![QbclVoxel::default(); volume];

    let reference = object.iter_palette_refs().next().map(|(id, _)| id);
    let live: Vec<_> = object.iter_live().collect();
    if live.len() != masks.len() {
        return Err(Error::invalid(format!(
            "qubicle-qbcl ext has {} masks but the object has {} solid voxels",
            masks.len(),
            live.len()
        )));
    }

    for (&voxel, &mask) in live.iter().zip(masks) {
        let position = object
            .voxel_position(voxel)
            .expect("a live voxel is within the grid");
        let [r, g, b] = voxel_color(object, palette, reference, voxel);
        // Storage order: index = y + size_y * (z + size_z * x).
        let index = position.y as usize
            + size_y as usize * (position.z as usize + size_z as usize * position.x as usize);
        voxels[index] = QbclVoxel::new(r, g, b, mask);
    }

    Ok(QbclMatrix {
        size: [size_x, size_y, size_z],
        position,
        pivot,
        voxels,
    })
}

/// The `[r, g, b]` color a live voxel samples from the shared palette, or black
/// if the reference, cell, or `rgb` attribute is missing.
fn voxel_color(
    object: &VoxObject,
    palette: Option<&VoxPalette>,
    reference: Option<U32Id<BVoxPaletteRef>>,
    voxel: U32Id<BVoxVoxel>,
) -> [u8; 3] {
    let lookup = || -> Option<[u8; 3]> {
        let palette = palette?;
        let reference = reference?;
        let cell = object.voxel_cell(voxel, reference)?;
        let rgb = attribute_id(palette, "rgb")?;
        Some(parse_rgb(palette.cell_value(cell, rgb)))
    };
    lookup().unwrap_or([0, 0, 0])
}

/// Converts a stored model-transform chunk into its fixed 36-byte array.
fn model_transform(bytes: &[u8]) -> Result<[u8; 36]> {
    <[u8; 36]>::try_from(bytes).map_err(|_| {
        Error::invalid(format!(
            "qubicle-qbcl model transform is {} bytes, expected 36",
            bytes.len()
        ))
    })
}

/// The id of the attribute named `name`, or `None`.
fn attribute_id(palette: &VoxPalette, name: &str) -> Option<U32Id<BVoxAttribute>> {
    palette
        .iter_attributes()
        .find(|(_, attribute)| *attribute == name)
        .map(|(id, _)| id)
}

/// Parses a `#RRGGBB` color cell into `[r, g, b]`, defaulting to black on a
/// missing or malformed value.
fn parse_rgb(value: Option<&VoxValue>) -> [u8; 3] {
    let Some(VoxValue::Text(hex)) = value else {
        return [0, 0, 0];
    };
    let hex = hex.strip_prefix('#').unwrap_or(hex);
    let byte = |index: usize| {
        hex.get(index * 2..index * 2 + 2)
            .and_then(|byte| u8::from_str_radix(byte, 16).ok())
            .unwrap_or(0)
    };
    [byte(0), byte(1), byte(2)]
}
