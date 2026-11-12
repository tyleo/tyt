use crate::{BoxCrosser, BoxDraw, BoxPart, HitTargets, PlacedBox, Result, TileMasks, ViewEye};
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

/// The bytes at the start of a `Place` that follow the eye: its matrix and
/// its ray.
const EYE_BYTES: usize = 80;

/// The meters past the eyes within which a box counts as holding an eye.
/// The reach covers the near plane.
const REACH: f32 = 0.02;

/// The meters a head moves from where the boxes last sorted before they
/// sort again.
const SLACK: f32 = 0.25;

/// A shaded view of a scene's boxes for both eyes of a headset: the box
/// raster with solid boxes, a mask march, and face depth. Each frame takes
/// its eyes and draws in multiview passes over a two-layer target with
/// depth in [`BoxView::DEPTH`]. The boxes draw nearest first. They sort
/// again once the head has moved [`SLACK`] from where they last sorted.
///
/// A hit takes its cell's base color, the scene's first sun without shadows,
/// and its hemispheres, under the contract's tonemap.
pub struct BoxView {
    boxes: BoxDraw,

    /// Each eye's `Place` for each placement.
    places: Buffer,

    /// The bytes of `places`. A frame rewrites each `Place`'s matrix and
    /// ray.
    place_bytes: Vec<u8>,

    /// Each placement's transform and its grid-to-world matrix.
    placements: Vec<(TyTransformF64, TyMatrix4x4F64)>,

    solid: ViewBoxes,
    marched: ViewBoxes,

    /// Whether each marched box shares space with another placement's box.
    crossing: Vec<bool>,

    /// Where the head stood when the boxes last sorted, and how far from
    /// there a box drew its back faces.
    sorted_for: Option<([f32; 3], f32)>,

    /// The last sort's boxes and keys, kept for their room.
    sorted: Vec<u8>,
    solid_keys: Vec<u64>,
    marched_keys: Vec<u64>,

    sky: Color,
}

/// Boxes as a sort reads them.
#[derive(Default)]
struct ViewBoxes {
    /// The low and then the high corner of the world box around each box.
    bounds: Vec<[f32; 6]>,

    /// Each box's three `vec4<u32>`s.
    words: Vec<[u8; BoxDraw::BOX_BYTES]>,
}

impl ViewBoxes {
    fn push(&mut self, placed: &PlacedBox, words: [u32; 12]) {
        let (low, high) = placed.world();
        let [low_x, low_y, low_z] = low.as_vec3().to_array();
        let [high_x, high_y, high_z] = high.as_vec3().to_array();
        let mut bytes = [0; BoxDraw::BOX_BYTES];

        for (bytes, word) in bytes.chunks_exact_mut(4).zip(words) {
            bytes.copy_from_slice(&word.to_le_bytes());
        }

        self.bounds
            .push([low_x, low_y, low_z, high_x, high_y, high_z]);
        self.words.push(bytes);
    }

    /// Fills `keys` with a key per box, nearest to `point` first, and
    /// returns how many of the boxes lie within `reach` of `point`. A key
    /// holds the bits of its box's squared distance above the box's index.
    fn nearest_first(&self, point: [f32; 3], reach: f32, keys: &mut Vec<u64>) -> usize {
        keys.clear();

        // A square is no less than zero, so its bits order as it does.
        keys.extend((0u64..).zip(&self.bounds).map(|(index, bounds)| {
            let square: f32 = (0..3)
                .map(|axis| {
                    let off = point[axis].clamp(bounds[axis], bounds[axis + 3]) - point[axis];

                    off * off
                })
                .sum();

            u64::from(square.to_bits()) << 32 | index
        }));

        keys.sort_unstable();
        keys.partition_point(|&key| f32::from_bits((key >> 32) as u32) <= reach * reach)
    }

    /// Appends the boxes of `keys` to `sorted` and returns their count.
    fn extend(&self, keys: impl IntoIterator<Item = u64>, sorted: &mut Vec<u8>) -> u32 {
        let mut count = 0;

        for key in keys {
            sorted.extend_from_slice(&self.words[index_of(key)]);
            count += 1;
        }

        count
    }
}

/// The index of the box a sort's `key` holds.
fn index_of(key: u64) -> usize {
    (key & 0xffff_ffff) as usize
}

