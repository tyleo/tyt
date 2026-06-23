use crate::VMaxCodecMaterial;

/// The decoded form of a `palette*.settings.vmaxpsb`: the decoded material slots
/// and unpacked RGBA color table plus the palette-level editor state Voxel Max
/// records. The codec backend's palette representation (see
/// [`VMaxCodecBackend`](crate::VMaxCodecBackend)); `vmax-codec` decodes a
/// [`VMaxSerdePaletteSettingsVmaxpsbFile`](crate::VMaxSerdePaletteSettingsVmaxpsbFile) into
/// this and encodes it back, round-tripping every field. The one value not
/// carried is each material's `mi` slot token: it is reconstructed from the
/// material's 1-based slot position on re-encode, the convention Voxel Max
/// writes (`"1".."8"` in slot order).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct VMaxCodecPaletteSettingsVmaxpsbFile {
    /// Palette display name.
    pub name: String,

    /// Decoded material slots.
    pub materials: Vec<VMaxCodecMaterial>,

    /// One `[r, g, b, a]` per color index, unpacked from the packed table. This
    /// is Voxel Max's color source when an object's `palette*.png` is absent;
    /// empty when the palette carries no color table.
    pub colors: Vec<[u8; 4]>,

    /// Color indices.
    pub indices: Vec<i64>,

    /// Layer-color usage mask.
    pub lc: Vec<u8>,

    /// Palette type tag.
    pub palette_type: i64,

    /// Global transparency.
    pub transparency: f64,

    /// Reserved counter.
    pub r: i64,

    /// Reserved token.
    pub rt: String,

    /// Comment string.
    pub cmt: String,

    /// Currently selected index.
    pub current: i64,

    /// Alias token.
    pub ali: String,
}
