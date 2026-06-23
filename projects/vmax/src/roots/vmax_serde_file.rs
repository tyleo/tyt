use crate::{VMaxFile, VMaxSerdeBackend};

/// A [`VMaxFile`] holding raw parsed payloads
/// ([`VMaxSerdeContentsVmaxbFile`](crate::VMaxSerdeContentsVmaxbFile) /
/// [`VMaxSerdePaletteSettingsVmaxpsbFile`](crate::VMaxSerdePaletteSettingsVmaxpsbFile));
/// the form read from and written to a `.vmax` package on disk.
pub type VMaxSerdeFile = VMaxFile<VMaxSerdeBackend>;
