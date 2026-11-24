use bytemuck::{Pod, Zeroable};

/// A box's three `vec4<u32>`s, as `box_view.wgsl`'s `boxes` and its vertex
/// stage take them.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct BoxWords {
    /// The box's low cell in its placement's grid.
    pub low: [u32; 3],

    pub placement: u32,

    /// The corner past the box's high cell.
    pub high: [u32; 3],

    /// Where the words of the box's tile start in its page's.
    pub tile: u32,

    /// A marched box's first crosser and its count of them, or the box
    /// around a glass box's surfaces.
    pub more: [u32; 2],

    /// The place of the tile's first material among the view's.
    pub material: u32,

    /// The first of the tile's lit words in its page's.
    pub lit: u32,
}
