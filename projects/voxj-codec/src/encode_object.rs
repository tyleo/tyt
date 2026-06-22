use crate::{
    PositionEncoding, SampleEncoding, encode_hilbert, encode_varint, hilbert_bits, pack_bits,
    packed_width,
};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use voxj::{VoxjCodecObject, VoxjSerdeObject, VoxjSerdePositionBlock, VoxjSerdeSampleBlock};

/// Encodes one [`VoxjCodecObject`]'s geometry into a [`VoxjSerdeObject`] with the
/// given fixed position and sample encodings. `cell_counts[p]` is the cell count
/// of the palette referenced by `object.palette_refs[p]`, used to derive the bit
/// width of `packed-base64` samples;
/// [`palette_cell_counts`](crate::palette_cell_counts) computes it from the
/// document's palettes.
pub fn encode_object(
    object: &VoxjCodecObject,
    cell_counts: &[usize],
    position: PositionEncoding,
    sample: SampleEncoding,
) -> VoxjSerdeObject {
    let num_palettes = object.palette_refs.len();

    let (voxel_positions, voxel_samples) = if object.positions.is_empty() {
        (
            VoxjSerdePositionBlock::RawJson(Vec::new()),
            VoxjSerdeSampleBlock::RawJson(Vec::new()),
        )
    } else {
        let (order, position_block) = encode_positions(object, position);
        let channels = channels_in_order(&object.samples, &order, num_palettes);
        let sample_block = encode_samples(&channels, sample, cell_counts, order.len());
        (position_block, sample_block)
    };

    VoxjSerdeObject {
        name: object.name.clone(),
        palette_refs: object.palette_refs.clone(),
        bounds: object.bounds,
        voxel_positions,
        voxel_samples,
    }
}

/// Encodes the voxel positions with `encoding`, returning the canonical voxel
/// order and the block.
fn encode_positions(
    object: &VoxjCodecObject,
    encoding: PositionEncoding,
) -> (Vec<usize>, VoxjSerdePositionBlock) {
    match encoding {
        PositionEncoding::RawJson => raw_positions(&object.positions),
        PositionEncoding::BitmapBase64 => bitmap_positions(&object.positions, object.bounds),
        PositionEncoding::Hilbert => hilbert_positions(&object.positions, object.bounds),
    }
}

