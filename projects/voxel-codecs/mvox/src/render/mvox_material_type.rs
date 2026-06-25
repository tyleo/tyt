/// A material's shading model, the `_type` key of a `MATL` chunk.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MVoxMaterialType {
    /// `_diffuse`.
    Diffuse,

    /// `_metal`.
    Metal,

    /// `_glass`.
    Glass,

    /// `_emit`.
    Emit,

    /// A `_type` value this crate does not model, preserved verbatim.
    Other(String),
}
