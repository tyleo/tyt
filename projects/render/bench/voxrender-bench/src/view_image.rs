use crate::{BoxView, Error, Gpu, Result, ViewEye, map_readback, world_to_clip};
use branded_id::U32Id;
use std::num::NonZeroU32;
use voxrender::{BRenderView, Error as RenderError, RenderBloom, RenderScene};
use wgpu::{
    BufferAddress, BufferDescriptor, BufferUsages, COPY_BYTES_PER_ROW_ALIGNMENT,
    CommandEncoderDescriptor, Extent3d, LoadOp, Operations, Origin3d, RenderPassColorAttachment,
    RenderPassDepthStencilAttachment, RenderPassDescriptor, StoreOp, TexelCopyBufferInfo,
    TexelCopyBufferLayout, TexelCopyTextureInfo, TextureAspect, TextureDescriptor,
    TextureDimension, TextureFormat, TextureUsages, TextureViewDescriptor,
};

/// The format of the image's texels.
const FORMAT: TextureFormat = TextureFormat::Rgba8UnormSrgb;

/// A [`BoxView`] of `scene` under `bloom` through two of its views, one an
/// eye, as each eye's sRGB texels of a `width` by `height` image, four
/// bytes each, row by row. Errors unless the views are the scene's and the
/// device has multiview.
pub fn view_image(
    gpu: &Gpu,
    scene: &RenderScene,
    bloom: RenderBloom,
    view_ids: [U32Id<BRenderView>; 2],
    width: u32,
    height: u32,
) -> Result<[Vec<u8>; 2]> {
    let eye = |view_id| {
        let view = scene
            .view(view_id)
            .ok_or(Error::Render(RenderError::UnknownView { view_id }))?;

        Ok(ViewEye {
            world_to_clip: world_to_clip(scene, view, width, height),
            position: view.pose.position,
        })
    };

    let eyes = [eye(view_ids[0])?, eye(view_ids[1])?];

    let mut boxes = BoxView::new(
        &gpu.device,
        &gpu.queue,
        scene,
        bloom,
        FORMAT,
        [width, height],
    )?;

    boxes.prepare(&gpu.queue, &eyes, 1.0);

    let texture = |label, format, usage| {
        gpu.device.create_texture(&TextureDescriptor {
            label: Some(label),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 2,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        })
    };

    let color = texture(
        "view eyes",
        FORMAT,
        TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
    );
    let depth = texture(
        "view depth",
        BoxView::DEPTH,
        TextureUsages::RENDER_ATTACHMENT,
    );

    let row_bytes = (width * 4).next_multiple_of(COPY_BYTES_PER_ROW_ALIGNMENT);

    let readback = gpu.device.create_buffer(&BufferDescriptor {
        label: Some("view readback"),
        size: BufferAddress::from(row_bytes) * BufferAddress::from(height) * 2,
        usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let mut encoder = gpu
        .device
        .create_command_encoder(&CommandEncoderDescriptor {
            label: Some("view"),
        });

    let layers = color.create_view(&TextureViewDescriptor::default());
    let depth = depth.create_view(&TextureViewDescriptor::default());
    let passes = boxes.passes();

    boxes.glow(&mut encoder);

    for at in 0..passes {
        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("view"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &layers,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: if at == 0 {
                        LoadOp::Clear(boxes.sky())
                    } else {
                        LoadOp::Load
                    },
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                view: &depth,
                depth_ops: Some(Operations {
                    load: if at == 0 {
                        LoadOp::Clear(0.0)
                    } else {
                        LoadOp::Load
                    },
                    store: if at + 1 == passes {
                        StoreOp::Discard
                    } else {
                        StoreOp::Store
                    },
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: NonZeroU32::new(0b11),
        });

        boxes.draw(&mut pass, at);
    }

    encoder.copy_texture_to_buffer(
        TexelCopyTextureInfo {
            texture: &color,
            mip_level: 0,
            origin: Origin3d::ZERO,
            aspect: TextureAspect::All,
        },
        TexelCopyBufferInfo {
            buffer: &readback,
            layout: TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(row_bytes),
                rows_per_image: Some(height),
            },
        },
        Extent3d {
            width,
            height,
            depth_or_array_layers: 2,
        },
    );

    gpu.queue.submit([encoder.finish()]);

    map_readback(&gpu.device, &readback)?;

    let rows = readback.get_mapped_range(..).map_err(Error::MapRange)?;
    let layer_bytes = row_bytes as usize * height as usize;

    let texels = |layer: &[u8]| {
        layer
            .chunks_exact(row_bytes as usize)
            .flat_map(|row| &row[..width as usize * 4])
            .copied()
            .collect()
    };

    Ok([texels(&rows[..layer_bytes]), texels(&rows[layer_bytes..])])
}
