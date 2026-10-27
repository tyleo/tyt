use crate::utilities::QuantizePoint;
use branded_id::U32Id;
use std::collections::HashMap;
use voxcore::BVoxMaterial;

/// A palette's chosen representatives over one set of layers.
#[derive(Clone, Debug)]
pub struct QuantizePlan {
    /// Each sampled material's representative, itself for a representative.
    pub(crate) representative_ids: HashMap<U32Id<BVoxMaterial>, U32Id<BVoxMaterial>>,

    /// Each sampled material's clustering point and partition index, which a
    /// dithered snap starts from.
    pub(crate) points: HashMap<U32Id<BVoxMaterial>, (QuantizePoint, usize)>,

    /// Each partition's representatives, the only targets a dithered snap of
    /// that partition's materials picks among.
    pub(crate) partition_representatives: Vec<Vec<QuantizePoint>>,

    /// How many leading axes the points use; the rest stay zero.
    pub(crate) dimensions: usize,
}
