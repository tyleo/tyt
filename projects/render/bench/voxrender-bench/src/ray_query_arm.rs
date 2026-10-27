use crate::BrickMarch;

/// A ray-query trace's bricks and march.
#[derive(Clone, Copy, Debug)]
pub struct RayQueryArm {
    /// The cells along each side of a brick.
    pub brick: u32,

    /// How a candidate finds its brick's first hit.
    pub march: BrickMarch,
}

impl RayQueryArm {
    /// Each march over 4^3 bricks and over 16^3 tiles.
    pub const ALL: [RayQueryArm; 4] = [
        RayQueryArm {
            brick: 4,
            march: BrickMarch::Integer,
        },
        RayQueryArm {
            brick: 16,
            march: BrickMarch::Integer,
        },
        RayQueryArm {
            brick: 4,
            march: BrickMarch::Float,
        },
        RayQueryArm {
            brick: 16,
            march: BrickMarch::Float,
        },
    ];

    /// The arm's name in a report.
    pub fn label(self) -> String {
        format!(
            "ray query, {} march, {}^3 bricks",
            self.march.label(),
            self.brick
        )
    }
}
