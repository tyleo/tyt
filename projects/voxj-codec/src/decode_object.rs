use crate::{decode_hilbert, decode_varint, hilbert_bits, packed_width, unpack_bits};
use base64::{DecodeError, Engine, engine::general_purpose::STANDARD as BASE64};
use std::{io, iter};
use voxj::{VoxjCodecObject, VoxjSerdeObject, VoxjSerdePositionBlock, VoxjSerdeSampleBlock};

/// Decodes one [`VoxjSerdeObject`] back into a [`VoxjCodecObject`], the inverse of
/// [`encode_object`](crate::encode_object). `cell_counts[p]` is the cell count
/// of the palette referenced by `object.palette_refs[p]`, needed to recover the
/// bit width of `packed-base64` samples;
/// [`palette_cell_counts`](crate::palette_cell_counts) computes it from the
/// document's palettes.
///
/// Bitmap and Hilbert positions decode in ascending cell / Hilbert-index order;
/// the sample channels share that same order, so each returned `positions[k]`
/// pairs with `samples[k]`.
pub fn decode_object(
    object: &VoxjSerdeObject,
    cell_counts: &[usize],
) -> io::Result<VoxjCodecObject> {
    let positions = decode_positions(&object.voxel_positions, object.bounds)?;
    let channels = decode_samples(&object.voxel_samples, cell_counts, positions.len())?;
    let samples = (0..positions.len())
        .map(|k| channels.iter().map(|channel| channel[k]).collect())
        .collect();
    Ok(VoxjCodecObject {
        name: object.name.clone(),
        palette_refs: object.palette_refs.clone(),
        bounds: object.bounds,
        positions,
        samples,
    })
}

/// Wraps a base64 decode error as invalid data.
fn invalid(error: DecodeError) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, error)
}

/// Inverse of the raster `cell_index`: `x = k / (Y*Z)`, `y = (k / Z) % Y`,
/// `z = k % Z`.
fn cell_to_position(cell: u64, bounds: [u32; 3]) -> [u32; 3] {
    let plane = bounds[1] as u64 * bounds[2] as u64;
    [
        (cell / plane) as u32,
        ((cell % plane) / bounds[2] as u64) as u32,
        (cell % bounds[2] as u64) as u32,
    ]
}

/// Decodes the position block into `[x, y, z]` positions.
fn decode_positions(block: &VoxjSerdePositionBlock, bounds: [u32; 3]) -> io::Result<Vec<[u32; 3]>> {
    Ok(match block {
        VoxjSerdePositionBlock::RawJson(positions) => positions.clone(),

        VoxjSerdePositionBlock::BitmapBase64(base64) => {
            let cells = bounds[0] as usize * bounds[1] as usize * bounds[2] as usize;
            let occupancy = unpack_bits(&BASE64.decode(base64).map_err(invalid)?, 1, cells);
            occupancy
                .iter()
                .enumerate()
                .filter(|&(_, &bit)| bit == 1)
                .map(|(cell, _)| cell_to_position(cell as u64, bounds))
                .collect()
        }

        VoxjSerdePositionBlock::HilbertIndexDeltaVarintBase64(base64) => {
            let bits = hilbert_bits(bounds);
            let mut index = 0u64;
            decode_varint(&BASE64.decode(base64).map_err(invalid)?)
                .iter()
                .map(|&delta| {
                    index += delta;
                    decode_hilbert(index, bits)
                })
                .collect()
        }
    })
}

/// Decodes the sample block into one channel (`Vec<u32>` of length `n`) per
/// referenced palette, in the position block's voxel order.
fn decode_samples(
    block: &VoxjSerdeSampleBlock,
    cell_counts: &[usize],
    n: usize,
) -> io::Result<Vec<Vec<u32>>> {
    Ok(match block {
        VoxjSerdeSampleBlock::RawJson(rows) => (0..cell_counts.len())
            .map(|p| rows.iter().map(|row| row[p]).collect())
            .collect(),

        VoxjSerdeSampleBlock::RleJson(channels) => {
            channels.iter().map(|channel| rle_decode(channel)).collect()
        }

        VoxjSerdeSampleBlock::PackedBase64(channels) => channels
            .iter()
            .enumerate()
            .map(|(p, base64)| {
                let width = packed_width(cell_counts.get(p).copied().unwrap_or(1));
                Ok(unpack_bits(
                    &BASE64.decode(base64).map_err(invalid)?,
                    width,
                    n,
                ))
            })
            .collect::<io::Result<Vec<_>>>()?,
    })
}

