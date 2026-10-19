use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BufferUsages, ComputePass, ComputePipeline,
    ComputePipelineDescriptor, Device, PipelineCompilationOptions, ShaderModuleDescriptor,
    ShaderSource,
    util::{BufferInitDescriptor, DeviceExt},
};

/// Each link's step, which stays exact in f32 under the limit.
const STEP: u32 = 7919;

/// The value each link's sum wraps at.
const LIMIT: u32 = 65521;

/// B1's dependent-chain kernel over one invocation per pixel of a `width` by
/// `height` image.
pub struct ChainKernel {
    pipeline: ComputePipeline,

    bind_group: BindGroup,

    width: u32,

    height: u32,
}

impl ChainKernel {
    /// Builds the chain's pipeline in f32 when `float`, else in i32.
    pub fn new(device: &Device, width: u32, height: u32, float: bool) -> Self {
        let chain = [width, height, STEP, LIMIT]
            .into_iter()
            .flat_map(u32::to_le_bytes)
            .collect::<Vec<_>>();

        let chain = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("chain"),
            contents: &chain,
            usage: BufferUsages::UNIFORM,
        });

        let ends = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("chain ends"),
            contents: &[0; 4],
            usage: BufferUsages::STORAGE,
        });

        let module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("chain"),
            source: ShaderSource::Wgsl(include_str!("chain.wgsl").into()),
        });

        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some("chain"),
            layout: None,
            module: &module,
            entry_point: Some("main"),
            compilation_options: PipelineCompilationOptions {
                constants: &[("FLOAT_CHAIN", f64::from(u8::from(float)))],
                ..PipelineCompilationOptions::default()
            },
            cache: None,
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("chain"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: chain.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: ends.as_entire_binding(),
                },
            ],
        });

        ChainKernel {
            pipeline,
            bind_group,
            width,
            height,
        }
    }

    /// Records the chain over every pixel into `pass`.
    pub fn dispatch(&self, pass: &mut ComputePass<'_>) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.dispatch_workgroups(self.width.div_ceil(8), self.height.div_ceil(8), 1);
    }
}
