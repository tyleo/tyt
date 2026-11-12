use wgpu::{Buffer, BufferBindingType, Sampler, ShaderStages, TextureView, Tlas};

/// What a [`BoxDraw`](crate::BoxDraw)'s shaders bind in group 0.
pub struct BoxBindings<'a> {
    /// Each buffer's stages, type, and buffer, in binding order from 0.
    pub buffers: &'a [(ShaderStages, BufferBindingType, &'a Buffer)],

    /// A two-layer texture of filterable floats and its sampler, which the
    /// fragments bind after the buffers.
    pub sampled: Option<(&'a TextureView, &'a Sampler)>,

    /// An acceleration structure the fragments' ray queries trace, bound two
    /// past the buffers.
    pub tlas: Option<&'a Tlas>,
}
