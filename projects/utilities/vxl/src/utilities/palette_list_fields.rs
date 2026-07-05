/// Which per-palette fields `palette list` renders beside the always-shown
/// index. Each is a settable boolean defaulting to shown, so a bare
/// `palette list` prints every field and `--show-* false` drops a column or
/// branch. The fields render in this order in every layout.
#[derive(Clone, Copy, Debug)]
pub struct PaletteListFields {
    /// The ordered attribute keys.
    pub attributes: bool,
    /// The material count.
    pub materials: bool,
    /// The referencing objects.
    pub objects: bool,
}
