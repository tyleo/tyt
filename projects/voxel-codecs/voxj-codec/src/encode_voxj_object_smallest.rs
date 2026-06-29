use crate::{
    PositionEncoding, Result, SampleEncoding, VoxjDecodedObject, encode_voxj_object, hilbert_bits,
};
use flate2::{Compression, write::DeflateEncoder};
use std::io::Write;
use voxj::VoxjObject;

/// Skip the dense bitmap candidate above this many cells to bound memory.
const MAX_BITMAP_CELLS: u64 = 8_000_000;

/// Hilbert positions are only valid for `bits <= 17` (every bounds dimension
/// `<= 131072`); above that the format requires bitmap or raw instead.
const MAX_HILBERT_BITS: u32 = 17;

/// Encodes one [`VoxjDecodedObject`] into a [`VoxjObject`], trying every
/// applicable non-raw encoding pairing, deflating each, and keeping the
/// smallest. The canonical shipping form. `cell_counts` comes from
/// [`voxj_palette_cell_counts`](crate::voxj_palette_cell_counts()).
pub fn encode_voxj_object_smallest(
    object: &VoxjDecodedObject,
    cell_counts: &[usize],
) -> Result<VoxjObject> {
    if object.positions.is_empty() {
        return encode_voxj_object(
            object,
            cell_counts,
            PositionEncoding::RawJson,
            SampleEncoding::RawJson,
        );
    }
    let smallest = candidate_positions(object.bounds)
        .into_iter()
        .flat_map(|position| {
            [SampleEncoding::RleJson, SampleEncoding::PackedBase64].map(|sample| (position, sample))
        })
        .map(|(position, sample)| encode_voxj_object(object, cell_counts, position, sample))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .min_by_key(deflated_len)
        .expect("at least one candidate");
    Ok(smallest)
}

/// The applicable non-raw position encodings for `bounds`, falling back to raw
/// only when neither bitmap nor Hilbert applies.
fn candidate_positions(bounds: [u32; 3]) -> Vec<PositionEncoding> {
    let mut positions = Vec::new();

    let cells = bounds[0] as u64 * bounds[1] as u64 * bounds[2] as u64;
    if cells <= MAX_BITMAP_CELLS {
        positions.push(PositionEncoding::BitmapBase64);
    }

    if hilbert_bits(bounds) <= MAX_HILBERT_BITS {
        positions.push(PositionEncoding::Hilbert);
    }

    if positions.is_empty() {
        positions.push(PositionEncoding::RawJson);
    }

    positions
}

/// Deflated byte length of an object's two blocks serialized together.
fn deflated_len(object: &VoxjObject) -> usize {
    let Ok(json) = serde_json::to_vec(&(&object.voxel_positions, &object.voxel_samples)) else {
        return usize::MAX;
    };
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
    let _ = encoder.write_all(&json);
    encoder.finish().map_or(usize::MAX, |v| v.len())
}

#[cfg(test)]
mod tests {
    use crate::{VoxjDecodedObject, encode_voxj_object_smallest};
    use voxj::VoxjSampleBlock;

    /// An object with voxels but zero palettes emits zero sample channels under
    /// every encoding, since there are no palettes to carry.
    #[test]
    fn zero_palette_object_keeps_sample_arity() {
        let object = encode_voxj_object_smallest(
            &VoxjDecodedObject {
                name: "o".to_owned(),
                palette_refs: Vec::new(),
                bounds: [3, 1, 1],
                origin: [0, 0, 0],
                positions: vec![[0, 0, 0], [1, 0, 0], [2, 0, 0]],
                samples: vec![Vec::new(), Vec::new(), Vec::new()],
            },
            &[],
        )
        .unwrap();
        match &object.voxel_samples {
            VoxjSampleBlock::RawJson(channels) => assert!(channels.is_empty()),
            VoxjSampleBlock::RleJson(channels) => assert!(channels.is_empty()),
            VoxjSampleBlock::PackedBase64(channels) => assert!(channels.is_empty()),
        }
    }
}
