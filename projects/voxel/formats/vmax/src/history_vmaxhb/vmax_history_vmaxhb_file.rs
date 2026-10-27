use crate::VMaxHistorySession;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Decoded `*.vmaxhb` undo-history file. Written per object (named by the
/// object's `hist` reference, e.g. `history1.vmaxhb`) and once for the working
/// scene (`scene.vmaxhb`). The envelope is an LZFSE-framed (`bvx2`) binary
/// plist.
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[cfg_attr(feature = "serde", serde(deny_unknown_fields))]
pub struct VMaxHistoryVmaxhbFile {
    /// The edit sessions, in order.
    pub sessions: Vec<VMaxHistorySession>,

    /// Active session id.
    pub asid: i64,
}
