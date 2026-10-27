use crate::material::MaterialRange;

/// The `0..1` every component of a modeled color property lies in.
pub const COLOR_RANGE: MaterialRange = MaterialRange {
    min: 0.0,
    max: Some(1.0),
    admits_zero: false,
};
