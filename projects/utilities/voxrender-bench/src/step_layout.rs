/// The memory a B2 step loop reads its cells from.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StepLayout {
    /// A storage buffer of u32 cells in raster order.
    Buffer,

    /// A 3D texture of u32 texels.
    Volume,

    /// A 2D array texture of u32 texels, one layer per z.
    Layers,

    /// A storage buffer of u32-pair masks, one pair per 4^3 brick.
    Masks,
}

impl StepLayout {
    /// Every layout, in report order.
    pub const ALL: [StepLayout; 4] = [
        StepLayout::Buffer,
        StepLayout::Volume,
        StepLayout::Layers,
        StepLayout::Masks,
    ];

    /// The layout's name in the report.
    pub fn label(self) -> &'static str {
        match self {
            StepLayout::Buffer => "storage buffer",
            StepLayout::Volume => "3D texture",
            StepLayout::Layers => "2D array",
            StepLayout::Masks => "u32-pair masks",
        }
    }

    /// The layout's entry point in `steps.wgsl`.
    pub fn entry_point(self) -> &'static str {
        match self {
            StepLayout::Buffer => "buffer_steps",
            StepLayout::Volume => "volume_steps",
            StepLayout::Layers => "layer_steps",
            StepLayout::Masks => "mask_steps",
        }
    }
}
