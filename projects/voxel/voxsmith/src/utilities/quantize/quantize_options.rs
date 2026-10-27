use crate::utilities::{
    AlphaMode, ColorSpace, Dither, PartitionProperties, PropertyInterpretation, ReductionMethod,
};
use std::num::NonZeroUsize;

/// How quantizing reduces the materials a set of layers samples.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QuantizeOptions {
    /// The most materials the layers may sample afterward.
    pub max_materials: NonZeroUsize,

    /// The property clustering runs on.
    pub property: String,

    /// How the property's values read as points.
    pub interpret_property: PropertyInterpretation,

    /// How a 4-component color's alpha takes part, or `None` for
    /// [`AlphaMode::Partition`]. Any other reading errors on `Some`.
    pub alpha: Option<AlphaMode>,

    /// The properties materials have to agree on to merge.
    pub partition: PartitionProperties,

    /// The clustering algorithm.
    pub method: ReductionMethod,

    /// The space a color reading measures distance in, or `None` for
    /// [`ColorSpace::Oklab`]. A numeric reading errors on `Some`.
    pub space: Option<ColorSpace>,

    /// Error diffusion applied when snapping samples to representatives.
    pub dither: Dither,
}
