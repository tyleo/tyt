/// A B2 step loop: the memory it reads its cells from and how it walks them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StepLoop {
    /// A storage buffer of u32 cells in raster order.
    Buffer,

    /// A 3D texture of u32 texels.
    Volume,

    /// A 2D array texture of u32 texels, one layer per z.
    Layers,

    /// A storage buffer of u32-pair masks, one pair per 4^3 brick.
    Masks,

    /// The storage buffer, with each pixel's start computed once.
    Hoisted,

    /// Two independent chains of the hoisted loop.
    TwoChains,

    /// The hoisted loop reading the vec4 that holds each cell.
    Quads,

    /// The hoisted loop with one start per workgroup.
    Uniform,
}

impl StepLoop {
    /// Every loop, in report order.
    pub const ALL: [StepLoop; 8] = [
        StepLoop::Buffer,
        StepLoop::Volume,
        StepLoop::Layers,
        StepLoop::Masks,
        StepLoop::Hoisted,
        StepLoop::TwoChains,
        StepLoop::Quads,
        StepLoop::Uniform,
    ];

    /// The loop's name in the report.
    pub fn label(self) -> &'static str {
        match self {
            StepLoop::Buffer => "storage buffer",
            StepLoop::Volume => "3D texture",
            StepLoop::Layers => "2D array",
            StepLoop::Masks => "u32-pair masks",
            StepLoop::Hoisted => "buffer, start hoisted",
            StepLoop::TwoChains => "buffer, two chains",
            StepLoop::Quads => "buffer, vec4 loads",
            StepLoop::Uniform => "buffer, uniform address",
        }
    }

    /// The loop's entry point in `steps.wgsl`.
    pub fn entry_point(self) -> &'static str {
        match self {
            StepLoop::Buffer => "buffer_steps",
            StepLoop::Volume => "volume_steps",
            StepLoop::Layers => "layer_steps",
            StepLoop::Masks => "mask_steps",
            StepLoop::Hoisted => "hoisted_steps",
            StepLoop::TwoChains => "two_chain_steps",
            StepLoop::Quads => "quad_steps",
            StepLoop::Uniform => "uniform_steps",
        }
    }

    /// The reads each invocation makes per step of its count.
    pub fn chains(self) -> u32 {
        match self {
            StepLoop::TwoChains => 2,
            _ => 1,
        }
    }
}
