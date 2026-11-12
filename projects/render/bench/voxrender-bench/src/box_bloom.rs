use std::num::NonZeroU32;
use voxrender::RenderBloom;
use wgpu::{
    AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, BlendComponent, BlendFactor,
    BlendOperation, BlendState, Buffer, BufferBindingType, BufferUsages, Color, ColorTargetState,
    ColorWrites, CommandEncoder, Device, Extent3d, FilterMode, FragmentState, LoadOp,
    MultisampleState, Operations, PipelineCompilationOptions, PipelineLayoutDescriptor,
    PrimitiveState, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, SamplerBindingType, SamplerDescriptor, ShaderModuleDescriptor,
    ShaderSource, ShaderStages, StoreOp, TextureDescriptor, TextureDimension, TextureFormat,
    TextureSampleType, TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension,
    VertexState,
    util::{BufferInitDescriptor, DeviceExt},
};

/// The pixels of the eyes that the first level's blur spans as a Gaussian's
/// standard deviation. Each level after spans twice the one before.
const FIRST_SIGMA: f64 = 3.0;

/// The most levels a halo keeps.
const MOST_LEVELS: usize = 6;

/// The contract's bloom over a [`BoxView`](crate::BoxView)'s eyes.
///
/// The view draws its emission over the threshold into the
/// [`source`](Self::source), at half the eyes' size and scaled by
/// [`scale`](Self::scale). [`run`](Self::run) then halves that into the
/// levels and adds the levels back up into the first. The first level spans
/// a quarter of the eyes' size. A level stands in for the contract's octaves
/// nearest its blur. The frame's fragments add the first level, which is the
/// halo.
pub struct BoxBloom {
    /// The source's view, then the levels', the halo first.
    levels: Vec<TextureView>,

    /// What the view scales the emission it draws by: the strength over
    /// the octaves.
    scale: f64,

    /// For each level: what the pass that halves into it binds.
    halves: Vec<BindGroup>,

    /// For each level but the last: what the pass that adds the next level
    /// binds, then the share of its own light it keeps.
    adds: Vec<(BindGroup, f64)>,

    down: RenderPipeline,

    up: RenderPipeline,
}

impl BoxBloom {
    /// The format of the source and the halo.
    pub const FORMAT: TextureFormat = TextureFormat::Rgba16Float;

    /// The bloom of `bloom` over eyes whose width and height are `size`.
    pub fn new(device: &Device, size: [u32; 2], bloom: RenderBloom) -> Self {
        let weights = level_weights(bloom.radius * f64::from(size[0].min(size[1])));
        let count = weights.len();
        let octaves: f64 = weights.iter().sum();

        let levels: Vec<_> = (0..=count)
            .map(|level| {
                let side = |side: u32| (side.div_ceil(2) >> level).max(1);

                device
                    .create_texture(&TextureDescriptor {
                        label: Some("bloom level"),
                        size: Extent3d {
                            width: side(size[0]),
                            height: side(size[1]),
                            depth_or_array_layers: 2,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: TextureDimension::D2,
                        format: Self::FORMAT,
                        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
                        view_formats: &[],
                    })
                    .create_view(&TextureViewDescriptor::default())
            })
            .collect();

        let layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("bloom"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("bloom"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            ..SamplerDescriptor::default()
        });

        // `box_bloom.wgsl`'s `Bloom`.
        let values = |scale: f64| {
            device.create_buffer_init(&BufferInitDescriptor {
                label: Some("bloom"),
                contents: &[scale as f32, 0.0, 0.0, 0.0]
                    .iter()
                    .flat_map(|value| value.to_le_bytes())
                    .collect::<Vec<_>>(),
                usage: BufferUsages::UNIFORM,
            })
        };

        let bound = |values: &Buffer, read: &TextureView| {
            device.create_bind_group(&BindGroupDescriptor {
                label: Some("bloom"),
                layout: &layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: values.as_entire_binding(),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::TextureView(read),
                    },
                    BindGroupEntry {
                        binding: 2,
                        resource: BindingResource::Sampler(&sampler),
                    },
                ],
            })
        };

        // A lone level stands in for every octave.
        let alone = if count == 1 { weights[0] } else { 1.0 };
        let plain = values(1.0);

        let halves = levels[..count]
            .iter()
            .map(|finer| bound(&plain, finer))
            .collect();

        // The last level joins the sum by its weight as the level before
        // reads it. Each other level's weight scales it as the next adds
        // in.
        let adds = (0..count - 1)
            .map(|level| {
                let scale = if level + 2 == count {
                    weights[count - 1]
                } else {
                    1.0
                };

                (bound(&values(scale), &levels[level + 2]), weights[level])
            })
            .collect();

