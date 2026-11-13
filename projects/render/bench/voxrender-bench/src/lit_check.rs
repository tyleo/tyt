use crate::{Error, Gpu, LitCorners, LitFill, Result, TileMasks, map_readback};
use std::collections::HashMap;
use ty_math::TyVector3F64;
use voxrender::{RenderLight, RenderScene};
use wgpu::{BufferDescriptor, BufferUsages, CommandEncoderDescriptor};

/// Compares the GPU's fill of `scene`'s lit corners under its first sun
/// with [`LitCorners`]' fill through the reference's walk, and returns a
/// line of the result. The line counts the scene's record cells, the
/// corners the two fills light, and the corners they differ on. Every live
/// cell counts as opaque. Errors unless the scene has a sun and a view, or
/// if a ray lies out of a grid's range.
pub fn lit_check(gpu: &Gpu, scene: &RenderScene) -> Result<String> {
    let toward = scene
        .iter_lights()
        .find_map(|(_, light)| match *light {
            RenderLight::Directional { rotation, .. } => Some(rotation * TyVector3F64::Z),
            _ => None,
        })
        .ok_or(Error::NoSun)?;

    let every = |_| true;
    let mut tiles = HashMap::new();

    for (object_id, object) in scene.iter_objects() {
        let masks = TileMasks::new(object, true, Some(&every))?;

        tiles.insert(
            object_id,
            masks
                .corners
                .into_iter()
                .zip(masks.recorded)
                .collect::<Vec<_>>(),
        );
    }

    let glass = scene
        .iter_objects()
        .map(|(object_id, _)| (object_id, Vec::new()))
        .collect();

    let fill = LitFill::new(&gpu.device, scene, toward, &tiles, &glass, &every)?;

    fill.fill(&gpu.device, &gpu.queue);

    let readback = gpu.device.create_buffer(&BufferDescriptor {
        label: Some("lit readback"),
        size: fill.lit.size(),
        usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let mut encoder = gpu
        .device
        .create_command_encoder(&CommandEncoderDescriptor {
            label: Some("lit readback"),
        });

    encoder.copy_buffer_to_buffer(&fill.lit, 0, &readback, 0, None);
    gpu.queue.submit([encoder.finish()]);

    map_readback(&gpu.device, &readback)?;

    let filled: Vec<u32> = readback
        .get_mapped_range(..)
        .map_err(Error::MapRange)?
        .chunks_exact(4)
        .map(|word| u32::from_le_bytes(word.try_into().expect("a chunk holds four bytes")))
        .collect();

    let walked = LitCorners::new(scene, toward, &tiles, &every)?;

    assert_eq!(
        fill.places, walked.places,
        "the fills lay their words out alike"
    );

    let (mut lit, mut differ) = (0u64, 0u64);

    for (filled, walked) in filled.iter().zip(&walked.words) {
        lit += u64::from((filled | walked).count_ones());
        differ += u64::from((filled ^ walked).count_ones());
    }

    Ok(format!(
        "{} record cells, {lit} lit corners, {differ} differ ({:.4}%)\n",
        walked.words.len(),
        differ as f64 / lit.max(1) as f64 * 100.0
    ))
}
