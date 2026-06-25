/// The procedural shape backing a shape [`GoxlLayer`](crate::GoxlLayer), from
/// the layer's `shape` key. Goxel fills such a layer from the shape rather than
/// from stored blocks.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum GoxlShape {
    /// `"sphere"`.
    #[default]
    Sphere,

    /// `"cube"`.
    Cube,

    /// `"cylinder"`.
    Cylinder,
}