        let module = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("bloom"),
            source: ShaderSource::Wgsl(include_str!("box_bloom.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("bloom"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });

        // An `up` pass keeps its target's light by the blend's constant.
        let kept = BlendComponent {
            src_factor: BlendFactor::One,
            dst_factor: BlendFactor::Constant,
            operation: BlendOperation::Add,
        };

        let pipeline = |fragment, blend| {
            device.create_render_pipeline(&RenderPipelineDescriptor {
                label: Some(fragment),
                layout: Some(&pipeline_layout),
                vertex: VertexState {
                    module: &module,
                    entry_point: Some("corner"),
                    compilation_options: PipelineCompilationOptions::default(),
                    buffers: &[],
                },
                primitive: PrimitiveState::default(),
                depth_stencil: None,
                multisample: MultisampleState::default(),
                fragment: Some(FragmentState {
                    module: &module,
                    entry_point: Some(fragment),
                    compilation_options: PipelineCompilationOptions::default(),
                    targets: &[Some(ColorTargetState {
                        format: Self::FORMAT,
                        blend,
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                multiview_mask: NonZeroU32::new(0b11),
                cache: None,
            })
        };

        BoxBloom {
            down: pipeline("down", None),
            up: pipeline(
                "up",
                Some(BlendState {
                    color: kept,
                    alpha: kept,
                }),
            ),
            levels,
            scale: bloom.strength * alone / octaves,
            halves,
            adds,
        }
    }

    /// What a frame draws its emission into, a layer an eye, at half the
    /// eyes' size and in [`FORMAT`](Self::FORMAT).
    pub fn source(&self) -> &TextureView {
        &self.levels[0]
    }

    /// What a frame scales the emission it draws into the
    /// [`source`](Self::source) by.
    pub fn scale(&self) -> f64 {
        self.scale
    }

    /// The halo both eyes' fragments add, a layer an eye, at a quarter of
    /// the eyes' size. It holds a frame's halo once the frame
    /// [`run`](Self::run)s.
    pub fn halo(&self) -> &TextureView {
        &self.levels[1]
    }

    /// Encodes the passes that turn the emission a frame drew into the
    /// [`source`](Self::source) into the [`halo`](Self::halo).
    pub fn run(&self, encoder: &mut CommandEncoder) {
        let mut pass = |target: usize, pipeline, bound, kept: Option<f64>| {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("bloom"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &self.levels[target],
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations {
                        load: if kept.is_some() {
                            LoadOp::Load
                        } else {
                            LoadOp::Clear(Color::BLACK)
                        },
                        store: StoreOp::Store,
                    },
                })],
                multiview_mask: NonZeroU32::new(0b11),
                ..RenderPassDescriptor::default()
            });

            if let Some(kept) = kept {
                pass.set_blend_constant(Color {
                    r: kept,
                    g: kept,
                    b: kept,
                    a: kept,
                });
            }

            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, bound, &[]);
            pass.draw(0..3, 0..1);
        };

        for (level, bound) in self.halves.iter().enumerate() {
            pass(level + 1, &self.down, bound, None);
        }

        for (level, (bound, kept)) in self.adds.iter().enumerate().rev() {
            pass(level + 1, &self.up, bound, Some(*kept));
        }
    }
}

/// How many of the contract's octaves each level stands in for, the first
/// level first. The octaves blur by `sigma` pixels of the eyes and by each
/// half of it down to a pixel. An octave falls to the level whose blur
/// lies nearest its own.
fn level_weights(mut sigma: f64) -> Vec<f64> {
    let nearest = |sigma: f64| (sigma / FIRST_SIGMA).log2().round().max(0.0) as usize;
    let mut weights = vec![0.0; (nearest(sigma) + 1).min(MOST_LEVELS)];
    let last = weights.len() - 1;

    loop {
        weights[nearest(sigma).min(last)] += 1.0;
        sigma /= 2.0;

        if sigma < 1.0 {
            break;
        }
    }

    weights
}

#[cfg(test)]
mod tests {
    use crate::box_bloom::level_weights;

    #[test]
    fn each_octave_falls_to_the_level_nearest_its_blur() {
        // A headset's eye, 1680 pixels across at a radius of 0.03: octaves
        // of 50.4 pixels down to 1.6, the last two under the first level.
        assert_eq!(level_weights(50.4), [2.0, 1.0, 1.0, 1.0, 1.0]);

        // A small image's one octave.
        assert_eq!(level_weights(1.5), [1.0]);

        // A wide blur's far octaves share the last level.
        assert_eq!(level_weights(400.0).len(), 6);
        assert_eq!(level_weights(400.0).iter().sum::<f64>(), 9.0);
    }
}
