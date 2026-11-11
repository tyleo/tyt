use crate::{Error, RasterArm, Result};
use std::{num::NonZeroU32, sync::mpsc};
use wgpu::{
    Buffer, BufferAddress, BufferDescriptor, BufferUsages, COPY_BYTES_PER_ROW_ALIGNMENT, Color,
    CommandEncoder, CommandEncoderDescriptor, Device, Extent3d, LoadOp, MapMode, Operations,
    Origin3d, PollType, QuerySet, Queue, RenderPass, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, RenderPassTimestampWrites, StoreOp,
    TexelCopyBufferInfo, TexelCopyBufferLayout, TexelCopyTextureInfo, Texture, TextureAspect,
    TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView,
    TextureViewDescriptor, TextureViewDimension,
};

/// Two `width` by `height` eyes of packed hits with their depth, and the
/// passes that draw into them. Depth runs reversed in f32.
pub struct HitTargets {
    multiview_mask: Option<NonZeroU32>,

    /// Both eyes' texels, one layer each.
    output: Texture,

    /// One target per pass.
    targets: Vec<HitTarget>,

    width: u32,

    height: u32,
}

/// What one pass draws into.
struct HitTarget {
    color: TextureView,

    /// The single-sample view an MSAA pass resolves into.
    resolve: Option<TextureView>,

    depth: TextureView,
}

impl HitTargets {
    /// The format a raster writes. Each texel holds a packed hit in the four
    /// bytes a shaded pixel would take.
    pub const FORMAT: TextureFormat = TextureFormat::Rgba8Unorm;

    /// The depth format.
    pub const DEPTH: TextureFormat = TextureFormat::Depth32Float;

    /// Builds the eyes as `arm` draws them. Errors unless the device has the
    /// arm's features.
    pub fn new(device: &Device, width: u32, height: u32, arm: RasterArm) -> Result<Self> {
        let missing = arm.features() - device.features();

        if !missing.is_empty() {
            return Err(Error::RasterFeatures(missing));
        }

        let texture = |label, layers, samples, format, usage| {
            device.create_texture(&TextureDescriptor {
                label: Some(label),
                size: Extent3d {
                    width,
                    height,
                    depth_or_array_layers: layers,
                },
                mip_level_count: 1,
                sample_count: samples,
                dimension: TextureDimension::D2,
                format,
                usage,
                view_formats: &[],
            })
        };

        let output = texture(
            "raster eyes",
            2,
            1,
            Self::FORMAT,
            TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
        );

        // A tiler keeps the samples and depth in tile memory because each
        // pass clears and discards them. The samples and depth skip transient
        // textures because wgpu's hold one layer.
        let samples = (arm.samples > 1).then(|| {
            texture(
                "raster samples",
                2,
                arm.samples,
                Self::FORMAT,
                TextureUsages::RENDER_ATTACHMENT,
            )
        });

        let depth = texture(
            "raster depth",
            if arm.multiview { 2 } else { 1 },
            arm.samples,
            Self::DEPTH,
            TextureUsages::RENDER_ATTACHMENT,
        );

        // The multiview pass draws both layers, and each eye's pass draws
        // one.
        let layers: Vec<Option<u32>> = if arm.multiview {
            vec![None]
        } else {
            vec![Some(0), Some(1)]
        };

        let layer_view = |texture: &Texture, layer: Option<u32>| {
            texture.create_view(&TextureViewDescriptor {
                dimension: Some(match layer {
                    Some(_) => TextureViewDimension::D2,
                    None => TextureViewDimension::D2Array,
                }),
                base_array_layer: layer.unwrap_or(0),
                array_layer_count: Some(if layer.is_some() { 1 } else { 2 }),
                ..TextureViewDescriptor::default()
            })
        };

        let depth = depth.create_view(&TextureViewDescriptor::default());

        let targets = layers
            .into_iter()
            .map(|layer| {
                let resolved = layer_view(&output, layer);

                match &samples {
                    Some(samples) => HitTarget {
                        color: layer_view(samples, layer),
                        resolve: Some(resolved),
                        depth: depth.clone(),
                    },

                    None => HitTarget {
                        color: resolved,
                        resolve: None,
                        depth: depth.clone(),
                    },
                }
            })
            .collect();

        Ok(HitTargets {
            multiview_mask: NonZeroU32::new(if arm.multiview { 0b11 } else { 0 }),
            output,
            targets,
            width,
            height,
        })
    }

