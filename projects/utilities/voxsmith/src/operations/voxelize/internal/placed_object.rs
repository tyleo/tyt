use std::ops::Range;

/// One placement of a mesh object by a hierarchy node, the unit that becomes
/// one voxel object.
pub(crate) struct PlacedObject {
    /// The mesh object's name, empty when it has none.
    pub object_name: String,

    /// The placing node's name, empty when it has none.
    pub node_name: String,

    /// The placement's triangles within the mesh's triangle list.
    pub range: Range<usize>,
}

impl PlacedObject {
    /// The voxel object's name: the mesh object's, else the placing node's,
    /// else `fallback`, else empty.
    pub fn name<'s>(&'s self, fallback: Option<&'s str>) -> &'s str {
        [
            self.object_name.as_str(),
            self.node_name.as_str(),
            fallback.unwrap_or_default(),
        ]
        .into_iter()
        .find(|name| !name.is_empty())
        .unwrap_or_default()
    }
}
