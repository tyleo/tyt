use vmax::{
    VMaxContentsVmaxbFile, VMaxHistoryVmaxhbFile, VMaxHistoryVmaxhvsbFile, VMaxHistoryVmaxhvscFile,
    VMaxPaletteSettingsVmaxpsbFile,
};

/// Serializes the typed files of a `.vmax` package to binary property lists.
/// Each method returns bare plist bytes; the caller adds any LZFSE framing.
pub trait EncodeVMaxPlist {
    /// The binary plist of a `contents*.vmaxb` object, or the reason it has
    /// none.
    fn encode_contents_vmaxb(&self, file: &VMaxContentsVmaxbFile) -> Result<Vec<u8>, String>;

    /// The binary plist of a `*.vmaxhb` undo history, or the reason it has
    /// none.
    fn encode_history_vmaxhb(&self, file: &VMaxHistoryVmaxhbFile) -> Result<Vec<u8>, String>;

    /// The binary plist of a `*.vmaxhvsb` snapshot buffer, or the reason it
    /// has none.
    fn encode_history_vmaxhvsb(&self, file: &VMaxHistoryVmaxhvsbFile) -> Result<Vec<u8>, String>;

    /// The binary plist of a `*.vmaxhvsc` snapshot sidecar, or the reason it
    /// has none.
    fn encode_history_vmaxhvsc(&self, file: &VMaxHistoryVmaxhvscFile) -> Result<Vec<u8>, String>;

    /// The binary plist of a `palette*.settings.vmaxpsb` palette, or the
    /// reason it has none.
    fn encode_palette_settings_vmaxpsb(
        &self,
        file: &VMaxPaletteSettingsVmaxpsbFile,
    ) -> Result<Vec<u8>, String>;
}
