use crate::encode_vmax_snapshots;
use vmax::{VMaxCodecContentsVmaxbFile, VMaxSerdeContentsVmaxbFile};

/// Encodes a [`VMaxCodecContentsVmaxbFile`] into a
/// `VMaxSerdeContentsVmaxbFile`: re-encodes the voxels into checkpoint
/// snapshots and carries the editor state through unchanged. The inverse of
/// [`decode_contents_vmaxb_file`](crate::decode_contents_vmaxb_file).
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
