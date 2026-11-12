use crate::{DepthProbeMode, Error, HitTargets, RasterArm, Result};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, BufferBindingType, BufferUsages, ColorTargetState,
    ColorWrites, CommandEncoder, CompareFunction, DepthBiasState, DepthStencilState, Device,
    FragmentState, MultisampleState, PipelineCompilationOptions, PipelineLayoutDescriptor,
    PrimitiveState, QuerySet, RenderPipeline, RenderPipelineDescriptor, ShaderModuleDescriptor,
    ShaderRuntimeChecks, ShaderSource, ShaderStages, StencilState, VertexState,
    util::{BufferInitDescriptor, DeviceExt},
};

/// The costly layers a probe draws, each behind the last.
const LAYERS: u32 = 4;

/// The steps of a costly fragment's loop.
const STEPS: u32 = 16;

/// One pass draws each eye.
const PASSES: RasterArm = RasterArm {
    multiview: false,
    samples: 1,
};

/// A probe of when the depth test rejects a fragment before it runs. Four
/// costly layers cover two `width` by `height` eyes. Each layer lies behind
/// the last. A near layer can draw first. A device that tests depth first
/// runs the costly fragments of one layer, or of none behind the near layer.
pub struct DepthProbe {
    near: Option<RenderPipeline>,

    costly: RenderPipeline,

    bind_group: BindGroup,

    targets: HitTargets,
}

impl DepthProbe {
    /// Builds the probe of `mode` over two `width` by `height` eyes, behind
    /// a near layer when `occluded`. Errors unless the device has the
    /// mode's features.
    pub fn new(
        device: &Device,
        width: u32,
        height: u32,
        mode: DepthProbeMode,
        occluded: bool,
    ) -> Result<Self> {
        let missing = mode.features() - device.features();

        if !missing.is_empty() {
            return Err(Error::RasterFeatures(missing));
        }

        let targets = HitTargets::new(device, width, height, PASSES)?;

        // SAFETY: the shaders index no binding, and the loop ends at the
        // frame's step count.
        let module = unsafe {
            device.create_shader_module_trusted(
                ShaderModuleDescriptor {
                    label: Some("depth probe"),
                    source: ShaderSource::Wgsl(
                        format!(
                            "{}\n{}",
                            include_str!("depth_probe.wgsl"),
                            match mode {
                                DepthProbeMode::Bounded => {
                                    include_str!("depth_probe_bounded.wgsl")
                                }
                                _ => "",
                            }
                        )
                        .into(),
                    ),
                },
                ShaderRuntimeChecks::unchecked(),
            )
        };

        // Both pipelines share the layout, so one bind group serves both.
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("depth probe"),
            entries: &[BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("depth probe"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        let pipeline = |fragment: &str| {
            device.create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("depth probe"),
                layout: Some(&layout),
                vertex: VertexState {
                    module: &module,
                    entry_point: Some("cover"),
                    compilation_options: PipelineCompilationOptions::default(),
                    buffers: &[],
                },
                primitive: PrimitiveState::default(),
                depth_stencil: Some(DepthStencilState {
                    format: HitTargets::DEPTH,
                    depth_write_enabled: Some(true),
                    depth_compare: Some(CompareFunction::Greater),
                    stencil: StencilState::default(),
                    bias: DepthBiasState::default(),
                }),
                multisample: MultisampleState::default(),
                fragment: Some(FragmentState {
                    module: &module,
                    entry_point: Some(fragment),
                    compilation_options: PipelineCompilationOptions::default(),
                    targets: &[Some(ColorTargetState {
                        format: HitTargets::FORMAT,
                        blend: None,
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                multiview_mask: targets.multiview_mask(),
                cache: None,
            })
        };

        let frame = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("depth probe frame"),
            contents: &[STEPS, 0, 0, 0]
                .into_iter()
                .flat_map(u32::to_le_bytes)
                .collect::<Vec<_>>(),
            usage: BufferUsages::UNIFORM,
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("depth probe"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: frame.as_entire_binding(),
            }],
        });

        Ok(DepthProbe {
            near: occluded.then(|| pipeline("near")),
            costly: pipeline(mode.label()),
            bind_group,
            targets,
        })
    }

    /// Records the probe over both eyes into `encoder`, between the
    /// timestamps at `first` and `first + 1` of `query_set`.
    pub fn record(&self, encoder: &mut CommandEncoder, query_set: &QuerySet, first: u32) {
        self.targets
            .draw(encoder, Some((query_set, first)), 1, |pass, _, _| {
                pass.set_bind_group(0, &self.bind_group, &[]);

                if let Some(near) = &self.near {
                    pass.set_pipeline(near);
                    pass.draw(0..3, 0..1);
                }

                pass.set_pipeline(&self.costly);
                pass.draw(0..3, 1..1 + LAYERS);
            });
    }
}
