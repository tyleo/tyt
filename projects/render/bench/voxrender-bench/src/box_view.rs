use crate::{BoxDraw, BoxPart, HitTargets, PlacedBox, Result, TileMasks, ViewEye};
use std::{collections::HashMap, f64::consts::PI, num::NonZeroU32};
use ty_math::{TyLinSrgbF32, TyLinSrgbF64, TyMatrix4x4F64, TyTransformF64, TyVector3F64};
use voxrender::{RenderLight, RenderScene, grid_point, tonemap};
use wgpu::{
    Buffer, BufferBindingType, BufferDescriptor, BufferUsages, Color, Device, Queue, RenderPass,
    ShaderModuleDescriptor, ShaderRuntimeChecks, ShaderSource, ShaderStages, TextureFormat,
    util::{BufferInitDescriptor, DeviceExt},
};

/// The bytes of one of `box_view.wgsl`'s `Place`s.
const PLACE_BYTES: usize = 128;

/// The meters past the eyes within which a box counts as holding an eye.
/// The reach covers the near plane.
const REACH: f64 = 0.02;

/// A shaded view of a scene's boxes for both eyes of a headset: the box
/// raster with solid boxes, a mask march, and face depth. Each frame takes
/// its eyes, sorts the boxes nearest first, and draws in one multiview pass
/// over a two-layer target with depth in [`BoxView::DEPTH`].
///
/// A hit takes its cell's base color, the scene's first sun without shadows,
/// and its hemispheres, under the contract's tonemap.
pub struct BoxView {
    boxes: BoxDraw,

    /// Each eye's `Place` for each placement.
    places: Buffer,

    /// Each placement's transform and its grid-to-world matrix.
    placements: Vec<(TyTransformF64, TyMatrix4x4F64)>,

    /// The solid boxes with their words.
    solid: Vec<(PlacedBox, [u32; 8])>,

    /// The marched boxes with their words and whether each crosses another
    /// placement's box.
    marched: Vec<(PlacedBox, [u32; 8], bool)>,

    sky: Color,
}

impl BoxView {
    /// The format of the depth a pass that draws the view clears to zero.
    pub const DEPTH: TextureFormat = HitTargets::DEPTH;

