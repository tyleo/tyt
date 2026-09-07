/// Where a mesh's baked resources go: embedded in the mesh itself, in loose
/// files beside it, or both.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceStorage {
    /// Packed into the mesh: a GLB binary chunk or a `.gltf` data URI.
    Embedded,

    /// Written as loose files beside the mesh, which references them by
    /// relative path.
    External,

    /// Both: the mesh references its embedded copy and the loose files are
    /// working copies.
    Both,
}
