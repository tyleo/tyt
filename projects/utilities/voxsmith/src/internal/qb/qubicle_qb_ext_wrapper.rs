use crate::QubicleQbExt;
use serde::{Deserialize, Serialize};

/// The envelope that namespaces the Qubicle `.qb` provenance under the
/// `qubicle-qb` key of a [`VoxState`](voxcore::VoxState) ext.
#[derive(Deserialize, Serialize)]
pub struct QubicleQbExtWrapper {
    #[serde(rename = "qubicle-qb")]
    pub qubicle_qb: QubicleQbExt,
}