/// The keys of `keys` whose boxes have `crosses` in `crossing`.
fn crossing_keys<'a>(
    keys: &'a [u64],
    crossing: &'a [bool],
    crosses: bool,
) -> impl Iterator<Item = u64> + 'a {
    keys.iter()
        .copied()
        .filter(move |&key| crossing[index_of(key)] == crosses)
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

        // A box's tile sits past its object's first word.
        let tile = |placed: &PlacedBox, solid: bool| {
            let object_id = object_ids[placed.placement as usize];
            let (tiles, solid_tiles) = &object_tiles[&object_id];

            offsets[&object_id] + if solid { solid_tiles } else { tiles }[placed.brick]
        };

        // Each marched box's entries in `crosser_words`, as their first and
        // their count.
        let crossers = PlacedBox::crossers(&marched, &solid);
        let mut crosser_words: Vec<u32> = Vec::new();

        let march_words: Vec<_> = marched
            .iter()
            .zip(&crossers)
            .map(|(placed, entries)| {
                let count = |entries: usize| {
                    u32::try_from(entries).expect("a scene's crossing boxes number under 2^32")
                };

                let span = [count(crosser_words.len() / 8), count(entries.len())];

                crosser_words.extend(entries.iter().flat_map(BoxCrosser::words));

                placed.words(placed.placement, tile(placed, false), span)
            })
            .collect();

        let solid_words: Vec<_> = solid
            .iter()
            .map(|placed| placed.words(placed.placement, tile(placed, true), [0, 0]))
            .collect();

        // `box_view.wgsl`'s `boxes`: the marched boxes, then the solid ones.
        let mut box_words: Vec<u32> = march_words
            .iter()
            .chain(&solid_words)
            .flat_map(|words| words.iter().copied().take(8))
            .collect();

        let mut solid_boxes = ViewBoxes::default();
        let mut marched_boxes = ViewBoxes::default();

        for (placed, &words) in solid.iter().zip(&solid_words) {
            solid_boxes.push(placed, words);
        }

        for (placed, &words) in marched.iter().zip(&march_words) {
            marched_boxes.push(placed, words);
        }

        let crossing: Vec<bool> = crossers.iter().map(|entries| !entries.is_empty()).collect();

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

        if box_words.is_empty() {
            box_words.resize(8, 0);
        }

        if crosser_words.is_empty() {
            crosser_words.resize(8, 0);
        }

        let init = |label, contents: &[u8], usage| {
            device.create_buffer_init(&BufferInitDescriptor {
                label: Some(label),
                contents,
                usage,
            })
        };

        let words_bytes = |words: &[u32]| -> Vec<u8> {
            words.iter().flat_map(|word| word.to_le_bytes()).collect()
        };

        let frame = init("view frame", &frame, BufferUsages::UNIFORM);
        let bits = init("view bits", &words_bytes(&words), BufferUsages::STORAGE);
        let colors = init("view colors", &colors, BufferUsages::STORAGE);
        let box_list = init(
            "view boxes",
            &words_bytes(&box_words),
            BufferUsages::STORAGE,
        );
        let crosser_list = init(
            "view crossers",
            &words_bytes(&crosser_words),
            BufferUsages::STORAGE,
        );

        let places = device.create_buffer(&BufferDescriptor {
            label: Some("view places"),
            size: (placements.len().max(1) * 2 * PLACE_BYTES) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // A `Place` ends with the world direction of each of its grid's
        // axes and the meters a cell spans along it.
        let mut place_bytes = vec![0; placements.len() * 2 * PLACE_BYTES];
        let each = placements.iter().chain(&placements);

        for (place, (transform, _)) in place_bytes.chunks_exact_mut(PLACE_BYTES).zip(each) {
            let [x, y, z] = transform.scale.to_array();

            let axes = [
                (TyVector3F64::X, x),
                (TyVector3F64::Y, y),
                (TyVector3F64::Z, z),
            ]
            .map(|(axis, meters)| (transform.rotation * axis).extend(meters).to_array());

            for (bytes, value) in place[EYE_BYTES..]
                .chunks_exact_mut(4)
                .zip(axes.as_flattened())
            {
                bytes.copy_from_slice(&(*value as f32).to_le_bytes());
            }
        }

        // SAFETY: a fragment reads only within a tile's words because each
        // step keeps its cell inside the tile's box. Its loop ends within
        // the box's extent. A tile's words hold its cells' materials, which
        // index `colors`. A box's entries in `crossers` index `boxes`. The
        // stages index `places` by an eye and a box's placement.
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

        // The runs start with every marched box in the run that draws its
        // front faces. Each sort then deals the boxes out again.
        let boxes = BoxDraw::new(
            device,
            &module,
            format,
            &[
                BoxPart {
                    vertex: "place_near",
                    fragment: "inside",
                    backs: true,
                    boxes: Vec::new(),
                },
                BoxPart {
                    vertex: "place",
                    fragment: "solid",
                    backs: false,
                    boxes: solid_words,
                },
                BoxPart {
                    vertex: "place",
                    fragment: "front",
                    backs: false,
                    boxes: march_words,
                },
                BoxPart {
                    vertex: "place",
                    fragment: "crossed",
                    backs: false,
                    boxes: Vec::new(),
                },
                BoxPart {
                    vertex: "place",
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
                (ShaderStages::FRAGMENT, storage, &box_list),
                (ShaderStages::FRAGMENT, storage, &crosser_list),
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
            place_bytes,
            placements,
            solid: solid_boxes,
            marched: marched_boxes,
            crossing,
            sorted_for: None,
            sorted: Vec::new(),
            solid_keys: Vec::new(),
            marched_keys: Vec::new(),
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

    /// Readies the next draw for `eyes`: writes each eye's matrices, and
    /// sorts the boxes when the head has left the last sort's reach.
    pub fn prepare(&mut self, queue: &Queue, eyes: &[ViewEye; 2]) {
        let each = eyes.iter().flat_map(|eye| {
            self.placements
                .iter()
                .map(move |placement| (eye, placement))
        });

        for (place, (eye, (transform, grid_to_world))) in
            self.place_bytes.chunks_exact_mut(PLACE_BYTES).zip(each)
        {
            let clip = (eye.world_to_clip * *grid_to_world).to_cols_array();
            let ray = grid_point(transform, eye.position).extend(1.0).to_array();

            for (bytes, value) in place[..EYE_BYTES]
                .chunks_exact_mut(4)
                .zip(clip.into_iter().chain(ray))
            {
                bytes.copy_from_slice(&(value as f32).to_le_bytes());
            }
        }

        queue.write_buffer(&self.places, 0, &self.place_bytes);

        let head = ((eyes[0].position + eyes[1].position) / 2.0)
            .as_vec3()
            .to_array();
        let reach = (eyes[0].position - eyes[1].position).length() as f32 / 2.0 + REACH;

        // A sort holds while each box an eye can enter draws its back faces.
        let holds = self.sorted_for.is_some_and(|(from, covered)| {
            let moved: f32 = (0..3).map(|axis| (head[axis] - from[axis]).powi(2)).sum();

            moved.sqrt() + reach <= covered
        });

        if !holds {
            self.sort(queue, head, reach + SLACK);
        }
    }

    /// Sorts the boxes nearest to `head` first into their runs. A marched
    /// box within `covered` of `head` draws its back faces because an eye
    /// can come to stand inside it before the next sort. Such a box writes
    /// its hits' depth when it crosses another placement's box because only
    /// a front face's fragment steps through the boxes it crosses. Every solid
    /// box draws its front faces because a ray that starts inside one sees
    /// none of it.
    fn sort(&mut self, queue: &Queue, head: [f32; 3], covered: f32) {
        self.solid
            .nearest_first(head, covered, &mut self.solid_keys);

        let within = self
            .marched
            .nearest_first(head, covered, &mut self.marched_keys);
        let (near, far) = self.marched_keys.split_at(within);
        let crossing = &self.crossing;

        self.sorted.clear();

        let counts = [
            self.marched
                .extend(crossing_keys(near, crossing, false), &mut self.sorted),
            self.solid
                .extend(self.solid_keys.iter().copied(), &mut self.sorted),
            self.marched
                .extend(crossing_keys(far, crossing, false), &mut self.sorted),
            self.marched
                .extend(crossing_keys(far, crossing, true), &mut self.sorted),
            self.marched
                .extend(crossing_keys(near, crossing, true), &mut self.sorted),
        ];

        self.boxes.rewrite(queue, &self.sorted, &counts);
        self.sorted_for = Some((head, covered));
    }

    /// How many passes draw the boxes [`prepare`](Self::prepare) sorted, at
    /// least one. Each is a multiview pass over both eyes. The first clears
    /// its target to [`sky`](Self::sky) and its depth to zero, and each
    /// later one loads what the pass before it stored.
    pub fn passes(&self) -> u32 {
        self.boxes.runs().max(1)
    }

    /// Draws the `at`th of the view's [`passes`](Self::passes) into `pass`.
    pub fn draw(&self, pass: &mut RenderPass<'_>, at: u32) {
        self.boxes.draw(pass, at);
    }
}
