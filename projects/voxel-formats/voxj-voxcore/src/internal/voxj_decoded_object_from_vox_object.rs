use branded_id::U32Id;
use ty_math::TyVector3U32;
use voxcore::{BVoxObject, VoxMain};
use voxj::objects::VoxjDecodedObject;

/// Builds a [`VoxjDecodedObject`] from object `object_id` of `state`, emitting
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
/// Panics if `object_id` is not one of `state`'s objects.
pub fn voxj_decoded_object_from_vox_object<T>(
    state: &VoxMain<T>,
    object_id: U32Id<BVoxObject>,
) -> VoxjDecodedObject {
    let object = state.object(object_id).expect("the object is the state's");

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

    // Layer ids, reused for each voxel's sample row: the wire's channel
    // order.
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
