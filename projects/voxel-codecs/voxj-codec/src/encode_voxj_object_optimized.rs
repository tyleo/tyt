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

/// Encodes one [`VoxjDecodedObject`] into a [`VoxjObject`], pinning each `Some`
/// block and searching each `None` block for the smallest deflated result. Both
/// `None` is the full smallest search; both `Some` is a fixed encoding.
/// `cell_counts` comes from
/// [`voxj_palette_cell_counts`](crate::voxj_palette_cell_counts()).
pub fn encode_voxj_object_optimized(
    object: &VoxjDecodedObject,
    cell_counts: &[usize],
    position: Option<PositionEncoding>,
    sample: Option<SampleEncoding>,
) -> Result<VoxjObject> {
    if object.positions.is_empty() {
        // No voxels: every encoding emits the same raw blocks, so there is
        // nothing to search.
        return encode_voxj_object(
            object,
            cell_counts,
            PositionEncoding::RawJson,
            SampleEncoding::RawJson,
        );
    }
    let positions = match position {
        Some(position) => vec![position],
        None => candidate_positions(object.bounds),
    };
    let samples = match sample {
        Some(sample) => vec![sample],
        None => vec![SampleEncoding::RleJson, SampleEncoding::PackedBase64],
    };
    let smallest = positions
        .into_iter()
        .flat_map(|position| samples.iter().map(move |&sample| (position, sample)))
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
    use crate::{
        PositionEncoding, SampleEncoding, VoxjDecodedObject, encode_voxj_object_optimized,
    };
    use voxj::{VoxjPositionBlock, VoxjSampleBlock};

    fn object() -> VoxjDecodedObject {
        VoxjDecodedObject {
            name: "o".to_owned(),
            palette_refs: vec![0],
            bounds: [2, 1, 1],
            origin: [0, 0, 0],
            positions: vec![[0, 0, 0], [1, 0, 0]],
            samples: vec![vec![1], vec![2]],
        }
    }

    /// The full both-`None` search keeps an object's sample arity even when it
    /// has voxels but zero palettes, since there are no channels to carry.
    #[test]
    fn zero_palette_object_keeps_sample_arity() {
        let object = encode_voxj_object_optimized(
            &VoxjDecodedObject {
                name: "o".to_owned(),
                palette_refs: Vec::new(),
                bounds: [3, 1, 1],
                origin: [0, 0, 0],
                positions: vec![[0, 0, 0], [1, 0, 0], [2, 0, 0]],
                samples: vec![Vec::new(), Vec::new(), Vec::new()],
            },
            &[],
            None,
            None,
        )
        .unwrap();
        match &object.voxel_samples {
            VoxjSampleBlock::RawJson(channels) => assert!(channels.is_empty()),
            VoxjSampleBlock::RleJson(channels) => assert!(channels.is_empty()),
            VoxjSampleBlock::PackedBase64(channels) => assert!(channels.is_empty()),
        }
    }

    /// A pinned position holds while the sample block is searched, and vice
    /// versa, so the pinned block always lands on the requested encoding.
    #[test]
    fn pinned_block_is_honored() {
        let pinned_position =
            encode_voxj_object_optimized(&object(), &[4], Some(PositionEncoding::RawJson), None)
                .unwrap();
        assert!(matches!(
            pinned_position.voxel_positions,
            VoxjPositionBlock::RawJson(_)
        ));

        let pinned_sample =
            encode_voxj_object_optimized(&object(), &[4], None, Some(SampleEncoding::RawJson))
                .unwrap();
        assert!(matches!(
            pinned_sample.voxel_samples,
            VoxjSampleBlock::RawJson(_)
        ));
    }
}