    /// The mask a pipeline that draws into the eyes takes.
    pub fn multiview_mask(&self) -> Option<NonZeroU32> {
        self.multiview_mask
    }

    /// Records every pass into `encoder`. `record` draws each pass, given
    /// the pass's eye. The multiview pass draws as eye 0. When `timestamps`
    /// holds a query set and a first index, the passes write the timestamps
    /// at that index and the next.
    pub fn draw(
        &self,
        encoder: &mut CommandEncoder,
        timestamps: Option<(&QuerySet, u32)>,
        mut record: impl FnMut(&mut RenderPass<'_>, u32),
    ) {
        let last = self.targets.len() - 1;

        for (eye, target) in (0u32..).zip(&self.targets) {
            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("raster"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &target.color,
                    depth_slice: None,
                    resolve_target: target.resolve.as_ref(),
                    ops: Operations {
                        load: LoadOp::Clear(Color::WHITE),
                        store: if target.resolve.is_some() {
                            StoreOp::Discard
                        } else {
                            StoreOp::Store
                        },
                    },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &target.depth,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(0.0),
                        store: StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: timestamps.map(|(query_set, first)| RenderPassTimestampWrites {
                    query_set,
                    beginning_of_pass_write_index: (eye == 0).then_some(first),
                    end_of_pass_write_index: (eye as usize == last).then_some(first + 1),
                }),
                occlusion_query_set: None,
                multiview_mask: self.multiview_mask,
            });

            record(&mut pass, eye);
        }
    }

    /// Draws once through `record` and reads back the first eye's texels as
    /// words, row by row.
    pub fn read(
        &self,
        device: &Device,
        queue: &Queue,
        record: impl FnMut(&mut RenderPass<'_>, u32),
    ) -> Result<Vec<u32>> {
        let row_bytes = (self.width * 4).next_multiple_of(COPY_BYTES_PER_ROW_ALIGNMENT);

        let readback = device.create_buffer(&BufferDescriptor {
            label: Some("raster readback"),
            size: BufferAddress::from(row_bytes) * BufferAddress::from(self.height),
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("raster"),
        });

        self.draw(&mut encoder, None, record);

        encoder.copy_texture_to_buffer(
            TexelCopyTextureInfo {
                texture: &self.output,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            TexelCopyBufferInfo {
                buffer: &readback,
                layout: TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(row_bytes),
                    rows_per_image: None,
                },
            },
            Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );

        queue.submit([encoder.finish()]);

        map(device, &readback)?;

        let view = readback.get_mapped_range(..).map_err(Error::MapRange)?;

        Ok(view
            .chunks_exact(row_bytes as usize)
            .flat_map(|row| row[..self.width as usize * 4].chunks_exact(4))
            .map(|texel| u32::from_le_bytes(texel.try_into().expect("a texel holds four bytes")))
            .collect())
    }
}

/// Maps `readback` for reading once the GPU has written it.
fn map(device: &Device, readback: &Buffer) -> Result<()> {
    let (sender, receiver) = mpsc::channel();

    readback.map_async(MapMode::Read, .., move |result| {
        sender.send(result).expect("the receiver outlives the wait");
    });

    device
        .poll(PollType::wait_indefinitely())
        .map_err(Error::Poll)?;

    receiver
        .recv()
        .expect("the map callback runs during the wait")
        .map_err(Error::Map)
}
