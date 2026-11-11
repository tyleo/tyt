use crate::{BoxFaces, BrickMarch};
use wgpu::Features;

/// A box raster's march and the faces it draws.
#[derive(Clone, Copy, Debug)]
pub struct BoxArm {
    /// How a fragment finds its box's first hit.
    pub march: BrickMarch,

    /// The faces of each box the raster draws.
    pub faces: BoxFaces,
}

impl BoxArm {
    /// Each march over each choice of faces.
    pub const ALL: [BoxArm; 4] = [
        BoxArm {
            march: BrickMarch::Integer,
            faces: BoxFaces::Back,
        },
        BoxArm {
            march: BrickMarch::Integer,
            faces: BoxFaces::Front,
        },
        BoxArm {
            march: BrickMarch::Float,
            faces: BoxFaces::Back,
        },
        BoxArm {
            march: BrickMarch::Float,
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
            "box raster, {} march, {} faces",
            self.march.label(),
            self.faces.label()
        )
    }
}
