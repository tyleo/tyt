use crate::authored_box;
use vmax::VMaxObject;

/// Identifies scene objects that place the same geometry more than once. Voxel
/// Max instances a model by reusing a `contents*.vmaxb` and its palette across
/// objects, so objects sharing the `data`/`palette` filenames and the same
/// authored box decode to one identical object.
#[derive(Clone, Eq, Hash, PartialEq)]
pub(crate) struct InstanceKey {
    /// The `contents*.vmaxb` filename.
    pub(crate) data: String,

    /// The `palette*.png` filename.
    pub(crate) palette: String,

    /// The authored box's min corner.
    pub(crate) box_min: [i32; 3],

    /// The authored box's size.
    pub(crate) size: [u32; 3],
}

impl InstanceKey {
    /// The key for `object`, or `None` when it cannot be instanced.
    pub(crate) fn of(object: &VMaxObject) -> Option<Self> {
        if object.data.is_empty() {
            return None;
        }
        let (box_min, size) = authored_box(object)?;
        Some(InstanceKey {
            data: object.data.clone(),
            palette: object.palette.clone(),
            box_min,
            size,
        })
    }
}
