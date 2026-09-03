use vmax::{
    VMaxContentsVmaxbFile, VMaxHistoryVmaxhbFile, VMaxHistoryVmaxhvsbFile, VMaxHistoryVmaxhvscFile,
    VMaxPaletteSettingsVmaxpsbFile,
};

/// Parses the binary property lists of a `.vmax` package into their typed
/// files. Each method takes bare plist bytes; the caller strips any LZFSE
/// framing.
pub trait DecodeVMaxPlist {
    /// The `contents*.vmaxb` object `bytes` hold, or the reason they are not
    /// one.
    fn decode_contents_vmaxb(&self, bytes: &[u8]) -> Result<VMaxContentsVmaxbFile, String>;

    /// The `*.vmaxhb` undo history `bytes` hold, or the reason they are not
    /// one.
    fn decode_history_vmaxhb(&self, bytes: &[u8]) -> Result<VMaxHistoryVmaxhbFile, String>;

    /// The `*.vmaxhvsb` snapshot buffer `bytes` hold, or the reason they are
    /// not one.
    fn decode_history_vmaxhvsb(&self, bytes: &[u8]) -> Result<VMaxHistoryVmaxhvsbFile, String>;

    /// The `*.vmaxhvsc` snapshot sidecar `bytes` hold, or the reason they are
    /// not one.
    fn decode_history_vmaxhvsc(&self, bytes: &[u8]) -> Result<VMaxHistoryVmaxhvscFile, String>;

    /// The `palette*.settings.vmaxpsb` palette `bytes` hold, or the reason
    /// they are not one.
    fn decode_palette_settings_vmaxpsb(
        &self,
        bytes: &[u8],
    ) -> Result<VMaxPaletteSettingsVmaxpsbFile, String>;
}