/// Expands flat run-length encoding `[value, count, value, count, ...]`.
fn rle_decode(rle: &[u32]) -> Vec<u32> {
    let mut out = Vec::new();
    for pair in rle.chunks_exact(2) {
        out.extend(iter::repeat_n(pair[0], pair[1] as usize));
    }
    out
}

#[cfg(test)]
mod tests {
    use crate::{PositionEncoding, SampleEncoding, decode_object, encode_object};
    use std::collections::BTreeSet;
    use voxj::VoxjCodecObject;

    const POSITIONS: [PositionEncoding; 3] = [
        PositionEncoding::RawJson,
        PositionEncoding::BitmapBase64,
        PositionEncoding::Hilbert,
    ];
    const SAMPLES: [SampleEncoding; 3] = [
        SampleEncoding::RawJson,
        SampleEncoding::RleJson,
        SampleEncoding::PackedBase64,
    ];

    /// Cell counts of the two palettes `sample_object` references.
    const CELL_COUNTS: [usize; 2] = [256, 8];

    fn sample_object() -> VoxjCodecObject {
        VoxjCodecObject {
            name: "o".to_owned(),
            palette_refs: vec![0, 1],
            bounds: [4, 4, 5],
            positions: vec![[0, 0, 0], [2, 1, 0], [1, 3, 4], [3, 3, 3]],
            samples: vec![vec![1, 0], vec![5, 2], vec![200, 7], vec![0, 1]],
        }
    }

    /// The set of `(position, samples)` pairs, order-independent, so it also
    /// proves positions and samples stay aligned through any reordering.
    fn voxel_set(object: &VoxjCodecObject) -> BTreeSet<([u32; 3], Vec<u32>)> {
        object
            .positions
            .iter()
            .copied()
            .zip(object.samples.iter().cloned())
            .collect()
    }

    #[test]
    fn round_trips_every_encoding_pair() {
        for position in POSITIONS {
            for sample in SAMPLES {
                let object = sample_object();
                let (expected, bounds) = (voxel_set(&object), object.bounds);
                let encoded = encode_object(&object, &CELL_COUNTS, position, sample);
                let decoded = decode_object(&encoded, &CELL_COUNTS).unwrap();
                assert_eq!(
                    voxel_set(&decoded),
                    expected,
                    "pair {position:?}/{sample:?}"
                );
                assert_eq!(decoded.bounds, bounds, "pair {position:?}/{sample:?}");
            }
        }
    }

    #[test]
    fn round_trips_empty_object() {
        let object = VoxjCodecObject {
            name: "o".to_owned(),
            palette_refs: Vec::new(),
            bounds: [0, 0, 0],
            positions: Vec::new(),
            samples: Vec::new(),
        };
        let encoded = encode_object(
            &object,
            &[],
            PositionEncoding::RawJson,
            SampleEncoding::RawJson,
        );
        let decoded = decode_object(&encoded, &[]).unwrap();
        assert!(decoded.positions.is_empty());
        assert!(decoded.samples.is_empty());
    }

    #[test]
    fn round_trips_zero_palette_object() {
        for sample in SAMPLES {
            let object = VoxjCodecObject {
                name: "o".to_owned(),
                palette_refs: Vec::new(),
                bounds: [2, 1, 1],
                positions: vec![[0, 0, 0], [1, 0, 0]],
                samples: vec![Vec::new(), Vec::new()],
            };
            let encoded = encode_object(&object, &[], PositionEncoding::BitmapBase64, sample);
            let decoded = decode_object(&encoded, &[]).unwrap();
            assert_eq!(decoded.positions.len(), 2, "sample {sample:?}");
            assert!(
                decoded.samples.iter().all(Vec::is_empty),
                "sample {sample:?}"
            );
        }
    }
}
