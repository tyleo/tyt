use crate::{BoxDepth, BoxDraw, BoxPart, HitPacking, PlacedBox, TileMasks, world_to_clip};
use std::{collections::HashMap, num::NonZeroU32};
use ty_math::{TyMatrix4x4F64, TyVector3F64};
use voxrender::{RenderProjection, RenderScene, RenderView, grid_point, grid_vector};
use wgpu::{
    BufferBindingType, BufferUsages, Device, ShaderModuleDescriptor, ShaderRuntimeChecks,
    ShaderSource, ShaderStages,
    util::{BufferInitDescriptor, DeviceExt},
};

/// The boxes of `scene` as `view` sees them over a `width` by `height`
/// image, drawn by `box_masks.wgsl`. Each fragment steps through its tile's
/// [`TileMasks`] and leaves `depth`. When `solids`, the solid boxes draw
/// first, nearest first, and their fragments step through nothing.
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
    solids: bool,
    depth: BoxDepth,
    packing: HitPacking,
    multiview_mask: Option<NonZeroU32>,
) -> BoxDraw {
    let world_to_clip = world_to_clip(scene, view, width, height);

    let mut words = Vec::new();
    let mut offsets = HashMap::new();
    let mut object_boxes = HashMap::new();
    let mut object_solids = HashMap::new();

    for (object_id, object) in scene.iter_objects() {
        let masks = TileMasks::new(object, solids);

        offsets.insert(
            object_id,
            u32::try_from(words.len()).expect("a scene's mask words number under 2^32"),
        );

        words.extend(masks.words);
        object_boxes.insert(object_id, masks.boxes);
        object_solids.insert(object_id, masks.solids);
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
                        include_str!("mask_march.wgsl")
                    )
                    .into(),
                ),
            },
            ShaderRuntimeChecks::unchecked(),
        )
    };

    let marched = PlacedBox::list(scene, view, &object_boxes);
    let solid = PlacedBox::list(scene, view, &object_solids);

    let march_words = |placed: &PlacedBox| {
        let tile = u32::try_from(placed.brick).expect("an object's tiles number under 2^32");

        placed.words(
            placed.placement,
            mask_offsets[placed.placement as usize] + tile * TileMasks::WORDS,
        )
    };

    // A solid box shows nothing from inside.
    let mut shown: Vec<_> = solid.iter().filter(|placed| !placed.inside).collect();

    shown.sort_by(|a, b| a.distance.total_cmp(&b.distance));

    let mut parts = vec![BoxPart {
        fragment: "solid",
        backs: false,
        boxes: shown
            .iter()
            .map(|placed| placed.words(placed.placement, 0))
            .collect(),
    }];

    match depth {
        BoxDepth::Hit => parts.push(BoxPart {
            fragment: "exact",
            backs: true,
            boxes: marched.iter().map(march_words).collect(),
        }),

        BoxDepth::Face => {
            let crossings = PlacedBox::crossings(&marched, &solid);
            let (exact, apart): (Vec<_>, Vec<_>) = marched
                .iter()
                .zip(crossings)
                .partition(|&(_, crosses)| crosses);

            let (inside, mut front): (Vec<_>, Vec<_>) = apart
                .into_iter()
                .map(|(placed, _)| placed)
                .partition(|placed| placed.inside);

            front.sort_by(|a, b| a.distance.total_cmp(&b.distance));

            parts.extend([
                BoxPart {
                    fragment: "front",
                    backs: false,
                    boxes: front.into_iter().map(march_words).collect(),
                },
                BoxPart {
                    fragment: "inside",
                    backs: true,
                    boxes: inside.into_iter().map(march_words).collect(),
                },
                BoxPart {
                    fragment: "exact",
                    backs: true,
                    boxes: exact
                        .into_iter()
                        .map(|(placed, _)| march_words(placed))
                        .collect(),
                },
            ]);
        }
    }

    let storage = BufferBindingType::Storage { read_only: true };

    BoxDraw::new(
        device,
        &module,
        &parts,
        &[
            (ShaderStages::FRAGMENT, BufferBindingType::Uniform, &frame),
            (ShaderStages::VERTEX, storage, &places),
            (ShaderStages::FRAGMENT, storage, &bits),
        ],
        multiview_mask,
    )
}
