use crate::{
    BoxBindings, BoxCrosser, BoxDepth, BoxDraw, BoxFaces, BoxPart, HitPacking, HitTargets,
    PlacedBox, Result, TileMasks, world_to_clip,
};
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
/// nearest first, and their fragments step through nothing.
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
) -> Result<BoxDraw> {
    let world_to_clip = world_to_clip(scene, view, width, height);

    let mut words = Vec::new();
    let mut offsets = HashMap::new();
    let mut object_boxes = HashMap::new();
    let mut object_solids = HashMap::new();
    let mut object_tiles = HashMap::new();

    for (object_id, object) in scene.iter_objects() {
        let masks = TileMasks::new(object, solids, None)?;

        offsets.insert(
            object_id,
            u32::try_from(words.len()).expect("a scene's mask words number under 2^32"),
        );

        words.extend(masks.words);
        object_boxes.insert(object_id, masks.boxes);
        object_solids.insert(object_id, masks.solids);
        object_tiles.insert(object_id, masks.tiles);
    }

    // Each placement's `Place` in `box_masks.wgsl`, the offset of its
    // object's masks, and the offset of each of its boxes' tiles in those.
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

        for (axis, meters) in [TyVector3F64::X, TyVector3F64::Y, TyVector3F64::Z]
            .into_iter()
            .zip(transform.scale.to_array())
        {
            for value in (transform.rotation * axis).extend(meters).to_array() {
                places.extend((value as f32).to_le_bytes());
            }
        }

        mask_offsets.push((
            offsets[&placement.object_id],
            &object_tiles[&placement.object_id],
        ));
    }

    // A binding cannot be empty.
    if places.is_empty() {
        places.resize(144, 0);
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

    let ray_w = match view.projection {
        RenderProjection::Perspective { .. } => 1,
        RenderProjection::Orthographic { .. } => 0,
    };

    // SAFETY: a fragment reads only within a tile's words because each step
    // keeps its cell inside the tile's box. Its loop ends within the box's
    // extent. A box's entries in `crossers` index `boxes`, and a box's
    // placement indexes `places`.
    let module = unsafe {
        device.create_shader_module_trusted(
            ShaderModuleDescriptor {
                label: Some("box masks"),
                source: ShaderSource::Wgsl(
                    format!(
                        "const FACE_BITS: u32 = {}u;\nconst RAY_W: f32 = {ray_w}.0;\n{}\n{}",
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

    let perspective = matches!(view.projection, RenderProjection::Perspective { .. });
    let marched = PlacedBox::list(scene, &object_boxes);
    let solid = PlacedBox::list(scene, &object_solids);

    let tile_of = |placed: &PlacedBox| {
        let (object, tiles) = mask_offsets[placed.placement as usize];

        object + tiles[placed.brick]
    };

    // Each marched box's entries in `crosser_words`, as their first and
    // their count.
    let crossers = PlacedBox::crossers(&marched, &solid);
    let mut crosser_words: Vec<u32> = Vec::new();
    let mut spans = Vec::new();

    for entries in &crossers {
        let count = |entries: usize| {
            u32::try_from(entries).expect("a scene's crossing boxes number under 2^32")
        };

        spans.push([count(crosser_words.len() / 8), count(entries.len())]);
        crosser_words.extend(entries.iter().flat_map(BoxCrosser::words));
    }

    let march_words = |index: usize, span: [u32; 2]| {
        let placed = &marched[index];

        placed.words(placed.placement, tile_of(placed), span)
    };

    let solid_words = |placed: &PlacedBox| placed.words(placed.placement, 0, [0, 0]);

    // `box_masks.wgsl`'s `boxes`: the marched boxes, then the solid ones.
    let mut box_words: Vec<u32> = (0..marched.len())
        .map(|index| march_words(index, [0, 0]))
        .chain(solid.iter().map(solid_words))
        .flat_map(|words| words.into_iter().take(8))
        .collect();

    // Every solid box draws its front faces because a ray that starts
    // inside one sees none of it.
    let solids = BoxPart {
        fragment: "solid",
        faces: BoxFaces::Front,
        module: 0,
        boxes: nearest_first(solid.iter().map(|placed| (placed.seen(view).1, placed)))
            .into_iter()
            .map(solid_words)
            .collect(),
        glass: false,
    };

    let exact = |boxes: Vec<[u32; 12]>| BoxPart {
        fragment: "exact",
        faces: BoxFaces::Back,
        module: 0,
        boxes,
        glass: false,
    };

    let parts = match depth {
        BoxDepth::Hit => vec![
            solids,
            exact(
                (0..marched.len())
                    .map(|index| march_words(index, [0, 0]))
                    .collect(),
            ),
        ],

        BoxDepth::Face => {
            let mut inside = Vec::new();
            let mut front = Vec::new();
            let mut crossed = Vec::new();
            let mut deep = Vec::new();

            for (index, placed) in marched.iter().enumerate() {
                let (holds, distance) = placed.seen(view);
                let crosses = !crossers[index].is_empty();

                // A box that crosses others keeps its face's depth only
                // where its fragment steps through them, which `crossed`
                // does under a perspective view. An orthographic view's
                // depth has no scale that brings a back face nearer.
                let writes = if holds {
                    crosses || !perspective
                } else {
                    crosses && !perspective
                };

                if writes {
                    deep.push(march_words(index, [0, 0]));
                } else if holds {
                    inside.push(march_words(index, [0, 0]));
                } else if crosses {
                    crossed.push((distance, (index, spans[index])));
                } else {
                    front.push((distance, (index, [0, 0])));
                }
            }

            let fronts = |fragment, boxes: Vec<(f64, (usize, [u32; 2]))>| BoxPart {
                fragment,
                faces: BoxFaces::Front,
                module: 0,
                boxes: nearest_first(boxes)
                    .into_iter()
                    .map(|(index, span)| march_words(index, span))
                    .collect(),
                glass: false,
            };

            vec![
                BoxPart {
                    fragment: "inside",
                    faces: BoxFaces::NearBack,
                    module: 0,
                    boxes: inside,
                    glass: false,
                },
                solids,
                fronts("front", front),
                fronts("crossed", crossed),
                exact(deep),
            ]
        }
    };

    // A binding cannot be empty.
    if box_words.is_empty() {
        box_words.resize(8, 0);
    }

    if crosser_words.is_empty() {
        crosser_words.resize(8, 0);
    }

    let words_bytes =
        |words: &[u32]| -> Vec<u8> { words.iter().flat_map(|word| word.to_le_bytes()).collect() };

    let boxes = init("box list", &words_bytes(&box_words), BufferUsages::STORAGE);
    let crossers = init(
        "box crossers",
        &words_bytes(&crosser_words),
        BufferUsages::STORAGE,
    );

    let storage = BufferBindingType::Storage { read_only: true };

    Ok(BoxDraw::new(
        device,
        &[&module],
        HitTargets::FORMAT,
        &parts,
        &BoxBindings {
            buffers: &[
                (ShaderStages::FRAGMENT, BufferBindingType::Uniform, &frame),
                (
                    ShaderStages::VERTEX | ShaderStages::FRAGMENT,
                    storage,
                    &places,
                ),
                (ShaderStages::FRAGMENT, storage, &bits),
                (ShaderStages::FRAGMENT, storage, &boxes),
                (ShaderStages::FRAGMENT, storage, &crossers),
            ],
            sampled: None,
            tlas: None,
        },
        multiview_mask,
    ))
}

/// The items of `seen` by their distances, nearest first.
fn nearest_first<T>(seen: impl IntoIterator<Item = (f64, T)>) -> Vec<T> {
    let mut seen: Vec<_> = seen.into_iter().collect();

    seen.sort_by(|(a, _), (b, _)| a.total_cmp(b));

    seen.into_iter().map(|(_, item)| item).collect()
}
