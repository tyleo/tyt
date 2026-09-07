/// Which per-palette fields
/// [`palette_list`](crate::operations::palette_list::palette_list()) renders
/// beside the always-shown index. The fields render in this order in every
/// layout.
#[derive(Clone, Copy, Debug)]
pub struct PaletteListFields {
    /// The ordered property keys.
    pub properties: bool,

    /// The material count.
    pub materials: bool,

    /// The referencing objects.
    pub objects: bool,
}
