use crate::{
    EditStateMode, Result, VoxjVoxMain, VoxjWriteOptions, voxj_map_from_vox_map_entries,
    voxj_value_from_vox_value,
};
use branded_id::U32Id;
use ty_math::{TyTransformF64, TyVector3U32};
use voxcore::{
    BVoxObject, VoxExt, VoxHierarchyNode, VoxMain, VoxObject, VoxPalette, VoxValueColumn,
    VoxValuePool, VoxValuePoolValues,
};
use voxj::{
    CostVoxjObject, EncodeBase64, VoxjEditObject, VoxjEditState, VoxjFile, VoxjHierarchyNode,
    VoxjMain, VoxjPalette, VoxjProperty, VoxjRuntimeState, VoxjTransform, VoxjValuePool,
    objects::{
        Result as ObjectsResult, VoxjDecodedObject, encode_voxj_object_optimized,
        voxj_palette_material_counts,
    },
};

/// The voxj format version stamped on every written document. A [`VoxMain`]
/// carries no version.
const VOXJ_FORMAT_VERSION: u32 = 1;

/// Encodes a [`VoxjVoxMain`] into a [`VoxjFile`] carrying its `ext` block, the
/// inverse of [`from_voxj_file`](crate::from_voxj_file()). An empty ext writes
/// no block, and `options` can drop the block.
///
/// Value pools, palettes, objects, and hierarchy nodes are emitted in listing
/// order, and ids are written as array indices. Call
/// [`VoxMain::gc`](voxcore::VoxMain::gc) first after any removal or move so
/// each id equals its listing index and the cross references land intact.
///
/// # Arguments
/// 1. `dependencies` - encodes the base64 blocks and costs a searched block's
///    candidates. A pinned pair is never costed.
pub fn to_voxj_file<D: EncodeBase64 + CostVoxjObject>(
    dependencies: &D,
    main: &VoxjVoxMain,
    options: &VoxjWriteOptions,
) -> Result<VoxjFile> {
    let value_pools = main
        .iter_value_pools()
        .map(|(_, value_pool)| voxj_value_pool_from_vox_value_pool(value_pool))
        .collect::<Vec<_>>();

    let palettes = main
        .iter_palettes()
        .map(|(_, palette)| voxj_palette_from_vox_palette(palette))
        .collect::<Vec<_>>();

    let objects = main
        .iter_objects()
        .map(|(object_id, _)| {
            let decoded = voxj_decoded_object_from_vox_object(main, object_id);
            let material_counts = voxj_palette_material_counts(&decoded.layers, &palettes)?;
            encode_voxj_object_optimized(
                dependencies,
                &decoded,
                &material_counts,
                options.position_encoding,
                options.sample_encoding,
            )
        })
        .collect::<ObjectsResult<Vec<_>>>()?;

    let nodes = main
        .iter_hierarchy_nodes()
        .map(|(_, node)| voxj_hierarchy_node_from_vox_hierarchy_node(node))
        .collect();

    let root_nodes = main
        .root_hierarchy_node_ids()
        .iter()
        .map(|node_id| node_id.to_u32() as usize)
        .collect();

    // Editor state, aligned by index with the objects. Each entry is the
    // object's build volume.
    let edit_state = emit_edit_state(main, options.edit_state).then(|| VoxjEditState {
        objects: main
            .iter_objects()
            .map(|(_, object)| {
                let bounds = object.bounds();
                let origin = object.origin();
                VoxjEditObject {
                    bounds: bounds.to_array(),
                    origin: origin.to_array(),
                }
            })
            .collect(),
    });

    // An empty ext is no block.
    let ext = if options.ext && !main.ext().is_empty() {
        Some(voxj_map_from_vox_map_entries(main.ext().slots()))
    } else {
        None
    };

    Ok(VoxjFile {
        version: VOXJ_FORMAT_VERSION,
        main: VoxjMain {
            runtime_state: VoxjRuntimeState {
                value_pools,
                palettes,
                objects,
                nodes,
                root_nodes,
            },
            edit_state,
            ext,
        },
    })
}

