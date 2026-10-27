use wgpu::Features;

/// How a raster draws both eyes.
#[derive(Clone, Copy, Debug)]
pub struct RasterArm {
    /// Whether one multiview pass draws both eyes. Otherwise one pass draws
    /// each eye.
    pub multiview: bool,

    /// The samples per pixel.
    pub samples: u32,
}

impl RasterArm {
    /// The arms B2 times.
    pub const ALL: [RasterArm; 3] = [
        RasterArm {
            multiview: false,
            samples: 1,
        },
        RasterArm {
            multiview: true,
            samples: 1,
        },
        RasterArm {
            multiview: true,
            samples: 4,
        },
    ];

    /// The device features the arm needs.
    pub fn features(self) -> Features {
        let mut features = Features::empty();

        if self.multiview {
            features |= Features::MULTIVIEW;
        }

        // Both eyes' samples live in one two-layer texture.
        if self.samples > 1 {
            features |= Features::MULTISAMPLE_ARRAY;
        }

        features
    }

    /// The arm's name in a report.
    pub fn label(self) -> String {
        let passes = if self.multiview {
            "multiview"
        } else {
            "two passes"
        };

        if self.samples > 1 {
            format!("raster, {passes}, {}x MSAA", self.samples)
        } else {
            format!("raster, {passes}")
        }
    }
}