/// Listing order `0..n` paired with the raw block. The raw encoding never
/// reorders voxels, so positions pass through unchanged and the order is the
/// identity permutation.
fn raw_positions(positions: &[[u32; 3]]) -> (Vec<usize>, VoxjSerdePositionBlock) {
    let order = (0..positions.len()).collect();
    let block = VoxjSerdePositionBlock::RawJson(positions.to_vec());
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
fn bitmap_positions(
    positions: &[[u32; 3]],
    bounds: [u32; 3],
) -> (Vec<usize>, VoxjSerdePositionBlock) {
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
    let cells = (bounds[0] as usize) * (bounds[1] as usize) * (bounds[2] as usize);
    let mut bytes = vec![0u8; cells.div_ceil(8)];
    for &(cell, _) in &indexed {
        let c = cell as usize;
        debug_assert!(c < cells, "voxel cell {c} outside {cells}-cell bounds");
        bytes[c / 8] |= 1 << (7 - (c % 8));
    }
    let block = VoxjSerdePositionBlock::BitmapBase64(BASE64.encode(bytes));
    (order, block)
}

/// Voxel order ascending by Hilbert index, paired with the delta-varint
/// position block. Each voxel's Hilbert index is computed exactly once and
/// shared between the order permutation and the encoded deltas. Sorting
/// `(index, original_voxel)` pairs yields both in a single pass.
fn hilbert_positions(
    positions: &[[u32; 3]],
    bounds: [u32; 3],
) -> (Vec<usize>, VoxjSerdePositionBlock) {
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
    let block = VoxjSerdePositionBlock::HilbertIndexDeltaVarintBase64(
        BASE64.encode(encode_varint(&deltas)),
    );
    (order, block)
}

/// Reorders `samples[voxel][palette]` into one channel per palette, in the
/// position block's voxel order.
fn channels_in_order(samples: &[Vec<u32>], order: &[usize], num_palettes: usize) -> Vec<Vec<u32>> {
    (0..num_palettes)
        .map(|p| order.iter().map(|&i| samples[i][p]).collect())
        .collect()
}

/// Encodes the per-palette sample `channels` (already in the position block's
/// voxel order) with `encoding`. `n` is the voxel count.
fn encode_samples(
    channels: &[Vec<u32>],
    encoding: SampleEncoding,
    cell_counts: &[usize],
    n: usize,
) -> VoxjSerdeSampleBlock {
    match encoding {
        SampleEncoding::RawJson => samples_raw(channels, n),
        SampleEncoding::RleJson => samples_rle(channels),
        SampleEncoding::PackedBase64 => samples_packed(channels, cell_counts),
    }
}

/// Builds one row per voxel, each holding that voxel's cell index per palette.
/// `n` is the voxel count, sourced independently of `channels` so an object
/// with voxels but zero palettes still emits `n` empty rows (matching the
/// position block's voxel count).
fn samples_raw(channels: &[Vec<u32>], n: usize) -> VoxjSerdeSampleBlock {
    let rows = (0..n)
        .map(|k| channels.iter().map(|ch| ch[k]).collect())
        .collect();
    VoxjSerdeSampleBlock::RawJson(rows)
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

fn samples_rle(channels: &[Vec<u32>]) -> VoxjSerdeSampleBlock {
    VoxjSerdeSampleBlock::RleJson(channels.iter().map(|ch| rle_encode(ch)).collect())
}

fn samples_packed(channels: &[Vec<u32>], cell_counts: &[usize]) -> VoxjSerdeSampleBlock {
    let packed = channels
        .iter()
        .enumerate()
        .map(|(p, ch)| {
            let width = packed_width(cell_counts.get(p).copied().unwrap_or(1));
            BASE64.encode(pack_bits(ch, width))
        })
        .collect();
    VoxjSerdeSampleBlock::PackedBase64(packed)
}

#[cfg(test)]
mod tests {
    use crate::{PositionEncoding, SampleEncoding, encode_object};
    use voxj::{VoxjCodecObject, VoxjSerdeObject, VoxjSerdePositionBlock, VoxjSerdeSampleBlock};

    /// An object with voxels but zero palettes must still emit a sample block
    /// whose arity matches the position block: raw-json carries one (empty) row
    /// per voxel, and rle/packed carry zero channels.
    fn assert_zero_palette_arity(object: &VoxjSerdeObject) {
        match &object.voxel_samples {
            VoxjSerdeSampleBlock::RawJson(rows) => assert_eq!(rows.len(), 3),
            VoxjSerdeSampleBlock::RleJson(channels) => assert!(channels.is_empty()),
            VoxjSerdeSampleBlock::PackedBase64(channels) => assert!(channels.is_empty()),
        }
    }

    #[test]
    fn zero_palette_object_keeps_sample_arity() {
        assert_zero_palette_arity(&encode_object(
            &VoxjCodecObject {
                name: "o".to_owned(),
                palette_refs: Vec::new(),
                bounds: [3, 1, 1],
                positions: vec![[0, 0, 0], [1, 0, 0], [2, 0, 0]],
                samples: vec![Vec::new(), Vec::new(), Vec::new()],
            },
            &[],
            PositionEncoding::RawJson,
            SampleEncoding::RawJson,
        ));
    }

    /// A fixed encoding produces exactly the requested blocks.
    #[test]
    fn fixed_encoding_uses_requested_blocks() {
        let object = VoxjCodecObject {
            name: "o".to_owned(),
            palette_refs: vec![0],
            bounds: [2, 1, 1],
            positions: vec![[0, 0, 0], [1, 0, 0]],
            samples: vec![vec![1], vec![2]],
        };
        let object = encode_object(
            &object,
            &[4],
            PositionEncoding::BitmapBase64,
            SampleEncoding::PackedBase64,
        );
        assert!(matches!(
            object.voxel_positions,
            VoxjSerdePositionBlock::BitmapBase64(_)
        ));
        assert!(matches!(
            object.voxel_samples,
            VoxjSerdeSampleBlock::PackedBase64(_)
        ));
    }
}
