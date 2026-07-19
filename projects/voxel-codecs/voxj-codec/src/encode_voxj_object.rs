use crate::{
    Error, PositionEncoding, Result, SampleChannels, SampleEncoding, VoxjDecodedObject,
    encode_hilbert, encode_varint, hilbert_bits, pack_bits, packed_width,
};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use voxj::{VoxjObject, VoxjPositionBlock, VoxjSampleBlock};

/// Encodes one [`VoxjDecodedObject`] into a [`VoxjObject`] with the given fixed
/// position and sample encodings. `material_counts` comes from
/// [`voxj_palette_material_counts`](crate::voxj_palette_material_counts()); a
/// layer is sampled iff its count is above zero, and the sample block carries
/// one channel per sampled layer.
pub fn encode_voxj_object(
    object: &VoxjDecodedObject,
    material_counts: &[usize],
    position: PositionEncoding,
    sample: SampleEncoding,
) -> Result<VoxjObject> {
    let layout = SampleChannels::from_material_counts(material_counts);
    validate_object_shape(object, layout.channels())?;

    let (voxel_positions, voxel_samples) = if object.positions.is_empty() {
        // No voxels, but still one (empty) channel per sampled layer so the
        // block's arity matches the sampled layers.
        (
            VoxjPositionBlock::RawJson(Vec::new()),
            VoxjSampleBlock::RawJson(vec![Vec::new(); layout.channels()]),
        )
    } else {
        let (order, position_block) = encode_positions(object, position);
        let channels = channels_in_order(&object.samples, &order, layout.channels());
        let sample_block = encode_samples(&channels, sample, layout.counts());
        (position_block, sample_block)
    };

    Ok(VoxjObject {
        name: object.name.clone(),
        layers: object.layers.clone(),
        bounds: object.bounds,
        origin: object.origin,
        voxel_positions,
        voxel_samples,
    })
}

/// Validates that `object`'s samples are rectangular: one row per voxel, each
/// holding one material index per sampled layer. The encoders index
/// `samples[voxel][channel]` directly, so a ragged object would otherwise panic
/// or silently drop values.
fn validate_object_shape(object: &VoxjDecodedObject, num_channels: usize) -> Result<()> {
    if object.samples.len() != object.positions.len() {
        return Err(Error::Invalid(format!(
            "object \"{}\" has {} sample rows but {} positions",
            object.name,
            object.samples.len(),
            object.positions.len()
        )));
    }
    if let Some(row) = object.samples.iter().find(|row| row.len() != num_channels) {
        return Err(Error::Invalid(format!(
            "object \"{}\" has a sample row of {} values but has {num_channels} sampled layers",
            object.name,
            row.len()
        )));
    }
    Ok(())
}

/// Encodes the voxel positions with `encoding`, returning the canonical voxel
/// order and the block.
fn encode_positions(
    object: &VoxjDecodedObject,
    encoding: PositionEncoding,
) -> (Vec<usize>, VoxjPositionBlock) {
    match encoding {
        PositionEncoding::RawJson => raw_positions(&object.positions),
        PositionEncoding::BitmapBase64 => bitmap_positions(&object.positions, object.bounds),
        PositionEncoding::Hilbert => hilbert_positions(&object.positions, object.bounds),
    }
}

/// Listing order `0..n` paired with the raw block. The raw encoding never
/// reorders voxels, so positions pass through unchanged and the order is the
/// identity permutation.
fn raw_positions(positions: &[[u32; 3]]) -> (Vec<usize>, VoxjPositionBlock) {
    let order = (0..positions.len()).collect();
    let block = VoxjPositionBlock::RawJson(positions.to_vec());
    (order, block)
}

/// Raster cell index `k = x*Y*Z + y*Z + z`.
fn cell_index(pos: [u32; 3], bounds: [u32; 3]) -> u64 {
    let [x, y, z] = pos;
    x as u64 * bounds[1] as u64 * bounds[2] as u64 + y as u64 * bounds[2] as u64 + z as u64
}

