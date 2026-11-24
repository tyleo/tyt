use crate::CellBox;

/// One of an object's tiles that hold a surface between materials that
/// pass light.
pub struct GlassTile {
    /// The box around the tile's cells with a surface, where a walk can
    /// end.
    pub surfaces: CellBox,

    /// Where the tile's words start among its object's.
    pub offset: u32,

    /// The indices of the tile's cells with a surface, in order.
    pub cells: Vec<u16>,
}
