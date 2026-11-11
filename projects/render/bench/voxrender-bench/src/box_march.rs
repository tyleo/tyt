use crate::{BoxDepth, BrickMarch};

/// How a box raster's fragment finds its box's first hit.
#[derive(Clone, Copy, Debug)]
pub enum BoxMarch {
    /// A ray query's march over the object's [`Tree64`](crate::Tree64),
    /// along the pixel's ray. Each fragment writes its hit's depth.
    Tree(BrickMarch),

    /// A step a cell at a time through the tile's
    /// [`TileMasks`](crate::TileMasks), from the point the rasterizer gives
    /// the fragment on its box.
    Masks {
        /// Whether a plain raster also draws each tile's solid boxes, whose
        /// fragments march nothing.
        solids: bool,

        /// The depth a marching fragment leaves.
        depth: BoxDepth,
    },
}

impl BoxMarch {
    /// Each march B1 checks and B2 times.
    pub const ALL: [BoxMarch; 6] = [
        BoxMarch::Tree(BrickMarch::Integer),
        BoxMarch::Tree(BrickMarch::Float),
        BoxMarch::Masks {
            solids: false,
            depth: BoxDepth::Hit,
        },
        BoxMarch::Masks {
            solids: false,
            depth: BoxDepth::Face,
        },
        BoxMarch::Masks {
            solids: true,
            depth: BoxDepth::Hit,
        },
        BoxMarch::Masks {
            solids: true,
            depth: BoxDepth::Face,
        },
    ];

    /// The march's name in a report.
    pub fn label(self) -> String {
        match self {
            BoxMarch::Tree(march) => format!("box raster, {} march", march.label()),

            BoxMarch::Masks { solids, depth } => format!(
                "box raster, {}, {}",
                if solids {
                    "solid boxes and mask march"
                } else {
                    "mask march"
                },
                depth.label()
            ),
        }
    }
}
