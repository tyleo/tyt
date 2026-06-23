use crate::{VMaxCodecBackend, VMaxFile};

/// A [`VMaxFile`] holding decoded payloads
/// ([`VMaxCodecContentsVmaxbFile`](crate::VMaxCodecContentsVmaxbFile) /
/// [`VMaxCodecPaletteSettingsVmaxpsbFile`](crate::VMaxCodecPaletteSettingsVmaxpsbFile)); the codec's
/// top-level type, encoded into a [`VMaxSerdeFile`](crate::VMaxSerdeFile) for
/// writing.
pub type VMaxCodecFile = VMaxFile<VMaxCodecBackend>;
