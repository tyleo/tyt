use crate::{
    CostVoxjObject, EncodeBase64, VoxjObject,
    objects::{
        MAX_HILBERT_BITS, PositionEncoding, Result, SampleEncoding, VoxjDecodedObject,
        encode_voxj_object, hilbert_bits,
    },
};

/// Skip the dense bitmap candidate above this many cells to bound memory.
const MAX_BITMAP_CELLS: u64 = 8_000_000;

/// Encodes one [`VoxjDecodedObject`] into a [`VoxjObject`], pinning each `Some`
/// block and searching each `None` block for the pairing with the lowest
/// cost. Both `None` is the full search; both `Some` is a fixed encoding.
///
/// # Arguments
/// * `dependencies` - encodes the blocks and costs each candidate. A pinned
///   pair is never costed.
/// * `material_counts` - from
///   [`voxj_palette_material_counts`](crate::objects::voxj_palette_material_counts()).
pub fn encode_voxj_object_optimized<D: EncodeBase64 + CostVoxjObject>(
    dependencies: &D,
    object: &VoxjDecodedObject,
    material_counts: &[usize],
    position: Option<PositionEncoding>,
    sample: Option<SampleEncoding>,
) -> Result<VoxjObject> {
    if object.positions.is_empty() {
        // No voxels: every encoding emits the same raw blocks, so there is
        // nothing to search.
        return encode_voxj_object(
            dependencies,
            object,
            material_counts,
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
    let mut candidates = positions
        .into_iter()
        .flat_map(|position| samples.iter().map(move |&sample| (position, sample)))
        .map(|(position, sample)| {
            encode_voxj_object(dependencies, object, material_counts, position, sample)
        })
        .collect::<Result<Vec<_>>>()?;
    if candidates.len() == 1 {
        return Ok(candidates.remove(0));
    }
    Ok(candidates
        .into_iter()
        .min_by_key(|candidate| dependencies.cost_voxj_object(candidate))
        .expect("at least one candidate"))
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

#[cfg(all(test, feature = "impl"))]
mod tests {
    use crate::{
        CostVoxjObject, DependenciesImpl, EncodeBase64, VoxjObject, VoxjPositionBlock,
        VoxjSampleBlock,
        objects::{
            PositionEncoding, SampleEncoding, VoxjDecodedObject, encode_voxj_object_optimized,
        },
    };

    /// Base64 through the crate's implementation and a cost from the wrapped
    /// function.
    struct Costing<F>(F);

    impl<F> EncodeBase64 for Costing<F> {
        fn encode_base64(&self, bytes: &[u8]) -> String {
            DependenciesImpl.encode_base64(bytes)
        }
    }

    impl<F: Fn(&VoxjObject) -> usize> CostVoxjObject for Costing<F> {
        fn cost_voxj_object(&self, object: &VoxjObject) -> usize {
            (self.0)(object)
        }
    }

    fn object() -> VoxjDecodedObject {
        VoxjDecodedObject {
            name: "o".to_owned(),
            layers: vec![0],
            bounds: [2, 1, 1],
            origin: [0, 0, 0],
            positions: vec![[0, 0, 0], [1, 0, 0]],
            samples: vec![vec![1], vec![2]],
        }
    }

    /// A stand-in cost: the object's debug rendering length.
    fn debug_len(object: &VoxjObject) -> usize {
        format!("{object:?}").len()
    }

    fn is_bitmap(object: &VoxjObject) -> bool {
        matches!(object.voxel_positions, VoxjPositionBlock::BitmapBase64(_))
    }

    fn is_hilbert(object: &VoxjObject) -> bool {
        matches!(
            object.voxel_positions,
            VoxjPositionBlock::HilbertDeltaVarintBase64(_)
        )
    }

    /// The full both-`None` search keeps an object's sample arity even when it
    /// has voxels but zero layers, since there are no channels to carry.
    #[test]
    fn zero_layer_object_keeps_sample_arity() {
        let object = encode_voxj_object_optimized(
            &Costing(debug_len),
            &VoxjDecodedObject {
                name: "o".to_owned(),
                layers: Vec::new(),
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
        let pinned_position = encode_voxj_object_optimized(
            &Costing(debug_len),
            &object(),
            &[4],
            Some(PositionEncoding::RawJson),
            None,
        )
        .unwrap();
        assert!(matches!(
            pinned_position.voxel_positions,
            VoxjPositionBlock::RawJson(_)
        ));

        let pinned_sample = encode_voxj_object_optimized(
            &Costing(debug_len),
            &object(),
            &[4],
            None,
            Some(SampleEncoding::RawJson),
        )
        .unwrap();
        assert!(matches!(
            pinned_sample.voxel_samples,
            VoxjSampleBlock::RawJson(_)
        ));
    }

    #[test]
    fn the_cost_picks_the_pairing() {
        let favoring = |favored: fn(&VoxjObject) -> bool| {
            Costing(move |object: &VoxjObject| usize::from(!favored(object)))
        };

        let picked =
            encode_voxj_object_optimized(&favoring(is_hilbert), &object(), &[4], None, None)
                .unwrap();
        assert!(is_hilbert(&picked));

        let picked =
            encode_voxj_object_optimized(&favoring(is_bitmap), &object(), &[4], None, None)
                .unwrap();
        assert!(is_bitmap(&picked));
    }

    #[test]
    fn a_pinned_pair_skips_the_cost() {
        let never =
            Costing(|_: &VoxjObject| -> usize { panic!("a pinned pair has nothing to rank") });
        let object = encode_voxj_object_optimized(
            &never,
            &object(),
            &[4],
            Some(PositionEncoding::Hilbert),
            Some(SampleEncoding::RleJson),
        )
        .unwrap();
        assert!(is_hilbert(&object));
    }
}
