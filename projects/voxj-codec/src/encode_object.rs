use crate::{
    PositionEncoding, SampleEncoding, VoxelData, encode_hilbert, encode_varint, hilbert_bits,
    pack_bits, packed_width,
};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use voxj::{VoxjObject, VoxjPositionBlock, VoxjSampleBlock};

/// Encodes one object's geometry into a [`VoxjObject`] with the given fixed
/// position and sample encodings.
pub fn encode_object(
    name: String,
    palette_refs: Vec<usize>,
    data: VoxelData,
    position: PositionEncoding,
    sample: SampleEncoding,
) -> VoxjObject {
    let num_palettes = palette_refs.len();
    let (voxel_positions, voxel_samples) = if data.positions.is_empty() {
        empty_blocks()
    } else {
        let (order, position_block) = encode_positions(&data, position);
        let channels = channels_in_order(&data.samples, &order, num_palettes);
        let sample_block =
            encode_samples(&channels, sample, &data.palette_cell_counts, order.len());
        (position_block, sample_block)
    };
    object(
        name,
        palette_refs,
        data.bounds,
        voxel_positions,
        voxel_samples,
    )
}

/// Assembles a [`VoxjObject`] from its already-encoded blocks.
fn object(
    name: String,
    palette_refs: Vec<usize>,
    bounds: [u32; 3],
    voxel_positions: VoxjPositionBlock,
    voxel_samples: VoxjSampleBlock,
) -> VoxjObject {
    VoxjObject {
        name,
        palette_refs,
        bounds,
        voxel_positions,
        voxel_samples,
    }
}

/// Empty object: raw-json empties for both blocks (0 voxels -> 0 rows).
fn empty_blocks() -> (VoxjPositionBlock, VoxjSampleBlock) {
    (
        VoxjPositionBlock::RawJson(Vec::new()),
        VoxjSampleBlock::RawJson(Vec::new()),
    )
}

/// Encodes the voxel positions with `encoding`, returning the canonical voxel
/// order (so sample channels can be reordered to match) and the block.
fn encode_positions(
    data: &VoxelData,
    encoding: PositionEncoding,
) -> (Vec<usize>, VoxjPositionBlock) {
    match encoding {
        PositionEncoding::RawJson => {
            let order = order_raw(data.positions.len());
            let block = positions_raw(&data.positions);
            (order, block)
        }
        PositionEncoding::BitmapBase64 => (
            order_bitmap(&data.positions, data.bounds),
            positions_bitmap(&data.positions, data.bounds),
        ),
        PositionEncoding::Hilbert => {
            let bits = hilbert_bits(data.bounds);
            hilbert_positions(&data.positions, bits)
        }
    }
}

/// Encodes the per-palette sample `channels` (already in the position block's
/// voxel order) with `encoding`. `n` is the voxel count.
fn encode_samples(
    channels: &[Vec<u32>],
    encoding: SampleEncoding,
    cell_counts: &[usize],
    n: usize,
) -> VoxjSampleBlock {
    match encoding {
        SampleEncoding::RawJson => samples_raw(channels, n),
        SampleEncoding::RleJson => samples_rle(channels),
        SampleEncoding::PackedBase64 => samples_packed(channels, cell_counts),
    }
}

/// Listing order: `0..n`.
fn order_raw(n: usize) -> Vec<usize> {
    (0..n).collect()
}

/// Raster cell index `k = x*Y*Z + y*Z + z`.
fn cell_index(pos: [u32; 3], bounds: [u32; 3]) -> u64 {
    let [x, y, z] = pos;
    x as u64 * bounds[1] as u64 * bounds[2] as u64 + y as u64 * bounds[2] as u64 + z as u64
}

/// Voxel order ascending by raster cell index.
fn order_bitmap(positions: &[[u32; 3]], bounds: [u32; 3]) -> Vec<usize> {
    let mut order = order_raw(positions.len());
    order.sort_by_key(|&i| cell_index(positions[i], bounds));
    order
}

/// Voxel order ascending by Hilbert index, paired with the delta-varint
/// position block. Each voxel's Hilbert index is computed exactly once and
/// shared between the order permutation and the encoded deltas — sorting
/// `(index, original_voxel)` pairs yields both in a single pass. (Computing the
/// order via `sort_by_key(encode_hilbert)` would re-encode on every comparison,
/// an O(n log n) blow-up, and then re-encode again to build the deltas.)
fn hilbert_positions(positions: &[[u32; 3]], bits: u32) -> (Vec<usize>, VoxjPositionBlock) {
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
    let block =
        VoxjPositionBlock::HilbertIndexDeltaVarintBase64(BASE64.encode(encode_varint(&deltas)));
    (order, block)
}

