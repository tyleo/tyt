/// The editable cube each voxel object writes in a Voxel Max package.
/// `Auto` keeps a loaded object's supported extent when it fits its canvas;
/// a new object takes 256 or 512. A fixed size compacts empty canvas margins
/// and refuses any live geometry that cannot fit.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum VMaxObjectSize {
    /// Keep a loaded extent when possible; otherwise choose 256 or 512.
    #[default]
    Auto,

    /// A cube 32 voxels on a side.
    Size32,

    /// A cube 64 voxels on a side.
    Size64,

    /// A cube 128 voxels on a side.
    Size128,

    /// A cube 256 voxels on a side.
    Size256,

    /// A cube 512 voxels on a side.
    Size512,
}

impl VMaxObjectSize {
    /// The requested cube width, or `None` for automatic selection.
    pub fn dimension(self) -> Option<u32> {
        match self {
            Self::Auto => None,
            Self::Size32 => Some(32),
            Self::Size64 => Some(64),
            Self::Size128 => Some(128),
            Self::Size256 => Some(256),
            Self::Size512 => Some(512),
        }
    }
}
