#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The viewport camera recorded on a Voxel Max history step (`vc` / `fvc`). A
/// different, more compact shape than the per-object [`VMaxCamera`](crate::VMaxCamera):
/// four `[x, y, z]` vectors plus an orthographic flag. `vc` is the step's
/// camera; `fvc` is its focused/framed variant.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct VMaxViewCamera {
    /// Camera angles.
    pub ca: [f64; 3],

    /// Camera position.
    pub cp: [f64; 3],

    /// Camera origin / target.
    pub co: [f64; 3],

    /// Light angles.
    pub la: [f64; 3],

    /// Orthographic flag.
    pub o: bool,
}
