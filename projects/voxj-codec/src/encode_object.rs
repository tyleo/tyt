use crate::{VoxelData, hilbert_bits, hilbert_encode, pack_bits, packed_width, varint_encode};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use voxj::{PositionBlock, PositionEncoding, SampleBlock, SampleEncoding, VoxjObject};

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
    voxel_positions: PositionBlock,
    voxel_samples: SampleBlock,
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
fn empty_blocks() -> (PositionBlock, SampleBlock) {
    (
        PositionBlock::RawJson(Vec::new()),
        SampleBlock::RawJson(Vec::new()),
    )
}

/// Encodes the voxel positions with `encoding`, returning the canonical voxel
/// order (so sample channels can be reordered to match) and the block.
fn encode_positions(data: &VoxelData, encoding: PositionEncoding) -> (Vec<usize>, PositionBlock) {
    match encoding {
        PositionEncoding::RawJson => {
            let order = order_raw(data.positions.len());
            let block = positions_raw(&data.positions, &order);
            (order, block)
        }
        PositionEncoding::BitmapBase64 => (
            order_bitmap(&data.positions, data.bounds),
            positions_bitmap(&data.positions, data.bounds),
        ),
        PositionEncoding::Hilbert => {
            let bits = hilbert_bits(data.bounds);
            (
                order_hilbert(&data.positions, bits),
                positions_hilbert(&data.positions, bits),
            )
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
) -> SampleBlock {
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

/// Voxel order ascending by Hilbert index.
fn order_hilbert(positions: &[[u32; 3]], bits: u32) -> Vec<usize> {
    let mut order = order_raw(positions.len());
    order.sort_by_key(|&i| {
        let [x, y, z] = positions[i];
        hilbert_encode(x, y, z, bits)
    });
    order
}

fn positions_raw(positions: &[[u32; 3]], order: &[usize]) -> PositionBlock {
    PositionBlock::RawJson(order.iter().map(|&i| positions[i]).collect())
}

fn positions_bitmap(positions: &[[u32; 3]], bounds: [u32; 3]) -> PositionBlock {
    let cells = (bounds[0] as usize) * (bounds[1] as usize) * (bounds[2] as usize);
    let mut occupancy = vec![0u32; cells];
    for &pos in positions {
        occupancy[cell_index(pos, bounds) as usize] = 1;
    }
    PositionBlock::BitmapBase64(BASE64.encode(pack_bits(&occupancy, 1)))
}

fn positions_hilbert(positions: &[[u32; 3]], bits: u32) -> PositionBlock {
    let mut indices: Vec<u64> = positions
        .iter()
        .map(|&[x, y, z]| hilbert_encode(x, y, z, bits))
        .collect();
    indices.sort_unstable();
    let mut prev = 0u64;
    let deltas: Vec<u64> = indices
        .iter()
        .map(|&i| {
            let d = i - prev;
            prev = i;
            d
        })
        .collect();
    PositionBlock::HilbertIndexDeltaVarintBase64(BASE64.encode(varint_encode(&deltas)))
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
fn samples_raw(channels: &[Vec<u32>], n: usize) -> SampleBlock {
    let rows = (0..n)
        .map(|k| channels.iter().map(|ch| ch[k]).collect())
        .collect();
    SampleBlock::RawJson(rows)
}

fn samples_rle(channels: &[Vec<u32>]) -> SampleBlock {
    SampleBlock::RleJson(channels.iter().map(|ch| rle_encode(ch)).collect())
}

fn samples_packed(channels: &[Vec<u32>], cell_counts: &[usize]) -> SampleBlock {
    let packed = channels
        .iter()
        .enumerate()
        .map(|(p, ch)| {
            let width = packed_width(cell_counts.get(p).copied().unwrap_or(1));
            BASE64.encode(pack_bits(ch, width))
        })
        .collect();
    SampleBlock::PackedBase64(packed)
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
    use voxj::{PositionBlock, PositionEncoding, SampleBlock, SampleEncoding, VoxjObject};

    /// An object with voxels but zero palettes must still emit a sample block
    /// whose arity matches the position block: raw-json carries one (empty) row
    /// per voxel, and rle/packed carry zero channels.
    fn assert_zero_palette_arity(object: &VoxjObject) {
        match &object.voxel_samples {
            SampleBlock::RawJson(rows) => assert_eq!(rows.len(), 3),
            SampleBlock::RleJson(channels) => assert!(channels.is_empty()),
            SampleBlock::PackedBase64(channels) => assert!(channels.is_empty()),
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
            PositionBlock::BitmapBase64(_)
        ));
        assert!(matches!(object.voxel_samples, SampleBlock::PackedBase64(_)));
    }
}
