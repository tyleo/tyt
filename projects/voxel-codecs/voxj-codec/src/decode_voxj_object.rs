use crate::{
    Error, MAX_HILBERT_BITS, Result, SampleChannels, VoxjDecodedObject, decode_hilbert,
    decode_varint, hilbert_bits, packed_width, unpack_bits,
};
use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use std::iter;
use voxj::{VoxjObject, VoxjPositionBlock, VoxjSampleBlock};

/// Decodes one [`VoxjObject`] into a [`VoxjDecodedObject`], the inverse of
/// [`encode_voxj_object`](crate::encode_voxj_object()). `material_counts`
/// comes from
/// [`voxj_palette_material_counts`](crate::voxj_palette_material_counts()); a
/// layer is sampled iff its count is above zero, and the sample block must
/// carry one channel per sampled layer.
///
/// Each returned `positions[k]` pairs with `samples[k]`.
pub fn decode_voxj_object(
    object: &VoxjObject,
    material_counts: &[usize],
) -> Result<VoxjDecodedObject> {
    let layout = SampleChannels::from_material_counts(material_counts);
    let positions = decode_positions(&object.voxel_positions, object.bounds)?;
    let channels = decode_samples(&object.voxel_samples, layout.counts(), positions.len())?;
    let samples = (0..positions.len())
        .map(|k| channels.iter().map(|channel| channel[k]).collect())
        .collect();
    Ok(VoxjDecodedObject {
        name: object.name.clone(),
        layers: object.layers.clone(),
        bounds: object.bounds,
        origin: object.origin,
        positions,
        samples,
    })
}

/// Wraps a message describing malformed input as invalid data.
fn invalid_data(message: String) -> Error {
    Error::Invalid(message)
}