/// Raw block in listing order: the raw encoding never reorders voxels, so the
/// positions pass through unchanged. The paired `order` (see [`order_raw`]) is
/// the identity permutation; it still drives sample-channel reordering, a no-op
/// here.
fn positions_raw(positions: &[[u32; 3]]) -> VoxjPositionBlock {
    VoxjPositionBlock::RawJson(positions.to_vec())
}

fn positions_bitmap(positions: &[[u32; 3]], bounds: [u32; 3]) -> VoxjPositionBlock {
    let cells = (bounds[0] as usize) * (bounds[1] as usize) * (bounds[2] as usize);
    let mut occupancy = vec![0u32; cells];
    for &pos in positions {
        occupancy[cell_index(pos, bounds) as usize] = 1;
    }
    VoxjPositionBlock::BitmapBase64(BASE64.encode(pack_bits(&occupancy, 1)))
}

/// Reorders `samples[voxel][palette]` into one channel per palette, in the
/// position block's voxel order.
fn channels_in_order(samples: &[Vec<u32>], order: &[usize], num_palettes: usize) -> Vec<Vec<u32>> {
    (0..num_palettes)
        .map(|p| order.iter().map(|&i| samples[i][p]).collect())
        .collect()
}

/// Builds one row per voxel, each holding that voxel's cell index per palette.
/// `n` is the voxel count, sourced independently of `channels` so an object with
/// voxels but zero palettes still emits `n` empty rows (matching the position
/// block's voxel count).
fn samples_raw(channels: &[Vec<u32>], n: usize) -> VoxjSampleBlock {
    let rows = (0..n)
        .map(|k| channels.iter().map(|ch| ch[k]).collect())
        .collect();
    VoxjSampleBlock::RawJson(rows)
}

fn samples_rle(channels: &[Vec<u32>]) -> VoxjSampleBlock {
    VoxjSampleBlock::RleJson(channels.iter().map(|ch| rle_encode(ch)).collect())
}

fn samples_packed(channels: &[Vec<u32>], cell_counts: &[usize]) -> VoxjSampleBlock {
    let packed = channels
        .iter()
        .enumerate()
        .map(|(p, ch)| {
            let width = packed_width(cell_counts.get(p).copied().unwrap_or(1));
            BASE64.encode(pack_bits(ch, width))
        })
        .collect();
    VoxjSampleBlock::PackedBase64(packed)
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

#[cfg(test)]
mod tests {
    use crate::encode_object;
    use crate::{PositionEncoding, SampleEncoding};
    use voxj::{VoxjObject, VoxjPositionBlock, VoxjSampleBlock};

    /// An object with voxels but zero palettes must still emit a sample block
    /// whose arity matches the position block: raw-json carries one (empty) row
    /// per voxel, and rle/packed carry zero channels.
    fn assert_zero_palette_arity(object: &VoxjObject) {
        match &object.voxel_samples {
            VoxjSampleBlock::RawJson(rows) => assert_eq!(rows.len(), 3),
            VoxjSampleBlock::RleJson(channels) => assert!(channels.is_empty()),
            VoxjSampleBlock::PackedBase64(channels) => assert!(channels.is_empty()),
        }
    }

    #[test]
    fn zero_palette_object_keeps_sample_arity() {
        assert_zero_palette_arity(&encode_object(
            "o".to_owned(),
            Vec::new(),
            super::VoxelData {
                positions: vec![[0, 0, 0], [1, 0, 0], [2, 0, 0]],
                samples: vec![Vec::new(), Vec::new(), Vec::new()],
                bounds: [3, 1, 1],
                palette_cell_counts: Vec::new(),
            },
            PositionEncoding::RawJson,
            SampleEncoding::RawJson,
        ));
    }

    /// A fixed encoding produces exactly the requested blocks.
    #[test]
    fn fixed_encoding_uses_requested_blocks() {
        let data = super::VoxelData {
            positions: vec![[0, 0, 0], [1, 0, 0]],
            samples: vec![vec![1], vec![2]],
            bounds: [2, 1, 1],
            palette_cell_counts: vec![4],
        };
        let object = encode_object(
            "o".to_owned(),
            vec![0],
            data,
            PositionEncoding::BitmapBase64,
            SampleEncoding::PackedBase64,
        );
        assert!(matches!(
            object.voxel_positions,
            VoxjPositionBlock::BitmapBase64(_)
        ));
        assert!(matches!(
            object.voxel_samples,
            VoxjSampleBlock::PackedBase64(_)
        ));
    }
}
