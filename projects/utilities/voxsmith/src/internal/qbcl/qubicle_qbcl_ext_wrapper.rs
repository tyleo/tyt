use crate::QubicleQbclExt;
use serde::{Deserialize, Serialize};

/// The envelope that namespaces the Qubicle `.qbcl` provenance under the
/// `qubicle-qbcl` key of a [`VoxState`](voxcore::VoxState) ext.
#[derive(Deserialize, Serialize)]
pub struct QubicleQbclExtWrapper {
    #[serde(rename = "qubicle-qbcl")]
    pub qubicle_qbcl: QubicleQbclExt,
}
