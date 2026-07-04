use crate::ColorChannel;

/// One channel's value in a baked material map, the flag-agnostic source the
/// atlas bake reads. The caller resolves any custom-attribute binding and the
/// `smoothness` alias before building this, so `key` is always a concrete voxel
/// attribute key.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MaterialChannel {
    /// A material attribute by key, optionally one color `component` of it,
    /// inverted to `1 - value` when `invert` is set. A missing attribute falls
    /// back to its spec default.
    Attribute {
        /// The voxel attribute key read from the material.
        key: String,

        /// The color component to read, or `None` to read a scalar.
        component: Option<ColorChannel>,

        /// Whether to write `1 - value` instead of `value`.
        invert: bool,
    },

    /// The constant `0`.
    Zero,

    /// The constant `1`.
    One,

    /// Occlusion computed from the voxel geometry, which the palette atlas
    /// cannot hold; only an unwrap layout bakes it.
    ComputedOcclusion,
}
