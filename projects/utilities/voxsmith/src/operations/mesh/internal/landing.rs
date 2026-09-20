/// Where a destination's expression lands.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Landing {
    /// A vertex attribute, read at the corners.
    Attribute,

    /// A JSON entry or an extras value, off the mesh.
    Json,

    /// A primitive select, read at the faces.
    Select,

    /// A texture on the atlas of the value's domain, or a plain factor.
    Texture,
}
