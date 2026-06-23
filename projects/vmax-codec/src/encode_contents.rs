use crate::encode_snapshots;
use vmax::{VMaxCodecContentsVmaxbFile, VMaxSerdeContentsVmaxbFile};

/// Encodes a [`VMaxCodecContentsVmaxbFile`] into a `VMaxSerdeContentsVmaxbFile`, the inverse of
/// [`decode_contents`](crate::decode_contents): re-encodes the voxels into one
/// checkpoint snapshot per occupied chunk (via
/// [`encode_snapshots`](crate::encode_snapshots)) and carries the editor state
/// through unchanged.
pub fn encode_contents(contents: &VMaxCodecContentsVmaxbFile) -> VMaxSerdeContentsVmaxbFile {
    VMaxSerdeContentsVmaxbFile {
        snapshots: encode_snapshots(&contents.voxels),
        uuid: contents.uuid.clone(),
        v: contents.v,
        tools: contents.tools.clone(),
        brush: contents.brush.clone(),
        cam: contents.cam.clone(),
    }
}
