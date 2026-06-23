use crate::encode_vmax_snapshots;
use vmax::{VMaxCodecContentsVmaxbFile, VMaxSerdeContentsVmaxbFile};

/// Encodes a [`VMaxCodecContentsVmaxbFile`] into a `VMaxSerdeContentsVmaxbFile`, the inverse of
/// [`decode_contents_vmaxb_file`](crate::decode_contents_vmaxb_file): re-encodes the voxels into one
/// checkpoint snapshot per occupied chunk (via
/// [`encode_vmax_snapshots`](crate::encode_vmax_snapshots)) and carries the editor state
/// through unchanged.
pub fn encode_contents_vmaxb_file(
    contents: &VMaxCodecContentsVmaxbFile,
) -> VMaxSerdeContentsVmaxbFile {
    VMaxSerdeContentsVmaxbFile {
        snapshots: encode_vmax_snapshots(&contents.voxels),
        uuid: contents.uuid.clone(),
        v: contents.v,
        tools: contents.tools.clone(),
        brush: contents.brush.clone(),
        cam: contents.cam.clone(),
    }
}
