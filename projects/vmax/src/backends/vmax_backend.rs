use std::fmt::Debug;

/// Selects the in-memory representation of a [`VMaxFile`](crate::VMaxFile)'s
/// per-object and per-palette payloads, so one package hierarchy serves both the
/// serde form (raw `.vmaxb` / `.vmaxpsb` parses) and the codec form (decoded
/// voxels and materials). The scene graph and `palette*.png` color tables are
/// the same in both forms, so they are not parameterized.
pub trait VMaxBackend {
    /// A `contents*.vmaxb` object:
    /// [`VMaxSerdeContentsVmaxbFile`](crate::VMaxSerdeContentsVmaxbFile) for the serde
    /// backend, [`VMaxCodecContentsVmaxbFile`](crate::VMaxCodecContentsVmaxbFile) for the codec
    /// backend.
    type Contents: Clone + Debug + PartialEq;

    /// A `palette*.settings.vmaxpsb` palette:
    /// [`VMaxSerdePaletteSettingsVmaxpsbFile`](crate::VMaxSerdePaletteSettingsVmaxpsbFile)
    /// for the serde backend,
    /// [`VMaxCodecPaletteSettingsVmaxpsbFile`](crate::VMaxCodecPaletteSettingsVmaxpsbFile) for the
    /// codec backend.
    type PaletteSettings: Clone + Debug + PartialEq;
}
