/// Where a destination's expression lands.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Landing {
    /// A vertex attribute, read at the corners.
    Attribute,

    /// A material factor, one plain value.
    Factor,

    /// A JSON entry or an extras value, off the mesh.
    Json,

    /// A primitive select, read at the faces.
    Select,

    /// A texture on the atlas of the domain it bakes at.
    Texture,
}
