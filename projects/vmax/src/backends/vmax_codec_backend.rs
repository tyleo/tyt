use crate::{VMaxBackend, VMaxCodecContentsVmaxbFile, VMaxCodecPaletteSettingsVmaxpsbFile};

/// The codec [`VMaxBackend`]: payloads are the decoded forms
/// ([`VMaxCodecContentsVmaxbFile`] / [`VMaxCodecPaletteSettingsVmaxpsbFile`]) the codec produces
/// from and re-encodes into the serde form.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VMaxCodecBackend;

impl VMaxBackend for VMaxCodecBackend {
    type Contents = VMaxCodecContentsVmaxbFile;
    type PaletteSettings = VMaxCodecPaletteSettingsVmaxpsbFile;
}
