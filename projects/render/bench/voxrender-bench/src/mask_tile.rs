use ty_math::TyVector3U32;

/// One of an object's tiles that hold an opaque cell.
pub struct MaskTile {
    /// The tile's low corner in the object's cells.
    pub corner: TyVector3U32,

    /// Where the tile's words start among its object's.
    pub offset: u32,

    /// The indices of the tile's record cells, in order.
    pub recorded: Vec<u16>,

    /// Each record cell's material, as its place among the tile's material
    /// ids.
    pub entries: Vec<u8>,
}
