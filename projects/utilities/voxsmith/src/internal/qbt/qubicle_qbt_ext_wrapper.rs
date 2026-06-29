use crate::QubicleQbtExt;
use serde::{Deserialize, Serialize};

/// The envelope that namespaces the Qubicle `.qbt` provenance under the
/// `qubicle-qbt` key of a [`VoxMain`](voxcore::VoxMain) ext.
#[derive(Deserialize, Serialize)]
pub struct QubicleQbtExtWrapper {
    #[serde(rename = "qubicle-qbt")]
    pub qubicle_qbt: QubicleQbtExt,
}