/// Voxel order ascending by raster cell index, paired with a dense occupancy
/// bitmap: bit `k` (MSB-first, 8 per byte) is set when raster cell `k` holds a
/// voxel. Each cell index is computed exactly once, by sorting `(cell, voxel)`
/// pairs, and shared between the order permutation and the packed bits.
fn bitmap_positions(positions: &[[u32; 3]], bounds: [u32; 3]) -> (Vec<usize>, VoxjPositionBlock) {
    let mut indexed: Vec<(u64, usize)> = positions
        .iter()
        .enumerate()
        .map(|(i, &pos)| (cell_index(pos, bounds), i))
        .collect();
    indexed.sort_unstable();

    let order = indexed.iter().map(|&(_, i)| i).collect();

    // Pack the bits directly instead of filling a one-u32-per-cell occupancy
    // buffer and packing it afterward. Every position lies within bounds, so
    // its cell index is < cells.
    let cells = bounds[0] as usize * bounds[1] as usize * bounds[2] as usize;
    let mut bytes = vec![0u8; cells.div_ceil(8)];
    for &(cell, _) in &indexed {
        let c = cell as usize;
        debug_assert!(c < cells, "voxel cell {c} outside {cells}-cell bounds");
        bytes[c / 8] |= 1 << (7 - (c % 8));
    }
    let block = VoxjPositionBlock::BitmapBase64(BASE64.encode(bytes));
    (order, block)
}

/// Voxel order ascending by Hilbert index, paired with the delta-varint
/// position block. Each voxel's Hilbert index is computed exactly once and
/// shared between the order permutation and the encoded deltas. Sorting
/// `(index, original_voxel)` pairs yields both in a single pass.
fn hilbert_positions(positions: &[[u32; 3]], bounds: [u32; 3]) -> (Vec<usize>, VoxjPositionBlock) {
    let bits = hilbert_bits(bounds);
    let mut indexed: Vec<(u64, usize)> = positions
        .iter()
        .enumerate()
        .map(|(i, &[x, y, z])| (encode_hilbert(x, y, z, bits), i))
        .collect();
    indexed.sort_unstable();

    let order = indexed.iter().map(|&(_, i)| i).collect();
    let mut prev = 0u64;
    let deltas: Vec<u64> = indexed
        .iter()
        .map(|&(index, _)| {
            let d = index - prev;
            prev = index;
            d
        })
        .collect();
    let block = VoxjPositionBlock::HilbertDeltaVarintBase64(BASE64.encode(encode_varint(&deltas)));
    (order, block)
}

/// Reorders `samples[voxel][channel]` into one channel per sampled layer, in
/// the position block's voxel order.
fn channels_in_order(samples: &[Vec<u32>], order: &[usize], num_channels: usize) -> Vec<Vec<u32>> {
    (0..num_channels)
        .map(|c| order.iter().map(|&i| samples[i][c]).collect())
        .collect()
}

/// Encodes the per-sampled-layer `channels` with `encoding`.
fn encode_samples(
    channels: &[Vec<u32>],
    encoding: SampleEncoding,
    channel_counts: &[usize],
) -> VoxjSampleBlock {
    match encoding {
        SampleEncoding::RawJson => samples_raw(channels),
        SampleEncoding::RleJson => samples_rle(channels),
        SampleEncoding::PackedBase64 => samples_packed(channels, channel_counts),
    }
}

/// Emits the channels as raw JSON, one array per sampled layer holding that
/// layer's material index for every voxel. The same per-channel layout as the
/// rle-json and packed-base64 sample encodings, just left unencoded. An object
/// with voxels but no sampled layers emits zero channels.
fn samples_raw(channels: &[Vec<u32>]) -> VoxjSampleBlock {
    VoxjSampleBlock::RawJson(channels.to_vec())
}

/// Flat run-length encoding: `[value1, count1, value2, count2, ...]`.
fn rle_encode(channel: &[u32]) -> Vec<u32> {
    let mut out = Vec::new();
    let mut iter = channel.iter().copied();
    let Some(mut value) = iter.next() else {
        return out;
    };
    let mut count = 1u32;
    for v in iter {
        if v == value {
            count += 1;
        } else {
            out.push(value);
            out.push(count);
            value = v;
            count = 1;
        }
    }
    out.push(value);
    out.push(count);
    out
}

fn samples_rle(channels: &[Vec<u32>]) -> VoxjSampleBlock {
    VoxjSampleBlock::RleJson(channels.iter().map(|ch| rle_encode(ch)).collect())
}

fn samples_packed(channels: &[Vec<u32>], channel_counts: &[usize]) -> VoxjSampleBlock {
    let packed = channels
        .iter()
        .enumerate()
        .map(|(c, ch)| {
            let width = packed_width(channel_counts[c]);
            BASE64.encode(pack_bits(ch, width))
        })
        .collect();
    VoxjSampleBlock::PackedBase64(packed)
}

