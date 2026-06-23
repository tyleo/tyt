use crate::{VMaxBackend, VMaxSerdeContentsVmaxbFile, VMaxSerdePaletteSettingsVmaxpsbFile};

/// The serde [`VMaxBackend`]: payloads are the raw parsed files
/// ([`VMaxSerdeContentsVmaxbFile`] / [`VMaxSerdePaletteSettingsVmaxpsbFile`]), the form
/// read from and written to a `.vmax` package on disk.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VMaxSerdeBackend;

impl VMaxBackend for VMaxSerdeBackend {
    type Contents = VMaxSerdeContentsVmaxbFile;
    type PaletteSettings = VMaxSerdePaletteSettingsVmaxpsbFile;
}
