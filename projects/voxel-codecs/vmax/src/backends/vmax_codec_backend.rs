use crate::{VMaxBackend, VMaxCodecContentsVmaxbFile, VMaxCodecPaletteSettingsVmaxpsbFile};

/// The codec [`VMaxBackend`]: payloads are the decoded forms, produced from
/// the serde form and encoded back to it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VMaxCodecBackend;

impl VMaxBackend for VMaxCodecBackend {
    type Contents = VMaxCodecContentsVmaxbFile;
    type PaletteSettings = VMaxCodecPaletteSettingsVmaxpsbFile;
}