#[cfg(test)]
mod tests {
    use crate::{PositionEncoding, SampleEncoding, VoxjDecodedObject, encode_voxj_object};
    use voxj::{VoxjObject, VoxjPositionBlock, VoxjSampleBlock};

    /// The object's sample block carries zero channels under every encoding:
    /// there are no sampled layers to carry, and the voxel count lives in the
    /// position block, not the samples.
    fn assert_no_channels(object: &VoxjObject) {
        match &object.voxel_samples {
            VoxjSampleBlock::RawJson(channels) => assert!(channels.is_empty()),
            VoxjSampleBlock::RleJson(channels) => assert!(channels.is_empty()),
            VoxjSampleBlock::PackedBase64(channels) => assert!(channels.is_empty()),
        }
    }

    #[test]
    fn zero_layer_object_keeps_sample_arity() {
        assert_no_channels(
            &encode_voxj_object(
                &VoxjDecodedObject {
                    name: "o".to_owned(),
                    layers: Vec::new(),
                    bounds: [3, 1, 1],
                    origin: [0, 0, 0],
                    positions: vec![[0, 0, 0], [1, 0, 0], [2, 0, 0]],
                    samples: vec![Vec::new(), Vec::new(), Vec::new()],
                },
                &[],
                PositionEncoding::RawJson,
                SampleEncoding::RawJson,
            )
            .unwrap(),
        );
    }

    /// A layer over a palette with no materials is unsampled: the layer stays
    /// in `layers`, but the sample block holds no channel for it.
    #[test]
    fn unsampled_layer_carries_no_channel() {
        let object = encode_voxj_object(
            &VoxjDecodedObject {
                name: "o".to_owned(),
                layers: vec![0],
                bounds: [2, 1, 1],
                origin: [0, 0, 0],
                positions: vec![[0, 0, 0], [1, 0, 0]],
                samples: vec![Vec::new(), Vec::new()],
            },
            &[0],
            PositionEncoding::RawJson,
            SampleEncoding::RawJson,
        )
        .unwrap();
        assert_eq!(object.layers, vec![0]);
        assert_no_channels(&object);
    }

    /// A fixed encoding produces exactly the requested blocks.
    #[test]
    fn fixed_encoding_uses_requested_blocks() {
        let object = VoxjDecodedObject {
            name: "o".to_owned(),
            layers: vec![0],
            bounds: [2, 1, 1],
            origin: [0, 0, 0],
            positions: vec![[0, 0, 0], [1, 0, 0]],
            samples: vec![vec![1], vec![2]],
        };
        let object = encode_voxj_object(
            &object,
            &[4],
            PositionEncoding::BitmapBase64,
            SampleEncoding::PackedBase64,
        )
        .unwrap();
        assert!(matches!(
            object.voxel_positions,
            VoxjPositionBlock::BitmapBase64(_)
        ));
        assert!(matches!(
            object.voxel_samples,
            VoxjSampleBlock::PackedBase64(_)
        ));
    }

    /// A ragged object (a sample row whose arity differs from the sampled
    /// layer count, or a sample count that differs from the voxel count) is
    /// rejected rather than panicking or dropping values mid-encode.
    #[test]
    fn rejects_ragged_object() {
        let wrong_row_arity = VoxjDecodedObject {
            name: "o".to_owned(),
            layers: vec![0, 1],
            bounds: [1, 1, 1],
            origin: [0, 0, 0],
            positions: vec![[0, 0, 0]],
            samples: vec![vec![1]],
        };
        assert!(
            encode_voxj_object(
                &wrong_row_arity,
                &[4, 4],
                PositionEncoding::RawJson,
                SampleEncoding::RawJson,
            )
            .is_err()
        );

        let wrong_sample_count = VoxjDecodedObject {
            name: "o".to_owned(),
            layers: vec![0],
            bounds: [2, 1, 1],
            origin: [0, 0, 0],
            positions: vec![[0, 0, 0], [1, 0, 0]],
            samples: vec![vec![1]],
        };
        assert!(
            encode_voxj_object(
                &wrong_sample_count,
                &[4],
                PositionEncoding::RawJson,
                SampleEncoding::RawJson,
            )
            .is_err()
        );
    }
}
