use crate::decode_snapshots;
use vmax::{VMaxCodecContentsVmaxbFile, VMaxSerdeContentsVmaxbFile};

/// Decodes a `VMaxSerdeContentsVmaxbFile` into a [`VMaxCodecContentsVmaxbFile`]: replays the
/// snapshots into their final voxels (via
/// [`decode_snapshots`](crate::decode_snapshots)) and carries the editor state
/// through unchanged. The inverse of
/// [`encode_contents`](crate::encode_contents).
pub fn decode_contents(contents: &VMaxSerdeContentsVmaxbFile) -> VMaxCodecContentsVmaxbFile {
    VMaxCodecContentsVmaxbFile {
        voxels: decode_snapshots(&contents.snapshots),
        uuid: contents.uuid.clone(),
        v: contents.v,
        tools: contents.tools.clone(),
        brush: contents.brush.clone(),
        cam: contents.cam.clone(),
    }
}
