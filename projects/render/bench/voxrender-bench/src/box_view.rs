use crate::{
    BoxBindings, BoxBloom, BoxCrosser, BoxDraw, BoxFaces, BoxHide, BoxPart, GlassTiles, LitFill,
    MaterialGrid, PassTerms, PlacedBox, QuadTlas, Result, TileMasks, ViewEye, box_view_shader,
    solid_slabs,
};
use branded_id::U32Id;
use std::{collections::HashMap, mem, num::NonZeroU32};
use ty_math::{
    TyLinSrgbF32, TyLinSrgbF64, TyMatrix4x4F64, TyTransformF64, TyVector3F64, TyVector3U32,
};
use voxcore::material;
use voxrender::{RenderBloom, RenderLight, RenderScene, grid_point, tonemap};
use voxsurface::{SurfaceMethod, mesh_grid_keyed};
use wgpu::{
    AddressMode, Buffer, BufferBindingType, BufferDescriptor, BufferUsages, Color, CommandEncoder,
    Device, Extent3d, Features, FilterMode, LoadOp, Operations, Queue, RenderPass,
    RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor,
    SamplerDescriptor, ShaderModuleDescriptor, ShaderRuntimeChecks, ShaderSource, ShaderStages,
    StoreOp, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView,
    TextureViewDescriptor,
    util::{BufferInitDescriptor, DeviceExt},
};

/// The bytes of one of `box_view.wgsl`'s `Place`s.
const PLACE_BYTES: usize = 128;

/// The bytes at the start of a `Place` that follow the eye: its matrix and
/// its ray.
const EYE_BYTES: usize = 80;

/// The bytes of `box_view.wgsl`'s `Frame`, and where its eyes start.
const FRAME_BYTES: usize = 112;
const EYES_AT: u64 = 80;

/// The standard deviations of the bloom's widest blur that the halo reaches
/// past what glows.
const REACH_SIGMAS: f64 = 3.0;

/// The meters past the eyes within which a box counts as holding an eye,
/// in a headset's meters. The reach covers the near plane.
const REACH: f64 = 0.02;

/// The meters a head moves from where the boxes last sorted before they
/// sort again, in a headset's meters.
const SLACK: f64 = 0.25;

/// The part of a pixel that a hidden box keeps inside a shadow's sides,
/// which covers where the rasterizer rounds a corner to.
const KEPT: f64 = 0.25;

/// The lists of boxes a [`BoxHide`] tests for the view.
const MARCHED: usize = 0;
const SOLID: usize = 1;

/// Where each kind of opaque box starts among a view's runs, and how many
/// runs the kinds take. The held boxes and the inside ones take a run each.
/// Each other kind takes two: its boxes with no mirror, then those with
/// one.
const HELD_RUN: usize = 0;
const INSIDE_RUN: usize = 1;
const SOLID_RUN: usize = 2;
const FRONT_RUN: usize = 4;
const CROSSED_RUN: usize = 6;
const RUNS: usize = 8;

/// The fewest boxes a scene has for its view to test them. On Quest a
/// hidden box saves 66 ns of a frame, so a scene with fewer can hide too
/// few to pay for the test.
const TESTED: usize = 4096;

/// A shaded view of a scene's boxes for both eyes of a headset: the box
/// raster with solid boxes, a mask march, and face depth. Each frame takes
/// its eyes and draws in multiview passes over a two-layer target with
/// depth in [`BoxView::DEPTH`]. The boxes draw nearest first. They sort
/// again once the head has moved [`SLACK`] from where they last sorted. A
/// frame leaves out the boxes its [`BoxHide`] hides from both eyes.
///
/// A hit shades as the contract's opaque surface under the scene's first
/// sun and its hemispheres, with corner occlusion and the sun's corner
/// shadows. On a device with ray queries a face's mirror ray traces the
/// scene's opaque quads. The quad it meets shades flat with no shadow.
/// Elsewhere a face reflects the sky alone.
///
/// Where the scene blooms, a fragment adds the halo at its pixel before the
/// tonemap. [`glow`](Self::glow) makes the halo before a frame's passes, and
/// a last pass adds it over the sky.
///
/// The cells that pass light draw after the opaque ones where the device
/// blends two colors a fragment. A fragment walks its box through the
/// surfaces between such materials. The blend tints what the target holds
/// by what those surfaces pass and adds their light. The boxes draw farthest
/// first as of the last sort. Elsewhere those cells do not draw. The cells
/// tint the sun's light on the faces behind them and on their surfaces.
pub struct BoxView {
    boxes: BoxDraw,

    /// What makes the halo, where an emission passes the bloom's
    /// threshold.
    glow: Option<Glow>,

    /// The target's width and height.
    size: [u32; 2],

    /// `box_view.wgsl`'s `Frame`. A frame rewrites its eyes.
    frame: Buffer,

    /// Each eye's `Place` for each placement.
    places: Buffer,

    /// The bytes of `places`. A frame rewrites each `Place`'s matrix and
    /// ray.
    place_bytes: Vec<u8>,

    /// Each placement's transform and its grid-to-world matrix.
    placements: Vec<(TyTransformF64, TyMatrix4x4F64)>,

    solid: ViewBoxes,
    marched: ViewBoxes,

    /// The boxes around each tile's surfaces between materials that pass
    /// light. They draw after the rest, farthest first, where the device
    /// blends a tint a channel.
    glass: ViewBoxes,

    /// Whether each marched box shares space with another placement's box.
    crossing: Vec<bool>,

    /// The test that keeps hidden boxes from drawing, for a scene of
    /// [`TESTED`] boxes or more with a slab to hide one.
    hide: Option<BoxHide>,

    /// How many of the last sort's marched boxes lie where an eye can come
    /// to stand inside one before the next sort, and how many of its glass
    /// boxes.
    near: usize,
    glass_near: usize,

    /// Where the head stood when the boxes last sorted, and how far from
    /// there a box drew its back faces.
    sorted_for: Option<([f32; 3], f32)>,

    /// Each run's opaque boxes as of the last sort, in the order they
    /// draw.
    order: [Vec<usize>; RUNS],

    /// The last sort's boxes and keys, kept for their room.
    sorted: Vec<u8>,
    solid_keys: Vec<u64>,
    marched_keys: Vec<u64>,
    glass_keys: Vec<u64>,

    /// Room a sort moves its keys through.
    spare_keys: Vec<u64>,

