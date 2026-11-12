use crate::{
    BoxDraw, BoxFaces, BoxPart, BrickMarch, Error, HitPacking, HitTargets, PlacedBox, Result,
    Tree64, live_boxes, world_to_clip,
};
use std::{collections::HashMap, num::NonZeroU32};
use ty_math::TyMatrix4x4F64;
use voxrender::{RenderScene, RenderView, RenderViewRays};
use voxrender_wgpu::{GRID_WALK_WGSL, placement_bytes};
use wgpu::{
    BufferBindingType, BufferUsages, Device, ShaderModuleDescriptor, ShaderRuntimeChecks,
    ShaderSource, ShaderStages,
    util::{BufferInitDescriptor, DeviceExt},
};

/// The cells along each side of a tile, which is the brick each box bounds.
const TILE: u32 = 16;

/// The boxes of `scene` as `view` sees them over a `width` by `height`
/// image, drawn by `box_raster.wgsl` in placement order. Each back face's
/// fragment runs `march` over its object's [`Tree64`].
///
/// # Arguments
///
/// 1. `packing`: how a hit of the scene packs into a texel
/// 2. `multiview_mask`: the mask of the passes the boxes draw in
#[allow(clippy::too_many_arguments)]
pub fn tree_boxes(
    device: &Device,
    scene: &RenderScene,
    view: &RenderView,
    width: u32,
    height: u32,
    march: BrickMarch,
    packing: HitPacking,
    multiview_mask: Option<NonZeroU32>,
) -> Result<BoxDraw> {
    let view_rays = RenderViewRays::new(view, width, height);
    let grid_rays = view_rays.to_scene_grid_rays(scene).map_err(Error::Render)?;
    let world_to_clip = world_to_clip(scene, view, width, height);

    let mut words = Vec::new();
    let mut cell_offsets = HashMap::new();
    let mut object_boxes = HashMap::new();

    for (object_id, object) in scene.iter_objects() {
        cell_offsets.insert(
            object_id,
            u32::try_from(words.len()).expect("a scene's cells number under 2^32"),
        );

        words.extend(Tree64::new(object)?.words());
        object_boxes.insert(object_id, live_boxes(object, TILE));
    }

    let mut clips = Vec::new();

    for (_, placement) in scene.iter_placements() {
        let transform = &placement.transform;
        let grid_to_world = TyMatrix4x4F64::from_scale_rotation_translation(
            transform.scale,
            transform.rotation,
            transform.position,
        );

        for value in (world_to_clip * grid_to_world).as_mat4().to_cols_array() {
            clips.extend(value.to_le_bytes());
        }
    }

    let boxes = PlacedBox::list(scene, &object_boxes)
        .iter()
        .map(|placed| placed.words(placed.placement, 0, [0, 0]))
        .collect();

    let mut frame = Vec::new();

    for value in [packing.voxel_bits, 0, 0, 0] {
        frame.extend(value.to_le_bytes());
    }

    let steps = view_rays
        .origin_steps()
        .into_iter()
        .chain(view_rays.direction_steps())
        .map(|step| step.extend(0.0));

    for vector in steps.chain([world_to_clip.row(2), world_to_clip.row(3)]) {
        for value in vector.to_array() {
            frame.extend((value as f32).to_le_bytes());
        }
    }

    let placement_cells: Vec<_> = scene
        .iter_placements()
        .map(|(_, placement)| cell_offsets[&placement.object_id])
        .collect();

    // A binding cannot be empty.
    if clips.is_empty() {
        clips.resize(64, 0);
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

    let frame = init("box frame", &frame, BufferUsages::UNIFORM);
    let placements = init(
        "box placements",
        &placement_bytes(scene, &grid_rays, &placement_cells),
        BufferUsages::STORAGE,
    );
    let cells = init(
        "box cells",
        &words
            .iter()
            .flat_map(|word| word.to_le_bytes())
            .collect::<Vec<_>>(),
        BufferUsages::STORAGE,
    );
    let clips = init("box clips", &clips, BufferUsages::STORAGE);

    // SAFETY: each march reads only within its placement's tree, and
    // each of its loops ends within a box's extent. The rest of the
    // module indexes its bindings by a box's placement.
    let module = unsafe {
        device.create_shader_module_trusted(
            ShaderModuleDescriptor {
                label: Some("box raster"),
                source: ShaderSource::Wgsl(
                    format!(
                        "const FACE_BITS: u32 = {}u;\n{GRID_WALK_WGSL}\n{}\n{}\n{}",
                        HitPacking::FACE_BITS,
                        include_str!("tree_walk.wgsl"),
                        march.wgsl(),
                        include_str!("box_raster.wgsl")
                    )
                    .into(),
                ),
            },
            ShaderRuntimeChecks::unchecked(),
        )
    };

    let storage = BufferBindingType::Storage { read_only: true };

    Ok(BoxDraw::new(
        device,
        &module,
        HitTargets::FORMAT,
        &[BoxPart {
            fragment: "back",
            faces: BoxFaces::Back,
            boxes,
        }],
        &[
            (ShaderStages::FRAGMENT, BufferBindingType::Uniform, &frame),
            (ShaderStages::FRAGMENT, storage, &placements),
            (ShaderStages::FRAGMENT, storage, &cells),
            (ShaderStages::VERTEX, storage, &clips),
        ],
        None,
        multiview_mask,
    ))
}
