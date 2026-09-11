use crate::{Error, GoxlExtSink, Result};
use branded_id::U32Id;
use goxl::{GoxlBlock, GoxlFile};
use std::collections::{HashMap, HashSet};
use ty_math::{TySrgbaU8, TyTransformF64, TyVector3U32};
use voxcore::{
    BVoxMaterial, BVoxObject, BVoxPalette, VoxHierarchyNode, VoxMain, VoxObject, VoxPalette,
    VoxValuePool, color::lin_srgba_f64_from_srgba_u8, material::BASE_COLOR,
};

/// Loads a Goxel [`GoxlFile`] into a [`VoxMain`] whose ext records what `T`
/// keeps. The shared `BL16` voxel blocks become objects sharing one `baseColor`
/// palette, and the `LAYR` layers become the hierarchy nodes, each placing the
/// blocks it stamps. The rest of the Goxel state goes to the ext.
///
/// Errors on a layer placement that references a block outside the block list,
/// or on a cross-reference the checked insertions reject.
pub fn read_goxl<T: GoxlExtSink>(file: &GoxlFile) -> Result<VoxMain<T>> {
    let mut state = VoxMain::default();

    let (palette, material_ids) = build_palette(&mut state, file);
    let palette_id = state.retain_palette(palette)?;

    for block in &file.blocks {
        // A Goxel block is a fixed 16-cube; it becomes the object's build
        // volume directly, with its live voxels wherever they sit inside it.
        state.retain_object(build_object(block, palette_id, &material_ids))?;
    }

    // Every layer node is a root, in stored order.
    let nodes = build_layer_nodes(file, state.object_count())?;
    let root_ids = state.retain_hierarchy_nodes(nodes)?;
    state.set_root_hierarchy_node_ids(root_ids)?;

    Ok(T::record_file(state, file))
}

/// Builds the one shared palette: a color value pool of one entry per distinct
/// color across every block's solid voxels, bound to `baseColor`, with
/// one material per color and a map from a color to its material. The
/// value pool is added to `state`. A file with no solid voxels gets a single
/// placeholder color so objects have a default material to sample.
fn build_palette(
    state: &mut VoxMain<()>,
    file: &GoxlFile,
) -> (VoxPalette, HashMap<[u8; 4], U32Id<BVoxMaterial>>) {
    let mut order: Vec<[u8; 4]> = Vec::new();
    let mut seen: HashSet<[u8; 4]> = HashSet::new();
    for block in &file.blocks {
        for voxel in &block.voxels {
            if voxel.is_empty() {
                continue;
            }
            let color = [voxel.r, voxel.g, voxel.b, voxel.a];
            if seen.insert(color) {
                order.push(color);
            }
        }
    }
    if order.is_empty() {
        order.push([0, 0, 0, 0]);
    }

    // Colors decode to linear light and ride in a shared `vec-4-float` value
    // pool. Each material draws one value id into it.
    let value_pool_id = state.retain_value_pool(
        VoxValuePool::vec_4_float(
            order
                .iter()
                .map(|&color| <[f64; 4]>::from(lin_srgba_f64_from_srgba_u8(TySrgbaU8::from(color))))
                .collect(),
        )
        .expect("byte-derived components are finite and the list is non-empty"),
    );

    let mut palette = VoxPalette::default();
    palette
        .retain_property(BASE_COLOR.to_owned(), value_pool_id, U32Id::from_u32(0))
        .expect("the property names are distinct");
    let mut material_ids = HashMap::with_capacity(order.len());
    for (index, color) in order.iter().enumerate() {
        let material_id = palette
            .retain_material(vec![U32Id::from_u32(index as u32)])
            .expect("one value id for the one property");
        material_ids.insert(*color, material_id);
    }

    (palette, material_ids)
}

/// Builds an object from a block: a dense `16 x 16 x 16` grid referencing the
/// shared palette on one layer, each solid voxel sampling its color material.
fn build_object(
    block: &GoxlBlock,
    palette_id: U32Id<BVoxPalette>,
    material_ids: &HashMap<[u8; 4], U32Id<BVoxMaterial>>,
) -> VoxObject {
    let size = GoxlBlock::SIZE;
    let mut object = VoxObject::new(String::new(), TyVector3U32::new(size, size, size))
        .expect("a 16-cubed block fits the dense grid");

    // The single layer is the color; solid voxels overwrite material 0.
    object.retain_layer(palette_id, U32Id::<BVoxMaterial>::from_u32(0));

    for z in 0..size {
        for y in 0..size {
            for x in 0..size {
                let Some(voxel) = block.voxel(x, y, z) else {
                    continue;
                };
                if voxel.is_empty() {
                    continue;
                }
                let color = [voxel.r, voxel.g, voxel.b, voxel.a];
                let material_id = material_ids
                    .get(&color)
                    .copied()
                    .expect("every solid color is in the palette");
                let voxel_id = object
                    .voxel_id(TyVector3U32::new(x, y, z))
                    .expect("a coordinate inside the block is inside the grid");
                object
                    .retain_voxel(voxel_id, &[material_id])
                    .expect("one sample for the one layer");
            }
        }
    }

    object
}

/// Builds the hierarchy nodes, one per layer in stored order. A layer node
/// places the distinct blocks it stamps, deduplicated to satisfy voxcore's
/// per-node uniqueness rule; the exact placement list rides in the ext. Errors
/// on a placement that references a block outside the block list.
fn build_layer_nodes(file: &GoxlFile, object_count: usize) -> Result<Vec<VoxHierarchyNode>> {
    let mut nodes = Vec::with_capacity(file.layers.len());
    for layer in &file.layers {
        let mut child_object_ids = Vec::new();
        let mut seen = HashSet::new();
        for placement in &layer.blocks {
            let index = placement.block_index;
            if index < 0 || index as usize >= object_count {
                return Err(Error::invalid(format!(
                    "layer block placement references block {index}, which does not exist"
                )));
            }
            if seen.insert(index) {
                child_object_ids.push(U32Id::<BVoxObject>::from_u32(index as u32));
            }
        }
        nodes.push(VoxHierarchyNode {
            name: layer.name.clone(),
            child_node_ids: Vec::new(),
            child_object_ids,
            transform: TyTransformF64::default(),
        });
    }
    Ok(nodes)
}
