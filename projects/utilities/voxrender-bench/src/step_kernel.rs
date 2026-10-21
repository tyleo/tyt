use crate::{STEP_GRID, StepGrid, StepLayout};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, BufferUsages, ComputePass,
    ComputePipeline, ComputePipelineDescriptor, Device, PipelineCompilationOptions,
    ShaderModuleDescriptor, ShaderSource,
    util::{BufferInitDescriptor, DeviceExt},
};

/// B2's fixed-step loop over one layout of a [`StepGrid`], one invocation
/// per pixel of a `width` by `height` image.
pub struct StepKernel {
    pipeline: ComputePipeline,

    bind_group: BindGroup,

    width: u32,

    height: u32,
}

impl StepKernel {
    /// Builds the loop of `count` steps over `layout`.
    pub fn new(
        device: &Device,
        grid: &StepGrid,
        layout: StepLayout,
        width: u32,
        height: u32,
        count: u32,
    ) -> Self {
        // The salt is zero, and no cell holds the miss.
        let steps = [width, height, 0, u32::MAX]
            .into_iter()
            .flat_map(u32::to_le_bytes)
            .collect::<Vec<_>>();

        let steps = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("steps"),
            contents: &steps,
            usage: BufferUsages::UNIFORM,
        });

        let ends = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("step ends"),
            contents: &[0; 4],
            usage: BufferUsages::STORAGE,
        });

        let module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("steps"),
            source: ShaderSource::Wgsl(include_str!("steps.wgsl").into()),
        });

        let pipeline = device.create_compute_pipeline(&ComputePipelineDescriptor {
            label: Some(layout.label()),
            layout: None,
            module: &module,
            entry_point: Some(layout.entry_point()),
            compilation_options: PipelineCompilationOptions {
                constants: &[("STEPS", f64::from(count)), ("SIZE", f64::from(STEP_GRID))],
                ..PipelineCompilationOptions::default()
            },
            cache: None,
        });

        let (binding, resource) = match layout {
            StepLayout::Buffer => (1, grid.cells.as_entire_binding()),
            StepLayout::Volume => (2, BindingResource::TextureView(&grid.volume)),
            StepLayout::Layers => (3, BindingResource::TextureView(&grid.layers)),
            StepLayout::Masks => (4, grid.masks.as_entire_binding()),
        };

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some(layout.label()),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: steps.as_entire_binding(),
                },
                BindGroupEntry { binding, resource },
                BindGroupEntry {
                    binding: 5,
                    resource: ends.as_entire_binding(),
                },
            ],
        });

        StepKernel {
            pipeline,
            bind_group,
            width,
            height,
        }
    }

    /// Records the loop over every pixel into `pass`.
    pub fn dispatch(&self, pass: &mut ComputePass<'_>) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.dispatch_workgroups(self.width.div_ceil(8), self.height.div_ceil(8), 1);
    }
}
