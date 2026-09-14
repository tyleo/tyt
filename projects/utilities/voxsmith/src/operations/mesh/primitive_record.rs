use crate::operations::mesh::{ArrayDomain, AttributeWrite};
use branded_id::U32Id;
use meshdoc::BMeshMaterial;

/// One primitive's elements.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrimitiveRecord {
    /// The material the primitive draws with; `None` is no material.
    pub material_id: Option<U32Id<BMeshMaterial>>,

    /// The select routing faces to the primitive.
    pub select: String,

    /// The primitive's name, which the glTF bridge writes as
    /// `extras.vxl.name`.
    pub name: Option<String>,

    /// Whether the primitive writes `NORMAL`.
    pub normal: bool,

    /// The declared stream list; when absent, the material's list applies.
    pub uv_streams: Option<Vec<ArrayDomain>>,

    /// The vertex attribute writes.
    pub attributes: Vec<AttributeWrite>,
}
