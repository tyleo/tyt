use crate::MagicaVoxelExt;
use serde::{Deserialize, Serialize};

/// The envelope that namespaces the MagicaVoxel provenance under the
/// `magica-voxel` key of a [`VoxState`](voxcore::VoxState) ext.
#[derive(Deserialize, Serialize)]
pub(crate) struct MagicaVoxelExtWrapper {
    #[serde(rename = "magica-voxel")]
    pub magica_voxel: MagicaVoxelExt,
}