    /// Builds the view of `scene` for a two-layer target of `format`. Errors
    /// if a tile holds over 256 materials.
    pub fn new(device: &Device, scene: &RenderScene, format: TextureFormat) -> Result<Self> {
        let mut words = Vec::new();
        let mut offsets = HashMap::new();
        let mut object_boxes = HashMap::new();
        let mut object_solids = HashMap::new();
        let mut object_tiles = HashMap::new();

        for (object_id, object) in scene.iter_objects() {
            let masks = TileMasks::new(object, true, true)?;

            offsets.insert(
                object_id,
                u32::try_from(words.len()).expect("a scene's mask words number under 2^32"),
            );

            words.extend(masks.words);
            object_boxes.insert(object_id, masks.boxes);
            object_solids.insert(object_id, masks.solids);
            object_tiles.insert(object_id, (masks.tiles, masks.solid_tiles));
        }

        let object_ids: Vec<_> = scene
            .iter_placements()
            .map(|(_, placement)| placement.object_id)
            .collect();

        let placements: Vec<_> = scene
            .iter_placements()
            .map(|(_, placement)| {
                let transform = placement.transform;

                (
                    transform,
                    TyMatrix4x4F64::from_scale_rotation_translation(
                        transform.scale,
                        transform.rotation,
                        transform.position,
                    ),
                )
            })
            .collect();

        let marched = PlacedBox::list(scene, &object_boxes);
        let solid = PlacedBox::list(scene, &object_solids);
        let crossings = PlacedBox::crossings(&marched, &solid);

        // A box's tile sits past its object's first word.
        let tile = |placed: &PlacedBox, solid: bool| {
            let object_id = object_ids[placed.placement as usize];
            let (tiles, solid_tiles) = &object_tiles[&object_id];

            offsets[&object_id] + if solid { solid_tiles } else { tiles }[placed.brick]
        };

        let solid: Vec<_> = solid
            .into_iter()
            .map(|placed| {
                let words = placed.words(placed.placement, tile(&placed, true));

                (placed, words)
            })
            .collect();

        let marched: Vec<_> = marched
            .into_iter()
            .zip(crossings)
            .map(|(placed, crosses)| {
                let words = placed.words(placed.placement, tile(&placed, false));

                (placed, words, crosses)
            })
            .collect();

        let mut sun = (TyVector3F64::Y, TyLinSrgbF64::new(0.0, 0.0, 0.0));
        let mut sky = TyLinSrgbF64::new(0.0, 0.0, 0.0);
        let mut ground = sky;
        let mut suns = 0;

        for (_, light) in scene.iter_lights() {
            match *light {
                RenderLight::Directional {
                    rotation,
                    color,
                    strength,
                    ..
                } => {
                    if suns == 0 {
                        sun = (rotation * TyVector3F64::Z, color * (strength / PI));
                    }

                    suns += 1;
                }

                RenderLight::Hemisphere {
                    sky: above,
                    ground: below,
                    strength,
                } => {
                    sky += above * strength;
                    ground += below * strength;
                }

                _ => {}
            }
        }

        let mut frame = Vec::new();

        let lights = [
            sun.0.to_array(),
            [sun.1.red, sun.1.green, sun.1.blue],
            [sky.red, sky.green, sky.blue],
            [ground.red, ground.green, ground.blue],
        ];

        for value in lights.iter().flat_map(|light| light.iter().chain([&0.0])) {
            frame.extend((*value as f32).to_le_bytes());
        }

        let placement_count =
            u32::try_from(placements.len()).expect("a scene's placements number under 2^32");

        for value in [placement_count, 0, 0, 0] {
            frame.extend(value.to_le_bytes());
        }

        let materials = scene
            .iter_materials()
            .map(|(material_id, _)| material_id.to_u32() + 1)
            .max()
            .unwrap_or(0);

        let mut colors = vec![0u8; materials.max(1) as usize * 16];

        for (material_id, material) in scene.iter_materials() {
            let color = material.base_color.color;
            let first = material_id.to_u32() as usize * 16;

            for (at, value) in [color.red, color.green, color.blue].into_iter().enumerate() {
                colors[first + at * 4..first + at * 4 + 4]
                    .copy_from_slice(&(value as f32).to_le_bytes());
            }
        }

        // A binding cannot be empty.
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

        let frame = init("view frame", &frame, BufferUsages::UNIFORM);
        let bits = init(
            "view bits",
            &words
                .iter()
                .flat_map(|word| word.to_le_bytes())
                .collect::<Vec<_>>(),
            BufferUsages::STORAGE,
        );
        let colors = init("view colors", &colors, BufferUsages::STORAGE);

        let places = device.create_buffer(&BufferDescriptor {
            label: Some("view places"),
            size: (placements.len().max(1) * 2 * PLACE_BYTES) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // SAFETY: a fragment reads only within its tile's words because each
        // step keeps its cell inside the tile's box. Its loop ends within
        // the box's extent. A tile's words hold its cells' materials, which
        // index `colors`. The stages index `places` by an eye and a box's
        // placement.
        let module = unsafe {
            device.create_shader_module_trusted(
                ShaderModuleDescriptor {
                    label: Some("box view"),
                    source: ShaderSource::Wgsl(
                        format!(
                            "{}\n{}",
                            include_str!("box_view.wgsl"),
                            include_str!("mask_march.wgsl")
                        )
                        .into(),
                    ),
                },
                ShaderRuntimeChecks::unchecked(),
            )
        };

        let storage = BufferBindingType::Storage { read_only: true };
        let both = ShaderStages::VERTEX | ShaderStages::FRAGMENT;

        // Every box starts in its first run, which holds the buffer's room.
        // Each frame then sorts the boxes into their runs.
        let boxes = BoxDraw::new(
            device,
            &module,
            format,
            &[
                BoxPart {
                    fragment: "solid",
                    backs: false,
                    boxes: solid.iter().map(|&(_, words)| words).collect(),
                },
                BoxPart {
                    fragment: "front",
                    backs: false,
                    boxes: marched.iter().map(|&(_, words, _)| words).collect(),
                },
                BoxPart {
                    fragment: "inside",
                    backs: true,
                    boxes: Vec::new(),
                },
                BoxPart {
                    fragment: "exact",
                    backs: true,
                    boxes: Vec::new(),
                },
            ],
            &[
                (both, BufferBindingType::Uniform, &frame),
                (both, storage, &places),
                (ShaderStages::FRAGMENT, storage, &bits),
                (ShaderStages::FRAGMENT, storage, &colors),
            ],
            NonZeroU32::new(0b11),
        );

        let above = tonemap(TyLinSrgbF32::new(
            sky.red as f32,
            sky.green as f32,
            sky.blue as f32,
        ));

        Ok(BoxView {
            boxes,
            places,
            placements,
            solid,
            marched,
            sky: Color {
                r: f64::from(above.red),
                g: f64::from(above.green),
                b: f64::from(above.blue),
                a: 1.0,
            },
        })
    }

    /// The color a pass that draws the view clears its target to: the
    /// hemisphere's light from above, tonemapped.
    pub fn sky(&self) -> Color {
        self.sky
    }

    /// Readies the next draw for `eyes`: writes each eye's matrices and
    /// sorts the boxes nearest first into their runs.
    pub fn prepare(&mut self, queue: &Queue, eyes: &[ViewEye; 2]) {
        let mut places = Vec::with_capacity(self.placements.len() * 2 * PLACE_BYTES);

        for eye in eyes {
            for (transform, grid_to_world) in &self.placements {
                let clip = (eye.world_to_clip * *grid_to_world).to_cols_array();
                let ray = grid_point(transform, eye.position).extend(1.0);

                let axes = [TyVector3F64::X, TyVector3F64::Y, TyVector3F64::Z]
                    .map(|axis| (transform.rotation * axis).extend(0.0));

                let floats = clip
                    .into_iter()
                    .chain(ray.to_array())
                    .chain(axes.iter().flat_map(|axis| axis.to_array()));

                for value in floats {
                    places.extend((value as f32).to_le_bytes());
                }
            }
        }

        queue.write_buffer(&self.places, 0, &places);

        let head = (eyes[0].position + eyes[1].position) / 2.0;
        let reach = (eyes[0].position - eyes[1].position).length() / 2.0 + REACH;

        // A solid box shows nothing from inside.
        let mut solid: Vec<_> = self
            .solid
            .iter()
            .map(|(placed, words)| (placed.distance_from(head), words))
            .filter(|&(distance, _)| distance > reach)
            .collect();

        let mut front = Vec::new();
        let mut inside = Vec::new();
        let mut exact = Vec::new();

        for (placed, words, crosses) in &self.marched {
            let distance = placed.distance_from(head);

            if *crosses {
                exact.push(*words);
            } else if distance > reach {
                front.push((distance, words));
            } else {
                inside.push(*words);
            }
        }

        solid.sort_by(|(a, _), (b, _)| a.total_cmp(b));
        front.sort_by(|(a, _), (b, _)| a.total_cmp(b));

        self.boxes.rewrite(
            queue,
            &[
                solid.into_iter().map(|(_, words)| *words).collect(),
                front.into_iter().map(|(_, words)| *words).collect(),
                inside,
                exact,
            ],
        );
    }

    /// Draws the boxes [`prepare`](Self::prepare) sorted into `pass`, a
    /// multiview pass over both eyes.
    pub fn draw(&self, pass: &mut RenderPass<'_>) {
        self.boxes.draw(pass);
    }
}