    sky: Color,
}

/// What a scene that blooms draws before a frame's passes.
struct Glow {
    bloom: BoxBloom,

    /// The boxes whose fragments can meet an emission over the threshold,
    /// in the view's runs, drawn through the module that draws the bloom's
    /// source.
    boxes: BoxDraw,

    /// The depth the boxes draw with, at the source's size.
    depth: TextureView,

    /// The pixels of the target the halo reaches past what glows.
    reach: f64,
}

/// Boxes as a sort reads them.
#[derive(Default)]
struct ViewBoxes {
    /// The low and then the high corner of the world box around each box.
    bounds: Vec<[f32; 6]>,

    /// Each box's three `vec4<u32>`s.
    words: Vec<[u8; BoxDraw::BOX_BYTES]>,

    /// Whether each box's fragments can cast a mirror ray.
    mirror: Vec<bool>,

    /// Whether each box's fragments can meet an emission that blooms.
    glow: Vec<bool>,
}

impl ViewBoxes {
    fn push(&mut self, placed: &PlacedBox, words: [u32; 12], mirror: bool, glow: bool) {
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
        self.mirror.push(mirror);
        self.glow.push(glow);
    }

    /// The keys of `keys` whose boxes glow.
    fn glowing<'a>(
        &'a self,
        keys: impl IntoIterator<Item = u64> + 'a,
    ) -> impl Iterator<Item = u64> + 'a {
        keys.into_iter().filter(|&key| self.glow[index_of(key)])
    }

    /// Fills `keys` with a key per box, nearest to `point` first, and
    /// returns how many of the boxes lie within `reach` of `point`. A key
    /// holds the bits of its box's squared distance above the box's index.
    fn nearest_first(
        &self,
        point: [f32; 3],
        reach: f32,
        keys: &mut Vec<u64>,
        spare: &mut Vec<u64>,
    ) -> usize {
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

        sort_keys(keys, spare);
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

/// Sorts `keys` by their upper halves, least first, with `spare`'s room.
/// Keys whose upper halves match keep their order. A moving head sorts a
/// city's 26,000 keys each few frames, and on Quest these passes take
/// 0.4 ms less of such a frame than `sort_unstable` does.
fn sort_keys(keys: &mut Vec<u64>, spare: &mut Vec<u64>) {
    spare.clear();
    spare.resize(keys.len(), 0);

    // Three passes of 11 bits cover an upper half.
    for shift in [32, 43, 54] {
        let digit = |key: u64| (key >> shift) as usize & 0x7ff;
        let mut starts = [0; 0x800];

        for &key in keys.iter() {
            starts[digit(key)] += 1;
        }

        let mut next = 0;

        for start in &mut starts {
            next += mem::replace(start, next);
        }

        for &key in keys.iter() {
            spare[starts[digit(key)]] = key;
            starts[digit(key)] += 1;
        }

        mem::swap(keys, spare);
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
    /// A mirror ray runs to where what it meets would show by this share
    /// of its face's light. A face that reflects less casts none.
    pub const MIRROR_CUT: f32 = 1.0 / 64.0;

    /// The least cells a mirror ray reaches. A face whose ray would end
    /// sooner casts none: what touches it shows too faintly to earn a ray.
    pub const MIRROR_REACH: f32 = 2.0;

    /// The format of the depth a pass that draws the view clears to zero.
    pub const DEPTH: TextureFormat = BoxDraw::DEPTH;

    /// Builds the view of `scene` under `bloom` and submits the fill of the
    /// sun's corner shadows. The view draws to a two-layer target of
    /// `format` whose width and height are `size`. Errors if a tile holds
    /// over 256 materials, or if the scene has a sun and no view.
    pub fn new(
        device: &Device,
        queue: &Queue,
        scene: &RenderScene,
        bloom: RenderBloom,
        format: TextureFormat,
        size: [u32; 2],
    ) -> Result<Self> {
        // The scene blooms where an emission's brightest channel passes
        // the threshold.
        let glows = bloom.strength > 0.0
            && scene.iter_materials().any(|(_, material)| {
                let emission = material.emissive_color * material.emissive_strength;

                emission.red.max(emission.green).max(emission.blue) > bloom.threshold
            });

        let radius = bloom.radius;
        let bloom = glows.then(|| (BoxBloom::new(device, size, bloom), bloom.threshold as f32));

        let mut words = Vec::new();
        let mut offsets = HashMap::new();
        let mut object_boxes = HashMap::new();
        let mut object_solids = HashMap::new();
        let mut object_tiles = HashMap::new();
        let mut object_recorded = HashMap::new();
        let mut object_entries = HashMap::new();

        // A material that passes no light at normal incidence.
        let opaque = |material: u32| {
            scene
                .material(U32Id::from_u32(material))
                .is_some_and(|material| {
                    material::pass(
                        material.base_color,
                        material.metallic,
                        material.transmission,
                        material.ior,
                    ) == TyLinSrgbF64::new(0.0, 0.0, 0.0)
                })
        };

        // The surfaces between materials that pass light draw as glass
        // where the device blends a tint a channel. A cell's word holds
        // where its material's seven values start in `materials`.
        let blends = device.features().contains(Features::DUAL_SOURCE_BLENDING);
        let mut glass_ids: Vec<u32> = Vec::new();
        let mut object_glass = HashMap::new();
        let mut object_glass_cells = HashMap::new();
        let mut glass_tiles = HashMap::new();

        let mut place = |id: u32| {
            let place = glass_ids
                .iter()
                .position(|&placed| placed == id)
                .unwrap_or_else(|| {
                    glass_ids.push(id);

                    glass_ids.len() - 1
                });

            u32::try_from(place * 7)
                .ok()
                .filter(|&place| place < 1 << 12)
                .expect("a scene's glass materials number under 586")
        };

        for (object_id, object) in scene.iter_objects() {
            let masks = TileMasks::new(object, true, Some(&opaque))?;

            offsets.insert(
                object_id,
                u32::try_from(words.len()).expect("a scene's mask words number under 2^32"),
            );

            words.extend(masks.words);

            let glass = if blends {
                GlassTiles::new(object, &opaque, &mut place)
            } else {
                GlassTiles {
                    boxes: Vec::new(),
                    surfaces: Vec::new(),
                    tiles: Vec::new(),
                    cells: Vec::new(),
                    words: Vec::new(),
                }
            };

            let first = u32::try_from(words.len()).expect("a scene's mask words number under 2^32");

            words.extend(glass.words);
            glass_tiles.insert(object_id, (first, glass.tiles, glass.surfaces));

            // Each glass tile's low corner and its cells with a surface.
            object_glass_cells.insert(
                object_id,
                glass
                    .boxes
                    .iter()
                    .map(|(low, _)| *low / 16 * 16)
                    .zip(glass.cells)
                    .collect::<Vec<_>>(),
            );

            object_glass.insert(object_id, glass.boxes);
            object_boxes.insert(object_id, masks.boxes);
            object_solids.insert(object_id, masks.solids);
            object_tiles.insert(object_id, (masks.tiles, masks.solid_tiles));
            object_entries.insert(object_id, masks.entries);
            object_recorded.insert(
                object_id,
                masks
                    .corners
                    .into_iter()
                    .zip(masks.recorded)
                    .collect::<Vec<_>>(),
            );
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
                        sun = (rotation * TyVector3F64::Z, color * strength);
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

        let words_bytes = |words: &[u32]| -> Vec<u8> {
            words.iter().flat_map(|word| word.to_le_bytes()).collect()
        };

        // `box_view.wgsl`'s `lit`: a word for each record cell of each
        // placement's tiles, with the cell's material in its high 8 bits.
        // A hit the depth test drops can read the spare words past its
        // tile's last record cell's.
        let mut lit_places = Vec::new();
        let mut lit_words: Vec<u32> = Vec::new();

        for object_id in &object_ids {
            lit_places.push(
                u32::try_from(lit_words.len()).expect("a scene's lit words number under 2^32"),
            );

            lit_words.extend(
                object_entries[object_id]
                    .iter()
                    .flatten()
                    .map(|&entry| u32::from(entry) << 24),
            );
        }

        // A word for each glass cell follows: the first of each glass
        // tile's, for each placement.
        let glass_lit: Vec<Vec<u32>> = object_ids
            .iter()
            .map(|object_id| {
                object_glass_cells[object_id]
                    .iter()
                    .map(|(_, cells)| {
                        let first = u32::try_from(lit_words.len())
                            .expect("a scene's lit words number under 2^32");

                        lit_words.resize(lit_words.len() + cells.len(), 0);

                        first
                    })
                    .collect()
            })
            .collect();

        lit_words.extend([0; TileMasks::SPARE as usize]);

        // The sun's corner shadows, which the GPU fills into the words' low
        // bits before the view's first draw. A scene with no sun leaves them
        // zero, and its faces skip the sun's term.
        // Where a material passes light, the fill adds each cell's tint.
        let (lit_corners, tints) = if suns > 0 {
            let fill = LitFill::new(
                device,
                scene,
                sun.0,
                &object_recorded,
                &object_glass_cells,
                &opaque,
            )?;

            let glass_places: Vec<_> = glass_lit
                .iter()
                .zip(&fill.glass_places)
                .map(|(tiles, &place)| tiles.first().copied().unwrap_or(place))
                .collect();

            assert_eq!(
                (&fill.places, &fill.glass_places),
                (&lit_places, &glass_places),
                "the fill lays out the view's words"
            );

            queue.write_buffer(&fill.lit, 0, &words_bytes(&lit_words));
            fill.fill(device, queue);

            (fill.lit, fill.tints.unwrap_or(0))
        } else {
            let lit = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("view lit corners"),
                contents: &words_bytes(&lit_words),
                usage: BufferUsages::STORAGE,
            });

            (lit, 0)
        };

        // A material's three `vec4<f32>`s: its base color and metalness, its
        // emission and roughness, and its reflectance at normal incidence
        // and occlusion strength. An id the scene lacks holds zeros.
        let material_values = |id: u32| {
            scene
                .material(U32Id::from_u32(id))
                .map_or([0.0; 12], |material| {
                    let base = material.base_color.color;
                    let emission = material.emissive_color * material.emissive_strength;
                    let reflectance =
                        material::normal_reflectance(base, material.metallic, material.ior);

                    [
                        base.red,
                        base.green,
                        base.blue,
                        material.metallic,
                        emission.red,
                        emission.green,
                        emission.blue,
                        material.roughness,
                        reflectance.red,
                        reflectance.green,
                        reflectance.blue,
                        material.occlusion_strength,
                    ]
                    .map(|value| value as f32)
                })
        };

        // Each tile's first material in `materials`. The tile's materials
        // follow one another there, so a cell's place among them leads to
        // its material's values in one read. Tiles that hold the same
        // materials in the same order share them.
        // A glass material holds the seven `vec4<f32>`s `box_glass.wgsl`'s
        // `past` lists.
        // An id the scene lacks passes the light whole.
        let mut absent = [0.0; 28];

        absent[..3].fill(1.0);

        let glass_values = |id: u32| {
            scene
                .material(U32Id::from_u32(id))
                .map_or(absent, |material| {
                    let terms = PassTerms::new(material);
                    let white = TyLinSrgbF64::new(1.0, 1.0, 1.0);
                    let black = TyLinSrgbF64::new(0.0, 0.0, 0.0);
                    let alpha = (material.roughness * material.roughness).max(1.0e-3);
                    let emission = material.emissive_color * material.emissive_strength;
                    let transmitted = material.transmission * (1.0 - material.metallic);
                    let diffuse = material.base_color.color
                        * ((1.0 - material.metallic) * (1.0 - material.transmission));
                    let rise = terms.rise * transmitted;

                    // A reflectance with no hue fits one channel.
                    let gloss = terms.reflectance;
                    let plain = gloss.red == gloss.green && gloss.red == gloss.blue;

                    let follows = if !plain || emission != black {
                        2.0
                    } else if diffuse != black {
                        1.0
                    } else {
                        0.0
                    };

                    let colors = [
                        (
                            white * terms.uncovered + terms.most * (white - gloss),
                            material.base_color.alpha,
                        ),
                        (terms.most * terms.rise, 0.5 / alpha),
                        (
                            TyLinSrgbF64::new(gloss.red, rise.red, alpha * alpha),
                            follows,
                        ),
                        (diffuse, 0.0),
                        (gloss, 0.0),
                        (rise, 0.0),
                        (emission, 0.0),
                    ];

                    let mut values = [0.0; 28];

                    for (values, (color, last)) in values.chunks_exact_mut(4).zip(colors) {
                        values.copy_from_slice(
                            &[color.red, color.green, color.blue, last].map(|value| value as f32),
                        );
                    }

                    values
                })
        };

        // The glass materials come first, where a cell's word reaches them
        // in 12 bits. A tile's first material counts in threes after them.
        let mut materials: Vec<f32> = glass_ids.iter().flat_map(|&id| glass_values(id)).collect();

        materials.resize(materials.len().next_multiple_of(12), 0.0);

        let mut firsts: HashMap<Vec<u32>, u32> = HashMap::new();
        let mut tile_firsts: HashMap<u32, u32> = HashMap::new();

        let mut first_material = |tile: u32| {
            *tile_firsts.entry(tile).or_insert_with(|| {
                let palette = words[(tile + TileMasks::PALETTE) as usize];
                let ids = (tile + (palette & 0xffff)) as usize;
                let ids = words[ids..ids + (palette >> 16) as usize].to_vec();

                *firsts.entry(ids).or_insert_with_key(|ids| {
                    let first = materials.len() / 12;

                    materials.extend(ids.iter().flat_map(|&id| material_values(id)));

                    u32::try_from(first).expect("a scene's tiles' materials number under 2^32")
                })
            })
        };

        // A mirror ray needs a device with ray queries. A tile's fragments
        // can cast one where one of its materials' rays would reach past
        // `MIRROR_REACH`.
        let traced = device.features().contains(Features::EXPERIMENTAL_RAY_QUERY);
        let mut tile_mirrors: HashMap<u32, bool> = HashMap::new();

        let mut mirror_tile = |tile: u32| {
            *tile_mirrors.entry(tile).or_insert_with(|| {
                let palette = words[(tile + TileMasks::PALETTE) as usize];
                let ids = (tile + (palette & 0xffff)) as usize;

                traced
                    && words[ids..ids + (palette >> 16) as usize]
                        .iter()
                        .any(|&id| {
                            let values = material_values(id);
                            let alpha = (values[7] * values[7]).max(1.0e-3);
                            let reflectance = values[8].max(values[9]).max(values[10]);
                            let far =
                                (reflectance / Self::MIRROR_CUT - 1.0).max(0.0).sqrt() / alpha;

                            values[7] < 1.0 && far > Self::MIRROR_REACH
                        })
            })
        };

        // A tile glows where one of its materials' emission blooms.
        let mut tile_glows: HashMap<u32, bool> = HashMap::new();

        let mut glow_tile = |tile: u32| {
            *tile_glows.entry(tile).or_insert_with(|| {
                let palette = words[(tile + TileMasks::PALETTE) as usize];
                let ids = (tile + (palette & 0xffff)) as usize;

                bloom.as_ref().is_some_and(|&(_, threshold)| {
                    words[ids..ids + (palette >> 16) as usize]
                        .iter()
                        .any(|&id| {
                            let values = material_values(id);

                            values[4].max(values[5]).max(values[6]) > threshold
                        })
                })
            })
        };

        // What a box's fragments read of its tile with no load: its first
        // material and the first of its lit words.
        let mut shading = |placed: &PlacedBox, tile: u32| {
            let first = lit_places[placed.placement as usize];

            [
                first_material(tile),
                first + words[(tile + TileMasks::FIRST) as usize],
            ]
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

                let tile = tile(placed, false);
                let mut words = placed.words(placed.placement, tile, span);

                words[10..].copy_from_slice(&shading(placed, tile));

                words
            })
            .collect();

        let solid_words: Vec<_> = solid
            .iter()
            .map(|placed| {
                let tile = tile(placed, true);
                let mut words = placed.words(placed.placement, tile, [0, 0]);

                words[10..].copy_from_slice(&shading(placed, tile));

                words
            })
            .collect();

        // `box_view.wgsl`'s `boxes`: the marched boxes, then the solid ones.
        let mut box_words: Vec<u32> = march_words
            .iter()
            .chain(&solid_words)
            .flatten()
            .copied()
            .collect();

        let solid_mirrors: Vec<_> = solid
            .iter()
            .map(|placed| mirror_tile(tile(placed, true)))
            .collect();

        // A marched box's fragment can also shade a hit in a box it
        // crosses.
        let march_mirrors: Vec<_> = marched
            .iter()
            .zip(&crossers)
            .map(|(placed, entries)| {
                mirror_tile(tile(placed, false))
                    || entries.iter().any(|entry| {
                        let other = (entry.entry & !BoxCrosser::SOLID) as usize;

                        match other.checked_sub(marched.len()) {
                            Some(other) => solid_mirrors[other],
                            None => mirror_tile(tile(&marched[other], false)),
                        }
                    })
            })
            .collect();

        // The boxes an eye can stand in take the mirror's module wherever
        // the scene holds a mirror.
        let mirrors = solid_mirrors
            .iter()
            .chain(&march_mirrors)
            .any(|&mirror| mirror);

        let solid_glows: Vec<_> = solid
            .iter()
            .map(|placed| glow_tile(tile(placed, true)))
            .collect();

        let march_glows: Vec<_> = marched
            .iter()
            .zip(&crossers)
            .map(|(placed, entries)| {
                glow_tile(tile(placed, false))
                    || entries.iter().any(|entry| {
                        let other = (entry.entry & !BoxCrosser::SOLID) as usize;

                        match other.checked_sub(marched.len()) {
                            Some(other) => solid_glows[other],
                            None => glow_tile(tile(&marched[other], false)),
                        }
                    })
            })
            .collect();

        let mut solid_boxes = ViewBoxes::default();
        let mut marched_boxes = ViewBoxes::default();

        for (index, (placed, &words)) in solid.iter().zip(&solid_words).enumerate() {
            solid_boxes.push(placed, words, solid_mirrors[index], solid_glows[index]);
        }

        for (index, (placed, &words)) in marched.iter().zip(&march_words).enumerate() {
            marched_boxes.push(placed, words, march_mirrors[index], march_glows[index]);
        }

        let crossing: Vec<bool> = crossers.iter().map(|entries| !entries.is_empty()).collect();

        let hide = (marched.len() + solid.len() >= TESTED)
            .then(|| {
                let object_slabs: HashMap<_, _> = scene
                    .iter_objects()
                    .map(|(object_id, object)| (object_id, solid_slabs(object, &opaque)))
                    .collect();

                // The corners of each placement's slabs in the world. Bit
                // `a` of a corner's index picks the slab's far side on axis
                // `a`.
                scene
                    .iter_placements()
                    .flat_map(|(_, placement)| {
                        object_slabs[&placement.object_id]
                            .iter()
                            .map(|&(low, high)| {
                                [0, 1, 2, 3, 4, 5, 6, 7].map(|corner: u32| {
                                    let far =
                                        TyVector3U32::new(corner & 1, corner >> 1 & 1, corner >> 2);

                                    placement
                                        .transform
                                        .transform_point((low + (high - low) * far).as_dvec3())
                                })
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>()
            })
            .filter(|slabs| !slabs.is_empty())
            .map(|slabs| {
                // A list's boxes come a placement at a time.
                let runs = |placed: &[PlacedBox]| {
                    placed
                        .chunk_by(|a, b| a.placement == b.placement)
                        .map(<[PlacedBox]>::len)
                        .collect()
                };

                BoxHide::new(
                    slabs,
                    vec![
                        (marched_boxes.bounds.clone(), runs(&marched)),
                        (solid_boxes.bounds.clone(), runs(&solid)),
                    ],
                )
            });

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

        // The placements' count, then what a pixel spans of the target.
        for value in [placement_count, 0] {
            frame.extend(value.to_le_bytes());
        }

        for side in size {
            frame.extend((1.0 / side as f32).to_le_bytes());
        }

        // Each eye in the world, which a frame writes.
        frame.resize(FRAME_BYTES, 0);

        if words.is_empty() {
            words.push(0);
        }

        if box_words.is_empty() {
            box_words.resize(12, 0);
        }

        // What a mirror ray traces, on a device with ray queries: the
        // scene's opaque quads, each of one material, and each quad's
        // face under its material's place in `materials`.
        let mut quad_words: Vec<u32> = Vec::new();

        // The bloom's source casts a ray to each glowing hit too.
        let tlas = (mirrors || traced && bloom.is_some()).then(|| {
            let quads = QuadTlas::new(
                device,
                queue,
                scene,
                |object| {
                    // A quad shades as one material, so a run merges the
                    // cells of one material alone.
                    mesh_grid_keyed(
                        &MaterialGrid {
                            object,
                            opaque: &opaque,
                        },
                        SurfaceMethod::Greedy,
                        &|cell| {
                            object
                                .voxel_material(cell)
                                .expect("a grid's cell holds a material")
                                .to_u32()
                        },
                        &|_| true,
                    )
                },
                |_, first_quad| first_quad,
            );

            assert!(
                quads.faces.len() < 1 << 24,
                "a scene's quads number under 2^24"
            );

            // Each object's quads follow one another, as their cells do.
            let mut objects: Vec<_> = quads.first_quads.iter().collect();

            objects.sort_by_key(|&(_, &first)| first);

            let mut places: HashMap<u32, u32> = HashMap::new();
            let mut cells = quads.cells.iter();

            for (at, (object_id, _)) in objects.iter().enumerate() {
                let object = scene
                    .object(**object_id)
                    .expect("the object is the scene's");

                let end = objects
                    .get(at + 1)
                    .map_or(quads.faces.len(), |&(_, &first)| first as usize);

                while quad_words.len() < end {
                    let cell = *cells.next().expect("each quad covers a cell");

                    let id = object
                        .voxel_material(cell)
                        .expect("a quad's cell holds a material")
                        .to_u32();

                    let place = *places.entry(id).or_insert_with(|| {
                        let place = materials.len() / 12;

                        materials.extend(material_values(id));

                        u32::try_from(place)
                            .ok()
                            .filter(|&place| place < 1 << 29)
                            .expect("a scene's materials number under 2^29")
                    });

                    quad_words.push(place << 3 | quads.faces[quad_words.len()]);
                }
            }

            quads.tlas
        });

        if quad_words.is_empty() {
            quad_words.push(0);
        }

        let glass = PlacedBox::list(scene, &object_glass);
        let mut glass_boxes = ViewBoxes::default();

        let glass_words: Vec<_> = glass
            .iter()
            .map(|placed| {
                let (first, tiles, surfaces) = &glass_tiles[&object_ids[placed.placement as usize]];

                // The box around the tile's surfaces, 4 bits a side: its
                // low cell in the tile, then its high cell.
                let (low, high) = surfaces[placed.brick];
                let [x, y, z] = (low % 16).to_array();
                let [far_x, far_y, far_z] = ((high - 1) % 16).to_array();
                let surfaces = x | y << 4 | z << 8 | far_x << 12 | far_y << 16 | far_z << 20;

                let mut words =
                    placed.words(placed.placement, first + tiles[placed.brick], [surfaces, 0]);

                // The first of the tile's words in `lit`.
                words[11] = glass_lit[placed.placement as usize][placed.brick];

                glass_boxes.push(placed, words, false, false);

                words
            })
            .collect();

        // A hit in a row with no record cell can read another tile's word
        // in `lit`, whose material lies up to 255 past the hit tile's first.
        materials.resize(materials.len() + 255 * 12, 0.0);

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

        let material_bytes: Vec<u8> = materials
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect();

        let frame = init(
            "view frame",
            &frame,
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );
        let quad_list = init(
            "view quads",
            &words_bytes(&quad_words),
            BufferUsages::STORAGE,
        );
        let bits = init("view bits", &words_bytes(&words), BufferUsages::STORAGE);
        let materials = init("view materials", &material_bytes, BufferUsages::STORAGE);
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

        // A `Place` ends with what holds from frame to frame: the world
        // direction of each of its grid's axes with the meters a cell spans
        // along it.
        let mut place_bytes = vec![0; placements.len() * 2 * PLACE_BYTES];
        let each = (0..placements.len()).chain(0..placements.len());

        for (place, index) in place_bytes.chunks_exact_mut(PLACE_BYTES).zip(each) {
            let (transform, _) = &placements[index];
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
        // step keeps its cell inside the tile's box. Its loop ends within the
        // box's extent. A cell's place among its tile's record cells ends
        // within the tile's spare records. Its record then lies in the tile's
        // words, and its word in `lit` lies within the spare words that
        // buffer ends with. That word's material is under 256 and indexes
        // `materials` within the spare values that buffer ends with. A box's
        // entries in `crossers` index `boxes`. The stages index `places` by
        // an eye and a box's placement.
        let threshold = bloom.as_ref().map(|&(_, threshold)| threshold);

        let module = |traced, source, glass| unsafe {
            device.create_shader_module_trusted(
                ShaderModuleDescriptor {
                    label: Some(if traced {
                        "box view mirrors"
                    } else {
                        "box view"
                    }),
                    source: ShaderSource::Wgsl(
                        box_view_shader(traced, threshold, source, glass, tints).into(),
                    ),
                },
                ShaderRuntimeChecks::unchecked(),
            )
        };

        // A mirror ray's query costs each fragment of its module, so only
        // the boxes that can cast one draw through it. Glass draws through
        // a module of its own, which a device needs dual-source blending
        // to take.
        let plain = module(false, None, false);
        let mirror = mirrors.then(|| module(true, None, false));
        let glassy = (!glass_words.is_empty()).then(|| module(false, None, true));
        let near = usize::from(mirror.is_some());
        let modules: Vec<_> = [Some(&plain), mirror.as_ref(), glassy.as_ref()]
            .into_iter()
            .flatten()
            .collect();

        let storage = BufferBindingType::Storage { read_only: true };
        let both = ShaderStages::VERTEX | ShaderStages::FRAGMENT;

        let texture = |label, size: [u32; 2], format, usage| {
            device
                .create_texture(&TextureDescriptor {
                    label: Some(label),
                    size: Extent3d {
                        width: size[0],
                        height: size[1],
                        depth_or_array_layers: 2,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: TextureDimension::D2,
                    format,
                    usage,
                    view_formats: &[],
                })
                .create_view(&TextureViewDescriptor::default())
        };

        // A texel no fragment reads, bound where a draw has no halo to
        // add: by a scene with no bloom, and by the draw that makes the
        // halo.
        let no_halo = texture(
            "no halo",
            [1, 1],
            BoxBloom::FORMAT,
            TextureUsages::TEXTURE_BINDING,
        );

        let halo = bloom
            .as_ref()
            .map_or(&no_halo, |(bloom, _)| bloom.halo())
            .clone();

        let bloom_sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("view bloom"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            ..SamplerDescriptor::default()
        });

        // The runs start with every marched box in the run that draws its
        // front faces. Each sort then deals the boxes out again. A box that
        // crosses another draws in two runs when it lies near the head, so
        // the first run starts with room for each.
        let held_words: Vec<_> = march_words
            .iter()
            .zip(&crossers)
            .filter(|(_, entries)| !entries.is_empty())
            .map(|(words, _)| *words)
            .collect();

        // The same runs of the boxes that glow.
        let glowing = |words: &[[u32; 12]], glows: &[bool]| -> Vec<_> {
            words
                .iter()
                .zip(glows)
                .filter(|&(_, &glows)| glows)
                .map(|(words, _)| *words)
                .collect()
        };

        let glow_solids = glowing(&solid_words, &solid_glows);
        let glow_marched = glowing(&march_words, &march_glows);

        let glow_held: Vec<_> = march_words
            .iter()
            .zip(&crossers)
            .zip(&march_glows)
            .filter(|&((_, entries), &glows)| glows && !entries.is_empty())
            .map(|((words, _), _)| *words)
            .collect();

        let part = |fragment, faces, module, boxes| BoxPart {
            fragment,
            faces,
            module,
            boxes,
            glass: false,
        };

        let mut parts = vec![
            part("held", BoxFaces::Held, near, held_words),
            part("inside", BoxFaces::NearBack, near, Vec::new()),
            part("solid", BoxFaces::Front, 0, solid_words),
            part("solid", BoxFaces::Front, near, Vec::new()),
            part("front", BoxFaces::Front, 0, march_words),
            part("front", BoxFaces::Front, near, Vec::new()),
            part("crossed", BoxFaces::Front, 0, Vec::new()),
            part("crossed", BoxFaces::Front, near, Vec::new()),
        ];

        // Glass draws last. A box near the head draws in both runs, and
        // each eye takes the one that fits it.
        if glassy.is_some() {
            parts.extend(
                [("glass", BoxFaces::Front), ("glass_held", BoxFaces::Held)].map(
                    |(fragment, faces)| BoxPart {
                        fragment,
                        faces,
                        module: modules.len() - 1,
                        boxes: glass_words.clone(),
                        glass: true,
                    },
                ),
            );
        }

        let buffers = [
            (both, BufferBindingType::Uniform, &frame),
            (both, storage, &places),
            (ShaderStages::FRAGMENT, storage, &bits),
            (ShaderStages::FRAGMENT, storage, &materials),
            (ShaderStages::FRAGMENT, storage, &box_list),
            (ShaderStages::FRAGMENT, storage, &crosser_list),
            (ShaderStages::FRAGMENT, storage, &lit_corners),
            (ShaderStages::FRAGMENT, storage, &quad_list),
        ];

        let mut boxes = BoxDraw::new(
            device,
            &modules,
            format,
            &parts,
            &BoxBindings {
                buffers: &buffers,
                sampled: Some((&halo, &bloom_sampler)),
                tlas: tlas.as_ref(),
            },
            NonZeroU32::new(0b11),
        );

        let glow = bloom.map(|(bloom, _)| {
            boxes.cover(
                device,
                &plain,
                "place_sky",
                "sky_halo",
                format,
                NonZeroU32::new(0b11),
            );

            // The source's ray needs the ray queries the TLAS came with.
            let source = module(tlas.is_some(), Some(bloom.scale() as f32), false);
            let half = size.map(|side| side.div_ceil(2));

            Glow {
                boxes: BoxDraw::new(
                    device,
                    &[&source],
                    BoxBloom::FORMAT,
                    &[
                        part("held", BoxFaces::Held, 0, glow_held),
                        part("inside", BoxFaces::NearBack, 0, Vec::new()),
                        part("solid", BoxFaces::Front, 0, glow_solids),
                        part("front", BoxFaces::Front, 0, glow_marched),
                        part("crossed", BoxFaces::Front, 0, Vec::new()),
                    ],
                    &BoxBindings {
                        buffers: &buffers,
                        sampled: Some((&no_halo, &bloom_sampler)),
                        tlas: tlas.as_ref(),
                    },
                    NonZeroU32::new(0b11),
                ),
                depth: texture(
                    "glow depth",
                    half,
                    Self::DEPTH,
                    TextureUsages::RENDER_ATTACHMENT,
                ),
                reach: REACH_SIGMAS * radius * f64::from(size[0].min(size[1])),
                bloom,
            }
        });

        let above = tonemap(TyLinSrgbF32::new(
            sky.red as f32,
            sky.green as f32,
            sky.blue as f32,
        ));

        Ok(BoxView {
            boxes,
            glow,
            size,
            frame,
            places,
            place_bytes,
            placements,
            solid: solid_boxes,
            marched: marched_boxes,
            glass: glass_boxes,
            crossing,
            hide,
            near: 0,
            glass_near: 0,
            sorted_for: None,
            order: Default::default(),
            sorted: Vec::new(),
            solid_keys: Vec::new(),
            marched_keys: Vec::new(),
            glass_keys: Vec::new(),
            spare_keys: Vec::new(),
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

    /// Readies the next draw for `eyes`. The boxes sort again when the head
    /// has left the last sort's reach, and deal out again when the test of
    /// hidden boxes changes its answer for one. A meter of the headset's
    /// spans `scale` of the scene's meters.
    pub fn prepare(&mut self, queue: &Queue, eyes: &[ViewEye; 2], scale: f64) {
        let each = eyes.iter().flat_map(|eye| {
            self.placements
                .iter()
                .map(move |placement| (eye, placement))
        });

        for (place, (eye, (transform, grid_to_world))) in
            self.place_bytes.chunks_exact_mut(PLACE_BYTES).zip(each)
        {
            let clip = (eye.world_to_clip * *grid_to_world).to_cols_array();

            // A box holds an eye within `REACH` of it, in the grid's cells
            // along its finest axis.
            let cells = REACH * scale / transform.scale.abs().min_element();
            let ray = grid_point(transform, eye.position).extend(cells).to_array();

            for (bytes, value) in place[..EYE_BYTES]
                .chunks_exact_mut(4)
                .zip(clip.into_iter().chain(ray))
            {
                bytes.copy_from_slice(&(value as f32).to_le_bytes());
            }
        }

        queue.write_buffer(&self.places, 0, &self.place_bytes);

        let eye_bytes: Vec<u8> = eyes
            .iter()
            .flat_map(|eye| eye.position.as_vec3().extend(0.0).to_array())
            .flat_map(f32::to_le_bytes)
            .collect();

        queue.write_buffer(&self.frame, EYES_AT, &eye_bytes);

        let head = ((eyes[0].position + eyes[1].position) / 2.0)
            .as_vec3()
            .to_array();
        let reach = ((eyes[0].position - eyes[1].position).length() / 2.0 + REACH * scale) as f32;
        let slack = (SLACK * scale) as f32;

        // A sort holds while each box an eye can enter draws its back faces.
        let holds = self.sorted_for.is_some_and(|(from, covered)| {
            let moved: f32 = (0..3).map(|axis| (head[axis] - from[axis]).powi(2)).sum();

            moved.sqrt() + reach <= covered
        });

        if !holds {
            self.sort(queue, head, reach + slack);
        }

        // The boxes deal out again when a sort moved them or the test
        // changed its answer for one.
        let mut deals = !holds;

        if let Some(hide) = &mut self.hide {
            let positions = [eyes[0].position, eyes[1].position];

            // The shadows hold while the sort does.
            if !holds {
                hide.aim((positions[0] + positions[1]) / 2.0);
            }

            // The widest angle a pixel spans: the one on an eye's axis. A
            // clip matrix's first two rows scale by two over the tangents
            // the target spans.
            let pixel = eyes
                .iter()
                .flat_map(|eye| {
                    let clip = eye.world_to_clip.to_cols_array();

                    [0, 1].map(|row| {
                        let scale: f64 = (0..3).map(|column| clip[column * 4 + row].powi(2)).sum();

                        2.0 / (scale.sqrt() * f64::from(self.size[row]))
                    })
                })
                .fold(0.0, f64::max);

            deals |= hide.test(positions, pixel * KEPT);
        }

        if deals {
            self.deal(queue);
        }

        if let Some(reach) = self.glow.as_ref().map(|glow| glow.reach) {
            let pixels = self.halo_pixels(eyes, reach);

            self.boxes.cover_within(pixels);
        }
    }

    /// Sorts the boxes nearest to `head` first and lists each run's opaque
    /// boxes in that order. A marched box within `covered` of `head` counts
    /// as near, because an eye can come to stand inside it before the next
    /// sort. A near marched box draws its back faces. Such a box that
    /// crosses another placement's box draws its back faces at the nearest
    /// depth for an eye it holds, and its front faces for one it does not.
    /// Every solid box draws its front faces because a ray that starts
    /// inside one sees none of it.
    fn sort(&mut self, queue: &Queue, head: [f32; 3], covered: f32) {
        let spare = &mut self.spare_keys;

        self.solid
            .nearest_first(head, covered, &mut self.solid_keys, spare);

        self.near = self
            .marched
            .nearest_first(head, covered, &mut self.marched_keys, spare);

        if !self.glass.words.is_empty() {
            self.glass_near = self
                .glass
                .nearest_first(head, covered, &mut self.glass_keys, spare);
        }

        self.sorted_for = Some((head, covered));

        for run in &mut self.order {
            run.clear();
        }

        for (at, &key) in self.marched_keys.iter().enumerate() {
            let index = index_of(key);
            let near = at < self.near;
            let mirror = usize::from(self.marched.mirror[index]);

            if self.crossing[index] {
                if near {
                    self.order[HELD_RUN].push(index);
                }

                self.order[CROSSED_RUN + mirror].push(index);
            } else if near {
                self.order[INSIDE_RUN].push(index);
            } else {
                self.order[FRONT_RUN + mirror].push(index);
            }
        }

        for &key in &self.solid_keys {
            let index = index_of(key);
            let mirror = usize::from(self.solid.mirror[index]);

            self.order[SOLID_RUN + mirror].push(index);
        }

        // The glowing boxes take the view's runs for the halo's draw.
        if let Some(glow) = &mut self.glow {
            let (near, far) = self.marched_keys.split_at(self.near);
            let crossing = &self.crossing;
            let solid = &self.solid;
            let marched = &self.marched;

            self.sorted.clear();

            let counts = [
                marched.extend(
                    marched.glowing(crossing_keys(near, crossing, true)),
                    &mut self.sorted,
                ),
                marched.extend(
                    marched.glowing(crossing_keys(near, crossing, false)),
                    &mut self.sorted,
                ),
                solid.extend(
                    solid.glowing(self.solid_keys.iter().copied()),
                    &mut self.sorted,
                ),
                marched.extend(
                    marched.glowing(crossing_keys(far, crossing, false)),
                    &mut self.sorted,
                ),
                marched.extend(
                    marched.glowing(crossing_keys(&self.marched_keys, crossing, true)),
                    &mut self.sorted,
                ),
            ];

            glow.boxes.rewrite(queue, &self.sorted, &counts);
        }
    }

    /// Deals the sorted boxes into their runs. A run of front faces leaves
    /// out the boxes the test hides.
    fn deal(&mut self, queue: &Queue) {
        self.sorted.clear();

        let mut counts = Vec::with_capacity(RUNS + 2);

        for (run, order) in self.order.iter().enumerate() {
            let (boxes, list) = if (SOLID_RUN..FRONT_RUN).contains(&run) {
                (&self.solid, SOLID)
            } else {
                (&self.marched, MARCHED)
            };

            // The runs from the solid ones on draw front faces.
            let hide = self.hide.as_ref().filter(|_| run >= SOLID_RUN);
            let before = self.sorted.len();

            for &index in order {
                if !hide.is_some_and(|hide| hide.hides(list, index)) {
                    self.sorted.extend_from_slice(&boxes.words[index]);
                }
            }

            let count = (self.sorted.len() - before) / BoxDraw::BOX_BYTES;

            counts.push(u32::try_from(count).expect("a scene's boxes number under 2^32"));
        }

        // Glass draws farthest first: every box through its front faces,
        // then the boxes an eye can stand in through their back faces.
        if !self.glass.words.is_empty() {
            let glass = &self.glass;
            let farthest_first = |keys: &[u64]| keys.iter().rev().copied().collect::<Vec<_>>();

            counts.push(glass.extend(farthest_first(&self.glass_keys), &mut self.sorted));
            counts.push(glass.extend(
                farthest_first(&self.glass_keys[..self.glass_near]),
                &mut self.sorted,
            ));
        }

        self.boxes.rewrite(queue, &self.sorted, &counts);
    }

    /// How many passes draw the boxes [`prepare`](Self::prepare) sorted, at
    /// least one. Each is a multiview pass over both eyes. The first clears
    /// its target to [`sky`](Self::sky) and its depth to zero, and each
    /// later one loads what the pass before it stored.
    pub fn passes(&self) -> u32 {
        self.boxes.passes()
    }

    /// Draws the `at`th of the view's [`passes`](Self::passes) into `pass`.
    pub fn draw(&self, pass: &mut RenderPass<'_>, at: u32) {
        self.boxes.draw(pass, at);
    }

    /// Encodes the passes that make a frame's halo, which come before the
    /// first of its [`passes`](Self::passes): the glowing boxes drawn into
    /// the bloom's source, then the bloom's. Encodes nothing for a scene
    /// with no bloom.
    pub fn glow(&self, encoder: &mut CommandEncoder) {
        let Some(glow) = &self.glow else {
            return;
        };

        let passes = glow.boxes.passes();

        for at in 0..passes {
            let first = at == 0;

            let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("glow"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: glow.bloom.source(),
                    depth_slice: None,
                    resolve_target: None,
                    ops: Operations {
                        load: if first {
                            LoadOp::Clear(Color::BLACK)
                        } else {
                            LoadOp::Load
                        },
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &glow.depth,
                    depth_ops: Some(Operations {
                        load: if first {
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
                multiview_mask: NonZeroU32::new(0b11),
                ..RenderPassDescriptor::default()
            });

            glow.boxes.draw(&mut pass, at);
        }

        glow.bloom.run(encoder);
    }

    /// The pixels of the target the halo can reach for `eyes`, as a
    /// rectangle's left, top, width, and height: what glows in either eye,
    /// grown by `reach` pixels. The rectangle covers the whole target where a
    /// glowing box reaches behind an eye.
    fn halo_pixels(&self, eyes: &[ViewEye; 2], reach: f64) -> [u32; 4] {
        let [width, height] = self.size.map(f64::from);
        let mut low = [f64::INFINITY; 2];
        let mut high = [f64::NEG_INFINITY; 2];

        let glowing = [&self.solid, &self.marched].into_iter().flat_map(|boxes| {
            boxes
                .bounds
                .iter()
                .zip(&boxes.glow)
                .filter(|&(_, &glows)| glows)
                .map(|(bounds, _)| bounds)
        });

        let clips = eyes.map(|eye| eye.world_to_clip.to_cols_array());

        for bounds in glowing {
            for clip in &clips {
                for corner in 0..8 {
                    let [x, y, z] =
                        [0, 1, 2].map(|axis| f64::from(bounds[axis + 3 * (corner >> axis & 1)]));
                    let along = |row: usize| {
                        clip[row] * x + clip[4 + row] * y + clip[8 + row] * z + clip[12 + row]
                    };
                    let w = along(3);

                    if w <= 0.0 {
                        return [0, 0, self.size[0], self.size[1]];
                    }

                    let pixel = [
                        (along(0) / w * 0.5 + 0.5) * width,
                        (0.5 - along(1) / w * 0.5) * height,
                    ];

                    for axis in 0..2 {
                        low[axis] = low[axis].min(pixel[axis] - reach);
                        high[axis] = high[axis].max(pixel[axis] + reach);
                    }
                }
            }
        }

        let left = low[0].clamp(0.0, width).floor();
        let top = low[1].clamp(0.0, height).floor();
        let right = high[0].clamp(left, width).ceil();
        let bottom = high[1].clamp(top, height).ceil();

        [left, top, right - left, bottom - top].map(|pixels| pixels as u32)
    }
}

#[cfg(test)]
mod tests {
    use crate::box_view::sort_keys;

    #[test]
    fn keys_sort_by_their_upper_halves_and_ties_keep_their_order() {
        // Upper halves that differ within each pass's bits.
        let uppers = [0x7f80_0000, 3, 0x0040_0001, 3, 0, 0x0000_0801, 0x0040_0000];

        let mut keys: Vec<u64> = uppers
            .iter()
            .zip(0..)
            .map(|(upper, index)| upper << 32 | index)
            .collect();

        let mut sorted = keys.clone();

        sorted.sort_unstable();
        sort_keys(&mut keys, &mut Vec::new());

        assert_eq!(keys, sorted);
    }
}
