/// When [`to_voxj_file`](crate::to_voxj_file) and
/// [`VoxjFileBuilder`](crate::VoxjFileBuilder) record each object's editor
/// build volume in the document's edit state. The build volume is the working
/// grid an object was authored in, which can be larger than the tight grid its
/// live voxels occupy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditStateMode {
    /// Record the edit state only when some object carries margin around its
    /// live voxels. An already-tight object recreates its build volume on load,
    /// so its entry would carry no information.
    Auto,

    /// Always record the edit state, even when every object is already tight.
    Always,

    /// Never record the edit state. Margin around an object's live voxels is
    /// lost and the object reloads as its tight runtime grid.
    Never,
}
