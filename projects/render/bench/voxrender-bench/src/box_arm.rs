use crate::{BoxFaces, BoxMarch, BrickMarch};
use wgpu::Features;

/// A box raster's march and the faces it draws.
#[derive(Clone, Copy, Debug)]
pub struct BoxArm {
    /// How a fragment finds its box's first hit.
    pub march: BoxMarch,

    /// The faces of each box the raster draws.
    pub faces: BoxFaces,
}

impl BoxArm {
    /// Each march over each choice of faces.
    pub const ALL: [BoxArm; 6] = [
        BoxArm {
            march: BoxMarch::Tree(BrickMarch::Integer),
            faces: BoxFaces::Back,
        },
        BoxArm {
            march: BoxMarch::Tree(BrickMarch::Integer),
            faces: BoxFaces::Front,
        },
        BoxArm {
            march: BoxMarch::Tree(BrickMarch::Float),
            faces: BoxFaces::Back,
        },
        BoxArm {
            march: BoxMarch::Tree(BrickMarch::Float),
            faces: BoxFaces::Front,
        },
        BoxArm {
            march: BoxMarch::Masks,
            faces: BoxFaces::Back,
        },
        BoxArm {
            march: BoxMarch::Masks,
            faces: BoxFaces::Front,
        },
    ];

    /// The device features the arm needs.
    pub fn features(self) -> Features {
        match self.faces {
            BoxFaces::Back => Features::empty(),
            BoxFaces::Front => Features::SHADER_EARLY_DEPTH_TEST,
        }
    }

    /// The arm's name in a report.
    pub fn label(self) -> String {
        format!(
            "box raster, {}, {} faces",
            self.march.label(),
            self.faces.label()
        )
    }
}
