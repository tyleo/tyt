use crate::{BoxDraw, BoxFaces, HitPacking, PlacedBox, TileMasks, world_to_clip};
use std::{collections::HashMap, num::NonZeroU32};
use ty_math::{TyMatrix4x4F64, TyVector3F64};
use voxrender::{RenderProjection, RenderScene, RenderView, grid_point, grid_vector};
use wgpu::{
    BufferBindingType, BufferUsages, Device, ShaderModuleDescriptor, ShaderRuntimeChecks,
    ShaderSource, ShaderStages,
    util::{BufferInitDescriptor, DeviceExt},
};

/// The boxes of `scene` as `view` sees them over a `width` by `height`
/// image, drawn by `box_masks.wgsl`. Each fragment steps through its
/// tile's [`TileMasks`].
///
/// # Arguments
///
/// 1. `packing`: how a hit of the scene packs into a texel
/// 2. `multiview_mask`: the mask of the passes the boxes draw in
#[allow(clippy::too_many_arguments)]
pub fn mask_boxes(
    device: &Device,
    scene: &RenderScene,
    view: &RenderView,
    width: u32,
    height: u32,
    faces: BoxFaces,
    packing: HitPacking,
    multiview_mask: Option<NonZeroU32>,
) -> BoxDraw {
    let world_to_clip = world_to_clip(scene, view, width, height);

    let mut words = Vec::new();
    let mut offsets = HashMap::new();
    let mut object_boxes = HashMap::new();

    for (object_id, object) in scene.iter_objects() {
        let masks = TileMasks::new(object);

        offsets.insert(
            object_id,
            u32::try_from(words.len()).expect("a scene's mask words number under 2^32"),
        );

        words.extend(masks.words);
        object_boxes.insert(object_id, masks.boxes);
    }

    // Each placement's `Place` in `box_masks.wgsl`, and the offset of its
    // object's masks.
    let mut places = Vec::new();
    let mut mask_offsets = Vec::new();

    for (_, placement) in scene.iter_placements() {
        let transform = &placement.transform;
        let grid_to_world = TyMatrix4x4F64::from_scale_rotation_translation(
            transform.scale,
            transform.rotation,
            transform.position,
        );

        let ray = match view.projection {
            RenderProjection::Perspective { .. } => {
                grid_point(transform, view.pose.position).extend(1.0)
            }

            RenderProjection::Orthographic { .. } => {
                let forward = view.pose.rotation * -TyVector3F64::Z;

                (-grid_vector(transform, forward)).extend(0.0)
            }
        };

        let floats = (world_to_clip * grid_to_world)
            .to_cols_array()
            .into_iter()
            .chain(ray.to_array());

        for value in floats {
            places.extend((value as f32).to_le_bytes());
        }

        let bounds = scene
            .object(placement.object_id)
            .expect("a placement's object is one of the scene's")
            .bounds();

        for value in [bounds.y * bounds.z, bounds.z, 0, 0] {
            places.extend(value.to_le_bytes());
        }

        mask_offsets.push(offsets[&placement.object_id]);
    }

    let (placed, fronts) = PlacedBox::list(scene, view, &object_boxes, faces);

    // A binding cannot be empty.
    if places.is_empty() {
        places.resize(96, 0);
    }

    if words.is_empty() {
        words.push(0);
    }

    let init = |label, contents: &[u8], usage| {
        device.create_buffer_init(&BufferInitDescriptor {
            label: Some(label),
            contents,
            usage,
        })
    };

    let mut frame = Vec::new();

    for value in [packing.voxel_bits, 0, 0, 0] {
        frame.extend(value.to_le_bytes());
    }

    let frame = init("box frame", &frame, BufferUsages::UNIFORM);
    let places = init("box places", &places, BufferUsages::STORAGE);
    let bits = init(
        "box bits",
        &words
            .iter()
            .flat_map(|word| word.to_le_bytes())
            .collect::<Vec<_>>(),
        BufferUsages::STORAGE,
    );

    // SAFETY: a fragment reads only within its tile's words because each
    // step keeps its cell inside the tile's box. Its loop ends within
    // the box's extent. The vertex stage indexes `places` by a box's
    // placement.
    let module = unsafe {
        device.create_shader_module_trusted(
            ShaderModuleDescriptor {
                label: Some("box masks"),
                source: ShaderSource::Wgsl(
                    format!(
                        "const FACE_BITS: u32 = {}u;\n{}\n{}",
                        HitPacking::FACE_BITS,
                        include_str!("box_masks.wgsl"),
                        match faces {
                            BoxFaces::Back => "",
                            BoxFaces::Front => include_str!("box_mask_fronts.wgsl"),
                        }
                    )
                    .into(),
                ),
            },
            ShaderRuntimeChecks::unchecked(),
        )
    };

    let storage = BufferBindingType::Storage { read_only: true };

    BoxDraw::new(
        device,
        &module,
        faces,
        &placed,
        fronts,
        |placed| {
            let tile = u32::try_from(placed.brick).expect("an object's tiles number under 2^32");

            [
                placed.placement,
                mask_offsets[placed.placement as usize] + tile * TileMasks::WORDS,
            ]
        },
        &[
            (ShaderStages::FRAGMENT, BufferBindingType::Uniform, &frame),
            (ShaderStages::VERTEX, storage, &places),
            (ShaderStages::FRAGMENT, storage, &bits),
        ],
        multiview_mask,
    )
}