/// Whether the document records editor build volumes under `mode`. Auto records
/// them only when some object carries margin around its live voxels. An
/// already-tight object recreates its build volume on load.
fn emit_edit_state<T: VoxExt>(main: &VoxMain<T>, mode: EditStateMode) -> bool {
    match mode {
        EditStateMode::Always => true,
        EditStateMode::Never => false,
        EditStateMode::Auto => main.iter_objects().any(|(_, object)| !is_tight(object)),
    }
}

/// Whether an object's build volume already equals its tight runtime grid. Such
/// an object needs no edit-state entry to be recovered on load.
fn is_tight(object: &VoxObject) -> bool {
    let bounds = object.bounds();
    match object.live_extent() {
        Some((min, size)) => min == TyVector3U32::default() && size == bounds,
        None => bounds == TyVector3U32::default(),
    }
}

/// Converts a [`VoxValuePool`] into a [`VoxjValuePool`], kind by kind, its
/// values in listing order. Every kind maps one to one. `json` values recurse
/// through [`voxj_value_from_vox_value`].
fn voxj_value_pool_from_vox_value_pool(value_pool: &VoxValuePool) -> VoxjValuePool {
    match value_pool.values() {
        VoxValuePoolValues::Bool(flags) => VoxjValuePool::Bool(copied(flags)),

        VoxValuePoolValues::Float(numbers) => VoxjValuePool::Float(copied(numbers)),

        VoxValuePoolValues::Int(numbers) => VoxjValuePool::Int(copied(numbers)),

        VoxValuePoolValues::Json(values) => VoxjValuePool::Json(
            values
                .iter()
                .map(|(_, value)| voxj_value_from_vox_value(value))
                .collect(),
        ),

        VoxValuePoolValues::String(texts) => {
            VoxjValuePool::String(texts.iter().map(|(_, text)| text.clone()).collect())
        }

        VoxValuePoolValues::Vec2Float(vectors) => VoxjValuePool::Vec2Float(copied(vectors)),

        VoxValuePoolValues::Vec2Int(vectors) => VoxjValuePool::Vec2Int(copied(vectors)),

        VoxValuePoolValues::Vec3Float(vectors) => VoxjValuePool::Vec3Float(copied(vectors)),

        VoxValuePoolValues::Vec3Int(vectors) => VoxjValuePool::Vec3Int(copied(vectors)),

        VoxValuePoolValues::Vec4Float(vectors) => VoxjValuePool::Vec4Float(copied(vectors)),

        VoxValuePoolValues::Vec4Int(vectors) => VoxjValuePool::Vec4Int(copied(vectors)),
    }
}

fn copied<T: Copy>(values: VoxValueColumn<'_, T>) -> Vec<T> {
    values.iter().map(|(_, &value)| value).collect()
}

/// Builds a [`VoxjPalette`] from a [`VoxPalette`], emitting properties and
/// materials in id order so each lands at its original index.
///
/// A voxcore property's value-pool id becomes the wire `valuePool`. `materials`
/// carries over one row per material, a value-index per property.
fn voxj_palette_from_vox_palette(palette: &VoxPalette) -> VoxjPalette {
    let properties: Vec<VoxjProperty> = palette
        .iter_properties()
        .map(|(_, property)| VoxjProperty {
            name: property.name.clone(),
            value_pool: property.value_pool_id.to_u32() as usize,
        })
        .collect();

    // Property ids, reused to read each material's row in property order.
    let property_ids: Vec<_> = palette
        .iter_properties()
        .map(|(property_id, _)| property_id)
        .collect();

    let materials = palette
        .iter_materials()
        .map(|material_id| {
            property_ids
                .iter()
                .map(|&property_id| {
                    palette
                        .value_id(material_id, property_id)
                        .expect("a material has a value id for every property")
                        .to_u32() as usize
                })
                .collect()
        })
        .collect();

    VoxjPalette {
        properties,
        materials,
    }
}

