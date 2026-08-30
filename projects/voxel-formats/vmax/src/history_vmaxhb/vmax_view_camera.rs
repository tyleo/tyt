#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The viewport camera on a history step (`vc` / `fvc`). A more compact shape
/// than the per-object [`VMaxCamera`](crate::VMaxCamera).
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
