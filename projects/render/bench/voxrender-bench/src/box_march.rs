use crate::BrickMarch;

/// How a box raster's fragment finds its box's first hit.
#[derive(Clone, Copy, Debug)]
pub enum BoxMarch {
    /// A ray query's march over the object's [`Tree64`](crate::Tree64),
    /// along the pixel's ray.
    Tree(BrickMarch),

    /// A step a cell at a time through the tile's
    /// [`TileMasks`](crate::TileMasks), from the point the rasterizer gives
    /// the fragment on its box.
    Masks,
}

impl BoxMarch {
    /// The march's name in a report.
    pub fn label(self) -> String {
        match self {
            BoxMarch::Tree(march) => format!("{} march", march.label()),
            BoxMarch::Masks => "mask march".to_owned(),
        }
    }
}
