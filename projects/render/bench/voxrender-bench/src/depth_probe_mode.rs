use wgpu::Features;

/// What a depth probe's costly fragments do with depth.
#[derive(Clone, Copy, Debug)]
pub enum DepthProbeMode {
    /// Writes color alone.
    Plain,

    /// Can discard, and never does.
    Kills,

    /// Writes the depth the rasterizer gave it.
    Written,

    /// Writes that depth and declares that it writes no nearer depth.
    Bounded,
}

impl DepthProbeMode {
    /// Every mode.
    pub const ALL: [DepthProbeMode; 4] = [
        DepthProbeMode::Plain,
        DepthProbeMode::Kills,
        DepthProbeMode::Written,
        DepthProbeMode::Bounded,
    ];

    /// The device features the mode needs.
    pub fn features(self) -> Features {
        match self {
            DepthProbeMode::Bounded => Features::SHADER_EARLY_DEPTH_TEST,
            _ => Features::empty(),
        }
    }

    /// The mode's fragment entry in `depth_probe.wgsl` and its name in a
    /// report.
    pub fn label(self) -> &'static str {
        match self {
            DepthProbeMode::Plain => "plain",
            DepthProbeMode::Kills => "kills",
            DepthProbeMode::Written => "written",
            DepthProbeMode::Bounded => "bounded",
        }
    }
}
