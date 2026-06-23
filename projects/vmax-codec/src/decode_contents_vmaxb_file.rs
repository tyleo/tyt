use crate::decode_vmax_snapshots;
use std::io;
use vmax::{VMaxCodecContentsVmaxbFile, VMaxSerdeContentsVmaxbFile};

/// Decodes a `VMaxSerdeContentsVmaxbFile` into a
/// [`VMaxCodecContentsVmaxbFile`]: replays the snapshots into their final
/// voxels and carries the editor state through unchanged. The inverse of
/// [`encode_contents_vmaxb_file`](crate::encode_contents_vmaxb_file).
pub fn decode_contents_vmaxb_file(
    contents: &VMaxSerdeContentsVmaxbFile,
) -> io::Result<VMaxCodecContentsVmaxbFile> {
    Ok(VMaxCodecContentsVmaxbFile {
        voxels: decode_vmax_snapshots(&contents.snapshots)?,
        uuid: contents.uuid.clone(),
        v: contents.v,
        tools: contents.tools.clone(),
        brush: contents.brush.clone(),
        cam: contents.cam.clone(),
    })
}
