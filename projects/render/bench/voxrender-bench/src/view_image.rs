use crate::{
    BoxView, Error, Gpu, Result, TileScene, ViewEye, ViewOptions, ViewTarget, map_readback,
    world_to_clip,
};
use branded_id::U32Id;
use voxrender::{BRenderView, Error as RenderError, RenderBloom, RenderScene};
use wgpu::{
    BufferAddress, BufferDescriptor, BufferUsages, COPY_BYTES_PER_ROW_ALIGNMENT,
    CommandEncoderDescriptor, Extent3d, Origin3d, TexelCopyBufferInfo, TexelCopyBufferLayout,
    TexelCopyTextureInfo, TextureAspect, TextureDescriptor, TextureDimension, TextureFormat,
    TextureUsages, TextureViewDescriptor,
};

/// The format of the image's texels.
const FORMAT: TextureFormat = TextureFormat::Rgba8UnormSrgb;

/// A [`BoxView`] of `tiles` under `bloom` through two of `scene`'s views,
/// one an eye, as each eye's sRGB texels of an image whose width and
/// height are `size`, four bytes each, row by row. The view draws as
/// `options` have it. Errors unless the views are the scene's and the
/// device has multiview, and as the view does.
pub fn view_image(
    gpu: &Gpu,
    tiles: &TileScene,
    scene: &RenderScene,
    bloom: RenderBloom,
    view_ids: [U32Id<BRenderView>; 2],
    size: [u32; 2],
    options: &ViewOptions,
) -> Result<[Vec<u8>; 2]> {
    let [width, height] = size;

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
        tiles,
        bloom,
        FORMAT,
        size,
        options.page_bytes,
    )?;

    boxes.set_detail(options.detail);

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

    let target = ViewTarget {
        color: &color.create_view(&TextureViewDescriptor::default()),
        depth: &depth.create_view(&TextureViewDescriptor::default()),
        keep_depth: false,
    };

    boxes.encode(&mut encoder, &target, boxes.sky());

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