/// A bit-packed base64 block, either a `bitmap-base64` position bitmap or a
/// `packed-base64` sample channel, decoded to exactly the bytes its bit count
/// needs, with the final byte's unused low bits zero (spec rules 11.3, 13.2).
/// `label` names the block for the error message.
fn check_packed_bytes(bytes: &[u8], used_bits: usize, label: &str) -> Result<()> {
    let required = used_bits.div_ceil(8);
    if bytes.len() != required {
        return Err(invalid_data(format!(
            "{label} decodes to {} bytes, need exactly {required}",
            bytes.len()
        )));
    }
    // The packing is MSB-first, so the only unused bits are the final byte's
    // low `pad` bits; the exact length rules out any other padding.
    let pad = required * 8 - used_bits;
    if pad != 0 && bytes[required - 1] & ((1u8 << pad) - 1) != 0 {
        return Err(invalid_data(format!(
            "{label} has non-zero padding bits in its final byte"
        )));
    }
    Ok(())
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
fn decode_positions(block: &VoxjPositionBlock, bounds: [u32; 3]) -> Result<Vec<[u32; 3]>> {
    Ok(match block {
        VoxjPositionBlock::RawJson(positions) => positions.clone(),

        VoxjPositionBlock::BitmapBase64(base64) => {
            let cells = bounds[0] as usize * bounds[1] as usize * bounds[2] as usize;
            let bytes = BASE64.decode(base64).map_err(Error::Base64)?;
            check_packed_bytes(&bytes, cells, "bitmap position block")?;
            unpack_bits(&bytes, 1, cells)
                .iter()
                .enumerate()
                .filter(|&(_, &bit)| bit == 1)
                .map(|(cell, _)| cell_to_position(cell as u64, bounds))
                .collect()
        }

        VoxjPositionBlock::HilbertDeltaVarintBase64(base64) => {
            let bits = hilbert_bits(bounds);
            if bits > MAX_HILBERT_BITS {
                return Err(invalid_data(format!(
                    "hilbert position block needs {bits} bits per axis, over the \
                     {MAX_HILBERT_BITS} limit; some bounds dimension of {bounds:?} \
                     exceeds 131072"
                )));
            }
            let mut index = 0u64;
            decode_varint(&BASE64.decode(base64).map_err(Error::Base64)?)?
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
/// sampled layer, in the position block's voxel order. `channel_counts` holds
/// one material count per sampled layer.
fn decode_samples(
    block: &VoxjSampleBlock,
    channel_counts: &[usize],
    n: usize,
) -> Result<Vec<Vec<u32>>> {
    let channels: Vec<Vec<u32>> = match block {
        VoxjSampleBlock::RawJson(channels) => channels.clone(),

        VoxjSampleBlock::RleJson(channels) => channels
            .iter()
            .map(|channel| rle_decode(channel))
            .collect::<Result<Vec<_>>>()?,

        VoxjSampleBlock::PackedBase64(channels) => channels
            .iter()
            .enumerate()
            .map(|(c, base64)| {
                // Width by `get` with a fallback rather than direct indexing,
                // unlike the encoder: hostile input may carry more channels
                // than sampled layers, and the arity check below runs only
                // after each channel decodes.
                let width = packed_width(channel_counts.get(c).copied().unwrap_or(1));
                let bytes = BASE64.decode(base64).map_err(Error::Base64)?;
                check_packed_bytes(
                    &bytes,
                    n * width as usize,
                    &format!("packed sample channel {c}"),
                )?;
                Ok(unpack_bits(&bytes, width, n))
            })
            .collect::<Result<Vec<_>>>()?,
    };

    // Every encoding must yield one channel per sampled layer, each holding a
    // value for every voxel; otherwise the object's samples are malformed.
    if channels.len() != channel_counts.len() {
        return Err(invalid_data(format!(
            "sample block has {} channels, expected {} (one per sampled layer)",
            channels.len(),
            channel_counts.len()
        )));
    }
    if let Some(channel) = channels.iter().find(|channel| channel.len() != n) {
        return Err(invalid_data(format!(
            "sample channel has {} values, expected {n}",
            channel.len()
        )));
    }
    Ok(channels)
}

/// Expands flat run-length encoding `[value, count, value, count, ...]`. The
/// stream must be even-length with every count positive (spec rule 11.2); an
/// odd tail leaves a value with no count and a zero count is not a run.
fn rle_decode(rle: &[u32]) -> Result<Vec<u32>> {
    if !rle.len().is_multiple_of(2) {
        return Err(invalid_data(format!(
            "rle sample channel has odd length {}, so a value has no count",
            rle.len()
        )));
    }
    let mut out = Vec::new();
    for pair in rle.chunks_exact(2) {
        let (value, count) = (pair[0], pair[1]);
        if count == 0 {
            return Err(invalid_data(format!(
                "rle sample channel has a zero count for value {value}"
            )));
        }
        out.extend(iter::repeat_n(value, count as usize));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use crate::{
        PositionEncoding, SampleEncoding, VoxjDecodedObject, decode_voxj_object, encode_voxj_object,
    };
    use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
    use std::collections::BTreeSet;
    use voxj::{VoxjObject, VoxjPositionBlock, VoxjSampleBlock};

    /// A one-layer object over the given bounds, position, and sample blocks.
    fn object(
        bounds: [u32; 3],
        voxel_positions: VoxjPositionBlock,
        voxel_samples: VoxjSampleBlock,
    ) -> VoxjObject {
        VoxjObject {
            name: "o".to_owned(),
            layers: vec![0],
            bounds,
            origin: [0, 0, 0],
            voxel_positions,
            voxel_samples,
        }
    }

    /// The two-voxel raw-json positions `(0,0,0)` and `(1,0,0)`, used to pair a
    /// valid position block with a malformed sample block.
    fn two_raw_positions() -> VoxjPositionBlock {
        VoxjPositionBlock::RawJson(vec![[0, 0, 0], [1, 0, 0]])
    }

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

    /// Material counts of the two layers' palettes in `sample_object`; both
    /// layers are sampled.
    const MATERIAL_COUNTS: [usize; 2] = [256, 8];

    fn sample_object() -> VoxjDecodedObject {
        VoxjDecodedObject {
            name: "o".to_owned(),
            layers: vec![0, 1],
            bounds: [4, 4, 5],
            origin: [0, 0, 0],
            positions: vec![[0, 0, 0], [2, 1, 0], [1, 3, 4], [3, 3, 3]],
            samples: vec![vec![1, 0], vec![5, 2], vec![200, 7], vec![0, 1]],
        }
    }

    /// The set of `(position, samples)` pairs, order-independent, so it also
    /// proves positions and samples stay aligned through any reordering.
    fn voxel_set(object: &VoxjDecodedObject) -> BTreeSet<([u32; 3], Vec<u32>)> {
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
                let encoded =
                    encode_voxj_object(&object, &MATERIAL_COUNTS, position, sample).unwrap();
                let decoded = decode_voxj_object(&encoded, &MATERIAL_COUNTS).unwrap();
                assert_eq!(
                    voxel_set(&decoded),
                    expected,
                    "pair {position:?}/{sample:?}"
                );
                assert_eq!(decoded.bounds, bounds, "pair {position:?}/{sample:?}");
            }
        }
    }

    /// A layer over a palette with no materials is unsampled and carries no
    /// channel: with counts `[256, 0, 8]` the middle layer contributes
    /// nothing, and the two-entry sample rows round-trip unchanged.
    #[test]
    fn round_trips_object_with_unsampled_layer() {
        let counts = [256, 0, 8];
        let mut object = sample_object();
        object.layers = vec![0, 2, 1];
        for position in POSITIONS {
            for sample in SAMPLES {
                let encoded = encode_voxj_object(&object, &counts, position, sample).unwrap();
                let decoded = decode_voxj_object(&encoded, &counts).unwrap();
                assert_eq!(
                    voxel_set(&decoded),
                    voxel_set(&object),
                    "pair {position:?}/{sample:?}"
                );
                assert_eq!(
                    decoded.layers, object.layers,
                    "pair {position:?}/{sample:?}"
                );
            }
        }
    }

    #[test]
    fn round_trips_empty_object() {
        let object = VoxjDecodedObject {
            name: "o".to_owned(),
            layers: Vec::new(),
            bounds: [0, 0, 0],
            origin: [0, 0, 0],
            positions: Vec::new(),
            samples: Vec::new(),
        };
        let encoded = encode_voxj_object(
            &object,
            &[],
            PositionEncoding::RawJson,
            SampleEncoding::RawJson,
        )
        .unwrap();
        let decoded = decode_voxj_object(&encoded, &[]).unwrap();
        assert!(decoded.positions.is_empty());
        assert!(decoded.samples.is_empty());
    }

    #[test]
    fn round_trips_zero_layer_object() {
        for sample in SAMPLES {
            let object = VoxjDecodedObject {
                name: "o".to_owned(),
                layers: Vec::new(),
                bounds: [2, 1, 1],
                origin: [0, 0, 0],
                positions: vec![[0, 0, 0], [1, 0, 0]],
                samples: vec![Vec::new(), Vec::new()],
            };
            let encoded =
                encode_voxj_object(&object, &[], PositionEncoding::BitmapBase64, sample).unwrap();
            let decoded = decode_voxj_object(&encoded, &[]).unwrap();
            assert_eq!(decoded.positions.len(), 2, "sample {sample:?}");
            assert!(
                decoded.samples.iter().all(Vec::is_empty),
                "sample {sample:?}"
            );
        }
    }

    /// A channel supplied for an unsampled layer is an arity fault: the layer's
    /// palette has no materials, so no channel may carry samples for it.
    #[test]
    fn rejects_channel_for_unsampled_layer() {
        let object = object(
            [2, 1, 1],
            two_raw_positions(),
            VoxjSampleBlock::RawJson(vec![vec![0, 0]]),
        );
        assert!(decode_voxj_object(&object, &[0]).is_err());
    }

    /// A raw-json sample channel shorter than the voxel count (here one value
    /// for two voxels) is malformed, not silently zero-filled.
    #[test]
    fn rejects_short_raw_json_sample_channel() {
        let object = object(
            [2, 1, 1],
            two_raw_positions(),
            VoxjSampleBlock::RawJson(vec![vec![1]]),
        );
        assert!(decode_voxj_object(&object, &[4]).is_err());
    }

    /// A packed channel with fewer bytes than the voxel count and bit width
    /// require is a truncated block, not zero-padded samples.
    #[test]
    fn rejects_truncated_packed_samples() {
        let object = object(
            [2, 1, 1],
            two_raw_positions(),
            VoxjSampleBlock::PackedBase64(vec![String::new()]),
        );
        assert!(decode_voxj_object(&object, &[4]).is_err());
    }

    /// A sample block carrying more channels than the object has sampled layers
    /// is rejected rather than packing the extra channel at a guessed width.
    #[test]
    fn rejects_channel_count_mismatch() {
        let object = object(
            [1, 1, 1],
            VoxjPositionBlock::RawJson(vec![[0, 0, 0]]),
            VoxjSampleBlock::RleJson(vec![vec![0, 1], vec![0, 1]]),
        );
        assert!(decode_voxj_object(&object, &[4]).is_err());
    }

    /// A bitmap over two cells needs exactly one byte; a second byte makes the
    /// block longer than its bounds allow (spec rule 13.2).
    #[test]
    fn rejects_bitmap_with_extra_bytes() {
        let bitmap = VoxjPositionBlock::BitmapBase64(BASE64.encode([0xC0, 0x00]));
        let object = object(
            [2, 1, 1],
            bitmap,
            VoxjSampleBlock::RawJson(vec![vec![0, 0]]),
        );
        assert!(decode_voxj_object(&object, &[4]).is_err());
    }

    /// The bitmap's two cells fill the top two bits; a set bit among the final
    /// byte's six pad bits is malformed (spec rule 13.2).
    #[test]
    fn rejects_bitmap_with_nonzero_pad_bits() {
        let bitmap = VoxjPositionBlock::BitmapBase64(BASE64.encode([0xC1]));
        let object = object(
            [2, 1, 1],
            bitmap,
            VoxjSampleBlock::RawJson(vec![vec![0, 0]]),
        );
        assert!(decode_voxj_object(&object, &[4]).is_err());
    }

    /// A packed channel for two width-2 values needs exactly one byte; a second
    /// byte is too long (spec rule 11.3).
    #[test]
    fn rejects_packed_with_extra_bytes() {
        let packed = VoxjSampleBlock::PackedBase64(vec![BASE64.encode([0x70, 0x00])]);
        let object = object([2, 1, 1], two_raw_positions(), packed);
        assert!(decode_voxj_object(&object, &[4]).is_err());
    }

    /// Two width-2 values fill the top four bits; a set bit among the final
    /// byte's four pad bits is malformed (spec rule 11.3).
    #[test]
    fn rejects_packed_with_nonzero_pad_bits() {
        let packed = VoxjSampleBlock::PackedBase64(vec![BASE64.encode([0x71])]);
        let object = object([2, 1, 1], two_raw_positions(), packed);
        assert!(decode_voxj_object(&object, &[4]).is_err());
    }

    /// An odd-length run stream leaves a trailing value with no count (spec rule
    /// 11.2).
    #[test]
    fn rejects_odd_length_rle() {
        let rle = VoxjSampleBlock::RleJson(vec![vec![1, 2, 3]]);
        let object = object([2, 1, 1], two_raw_positions(), rle);
        assert!(decode_voxj_object(&object, &[4]).is_err());
    }

    /// A zero count is not a run and must be rejected (spec rule 11.2).
    #[test]
    fn rejects_zero_count_rle() {
        let rle = VoxjSampleBlock::RleJson(vec![vec![1, 0, 3, 2]]);
        let object = object([2, 1, 1], two_raw_positions(), rle);
        assert!(decode_voxj_object(&object, &[4]).is_err());
    }

    /// A grid whose largest axis exceeds 131072 needs more than 17 Hilbert bits,
    /// which the format forbids for this encoding (spec rule 13.3.2).
    #[test]
    fn rejects_oversized_hilbert_bounds() {
        let hilbert = VoxjPositionBlock::HilbertDeltaVarintBase64(String::new());
        let object = object(
            [131073, 1, 1],
            hilbert,
            VoxjSampleBlock::RawJson(vec![Vec::new()]),
        );
        assert!(decode_voxj_object(&object, &[4]).is_err());
    }

    /// A single continuation byte ends the varint stream mid-value, so the
    /// hilbert block does not decode (spec rule 13.3.1).
    #[test]
    fn rejects_truncated_hilbert_varint() {
        let hilbert = VoxjPositionBlock::HilbertDeltaVarintBase64(BASE64.encode([0x80]));
        let object = object(
            [2, 1, 1],
            hilbert,
            VoxjSampleBlock::RawJson(vec![Vec::new()]),
        );
        assert!(decode_voxj_object(&object, &[4]).is_err());
    }
}
