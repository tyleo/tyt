use crate::decode_vmax_snapshots;
use vmax::{VMaxCodecContentsVmaxbFile, VMaxSerdeContentsVmaxbFile};

/// Decodes a `VMaxSerdeContentsVmaxbFile` into a [`VMaxCodecContentsVmaxbFile`]: replays the
/// snapshots into their final voxels (via
/// [`decode_vmax_snapshots`](crate::decode_vmax_snapshots)) and carries the editor state
/// through unchanged. The inverse of
/// [`encode_contents_vmaxb_file`](crate::encode_contents_vmaxb_file).
pub fn decode_contents_vmaxb_file(
    contents: &VMaxSerdeContentsVmaxbFile,
) -> VMaxCodecContentsVmaxbFile {
    VMaxCodecContentsVmaxbFile {
        voxels: decode_vmax_snapshots(&contents.snapshots),
        uuid: contents.uuid.clone(),
        v: contents.v,
        tools: contents.tools.clone(),
        brush: contents.brush.clone(),
        cam: contents.cam.clone(),
    }
}