/// Builds a [`VoxjDecodedObject`] from object `object_id` of `main`, emitting
/// the tight runtime grid: one position and sample row per live voxel in
/// ascending raster order, rebased so the live voxels fill the grid from its
/// origin. The object's wider build volume, when it has margin, is recorded
/// separately in the document's edit state.
///
/// Each layer becomes a `layers` entry (its palette id equals its index).
/// Sample rows hold one material index per layer, in layer order.
///
/// # Panics
///
/// Panics if `object_id` is not one of `main`'s objects.
fn voxj_decoded_object_from_vox_object<T: VoxExt>(
    main: &VoxMain<T>,
    object_id: U32Id<BVoxObject>,
) -> VoxjDecodedObject {
    let object = main
        .object(object_id)
        .expect("object_id is one of main's objects");

    let origin = object.origin();
    // The runtime grid is the live voxels' tight extent within the build
    // volume; an empty object collapses to a [0, 0, 0] grid at the build-volume
    // origin.
    let (min, size) = object
        .live_extent()
        .unwrap_or((TyVector3U32::new(0, 0, 0), TyVector3U32::new(0, 0, 0)));

    let palette_indices: Vec<usize> = object
        .iter_layers()
        .map(|(_, palette_id)| palette_id.to_u32() as usize)
        .collect();

    // Layer ids, reused for each voxel's sample row: the wire's channel order.
    let layer_ids: Vec<_> = object.iter_layers().map(|(layer_id, _)| layer_id).collect();

    let live_count = object.live_count();
    let mut positions = Vec::with_capacity(live_count);
    let mut samples = Vec::with_capacity(live_count);
    for voxel_id in object.iter_live() {
        let position = object
            .voxel_position(voxel_id)
            .expect("a live voxel id is within the grid");
        positions.push((position - min).to_array());

        let row = layer_ids
            .iter()
            .map(|&layer_id| {
                object
                    .voxel_material(voxel_id, layer_id)
                    .expect("a live voxel has a sample for every layer")
                    .to_u32()
            })
            .collect();
        samples.push(row);
    }

    VoxjDecodedObject {
        name: object.name().to_owned(),
        layers: palette_indices,
        bounds: size.to_array(),
        origin: (origin + min.as_ivec3()).to_array(),
        positions,
        samples,
    }
}

/// Builds a [`VoxjHierarchyNode`] from a [`VoxHierarchyNode`], mapping branded
/// child ids back to indices and the transform back to its voxj form.
fn voxj_hierarchy_node_from_vox_hierarchy_node(node: &VoxHierarchyNode) -> VoxjHierarchyNode {
    VoxjHierarchyNode {
        name: node.name.clone(),
        child_nodes: node
            .child_node_ids
            .iter()
            .map(|node_id| node_id.to_u32() as usize)
            .collect(),
        child_objects: node
            .child_object_ids
            .iter()
            .map(|object_id| object_id.to_u32() as usize)
            .collect(),
        transform: voxj_transform_from_vox_transform(&node.transform),
    }
}

/// Converts a [`TyTransformF64`] into a [`VoxjTransform`].
fn voxj_transform_from_vox_transform(transform: &TyTransformF64) -> VoxjTransform {
    VoxjTransform {
        position: transform.position.to_array(),
        rotation: transform.rotation.to_array(),
        scale: transform.scale.to_array(),
    }
}

#[cfg(test)]
mod tests {
    use crate::to_voxj_file::voxj_palette_from_vox_palette;
    use voxcore::VoxPalette;

    #[test]
    fn writes_a_property_less_palette_as_empty_rows() {
        let mut palette = VoxPalette::default();
        palette.retain_material(vec![]).unwrap();
        palette.retain_material(vec![]).unwrap();

        let out = voxj_palette_from_vox_palette(&palette);
        assert!(out.properties.is_empty());
        // One empty row per material, so the material count survives.
        assert_eq!(out.materials, [Vec::<usize>::new(), Vec::new()]);
    }
}
