use crate::{
    BoxBindings, BoxBloom, BoxCrosser, BoxDraw, BoxFaces, BoxHide, BoxPart, DetailTree, Error,
    GlassTiles, LISTS, LitFill, PassTerms, PlacedBox, Result, TileMasks, TileScene, ViewEye,
    box_view_shader, quad_tlas, tile_quads,
};
use std::{collections::HashMap, mem, num::NonZeroU32};
use ty_math::{
    TyLinSrgbF32, TyLinSrgbF64, TyMatrix4x4F64, TyTransformF64, TyVector3F64, TyVector3U32,
};
use voxcore::material;
use voxrender::{RenderBloom, RenderLight, grid_point, tonemap};
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

/// The lists of a view's boxes, as a [`DetailTree`] counts them. A
/// [`BoxHide`] tests the first two.
const MARCHED: usize = 0;
const SOLID: usize = 1;
const GLASS: usize = 2;

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

    /// How many pages hold the scene's tiles. A page's words fit one
    /// binding, and a box's fragments read its page's.
    pages: usize,

    /// The scene's tiles at each level, where its objects keep coarser
    /// copies.
    detail: Option<DetailTree>,

    /// The fewest pixels a cell spans before a coarser copy's cells draw
    /// in its place, or zero where no copy draws.
    finest: f64,

    /// The boxes the last sort drew, each list's as their places in it.
    shown: [Vec<usize>; LISTS],

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

    /// The page that holds each box's tile.
    page: Vec<usize>,

    /// Whether each box's fragments can cast a mirror ray.
    mirror: Vec<bool>,

    /// Whether each box's fragments can meet an emission that blooms.
    glow: Vec<bool>,
}

impl ViewBoxes {
    fn push(
        &mut self,
        placed: &PlacedBox,
        words: [u32; 12],
        page: usize,
        mirror: bool,
        glow: bool,
    ) {
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
        self.page.push(page);
        self.mirror.push(mirror);
        self.glow.push(glow);
    }

    /// Fills `keys` with a key per box, nearest to `point` first, and
    /// returns how many of the boxes lie within `reach` of `point`. A key
    /// holds the bits of its box's squared distance above the box's index.
    /// With `shown`, only the boxes at those indices take a key.
    fn nearest_first(
        &self,
        shown: Option<&[usize]>,
        point: [f32; 3],
        reach: f32,
        keys: &mut Vec<u64>,
        spare: &mut Vec<u64>,
    ) -> usize {
        keys.clear();

        // A square is no less than zero, so its bits order as it does.
        let key = |index: usize| {
            let bounds = &self.bounds[index];

            let square: f32 = (0..3)
                .map(|axis| {
                    let off = point[axis].clamp(bounds[axis], bounds[axis + 3]) - point[axis];

                    off * off
                })
                .sum();

            u64::from(square.to_bits()) << 32 | index as u64
        };

        match shown {
            Some(shown) => keys.extend(shown.iter().map(|&index| key(index))),
            None => keys.extend((0..self.bounds.len()).map(key)),
        }

        sort_keys(keys, spare);
        keys.partition_point(|&key| f32::from_bits((key >> 32) as u32) <= reach * reach)
    }

    /// Appends the boxes at `indices` to `sorted` a page at a time, and a
    /// run of `part` to `runs` for each of the `pages` that holds one of
    /// them. The boxes keep their order within a page.
    fn deal(
        &self,
        indices: impl Iterator<Item = usize> + Clone,
        part: usize,
        pages: usize,
        sorted: &mut Vec<u8>,
        runs: &mut Vec<(usize, usize, u32)>,
    ) {
        for page in 0..pages {
            let before = sorted.len();

            for index in indices.clone().filter(|&index| self.page[index] == page) {
                sorted.extend_from_slice(&self.words[index]);
            }

            let count = (sorted.len() - before) / BoxDraw::BOX_BYTES;

            if count > 0 {
                runs.push((
                    part,
                    page,
                    u32::try_from(count).expect("a scene's boxes number under 2^32"),
                ));
            }
        }
    }

    /// Appends the boxes at `indices` to `sorted` in order, and a run of
    /// `part` to `runs` for each stretch of them that one page holds.
    fn deal_in_order(
        &self,
        indices: impl Iterator<Item = usize>,
        part: usize,
        sorted: &mut Vec<u8>,
        runs: &mut Vec<(usize, usize, u32)>,
    ) {
        let mut last = None;

        for index in indices {
            let page = self.page[index];

            sorted.extend_from_slice(&self.words[index]);

            match runs.last_mut().filter(|_| last == Some(page)) {
                Some((_, _, count)) => *count += 1,
                None => runs.push((part, page, 1)),
            }

            last = Some(page);
        }
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

impl BoxView {
    /// A mirror ray runs to where what it meets would show by this share
    /// of its face's light. A face that reflects less casts none.
    pub const MIRROR_CUT: f32 = 1.0 / 64.0;

    /// The least cells a mirror ray reaches. A face whose ray would end
    /// sooner casts none: what touches it shows too faintly to earn a ray.
    pub const MIRROR_REACH: f32 = 2.0;

    /// The format of the depth a pass that draws the view clears to zero.
    pub const DEPTH: TextureFormat = BoxDraw::DEPTH;

    /// Whether a view on `device` draws the surfaces between materials that
    /// pass light, which takes a device that blends two colors a fragment.
    /// A [`TileScene`] keeps those surfaces for such a view.
    pub fn draws_glass(device: &Device) -> bool {
        device.features().contains(Features::DUAL_SOURCE_BLENDING)
    }

    /// Builds the view of `tiles` under `bloom` and fills the sun's corner
    /// shadows. The view draws to a two-layer target of
    /// `format` whose width and height are `size`. A page of the scene's
    /// tiles takes as many bytes as one of the device's storage bindings
    /// holds. Errors as [`paged`](Self::paged) does.
    pub fn new(
        device: &Device,
        queue: &Queue,
        tiles: &TileScene,
        bloom: RenderBloom,
        format: TextureFormat,
        size: [u32; 2],
    ) -> Result<Self> {
        let page_bytes = device.limits().max_storage_buffer_binding_size;

        Self::paged(device, queue, tiles, bloom, format, size, page_bytes)
    }

    /// Builds the view as [`new`](Self::new) does, with pages of
    /// `page_bytes` bytes, no more than one of the device's storage
    /// bindings holds.
    ///
    /// The scene's objects fill one page after another. A page holds its
    /// objects' tiles in one buffer and its placements' lit words in
    /// another. Objects whose boxes share space take one page, because a
    /// fragment that marches a box of another placement reads that box's
    /// tile. Errors if the tiles of such objects, or what their placements'
    /// cells take in lit words, pass a page, or if waiting on the sun's
    /// fill fails.
    pub fn paged(
        device: &Device,
        queue: &Queue,
        tiles: &TileScene,
        bloom: RenderBloom,
        format: TextureFormat,
        size: [u32; 2],
        page_bytes: u64,
    ) -> Result<Self> {
        // The scene blooms where an emission's brightest channel passes
        // the threshold.
        let glows = bloom.strength > 0.0
            && tiles.materials.iter().any(|material| {
                let emission = material.emissive_color * material.emissive_strength;

                emission.red.max(emission.green).max(emission.blue) > bloom.threshold
            });

        let radius = bloom.radius;
        let bloom = glows.then(|| (BoxBloom::new(device, size, bloom), bloom.threshold as f32));

        let count = |value: usize| u32::try_from(value).expect("a scene's words number under 2^32");
        let objects = &tiles.objects;

        // The surfaces between materials that pass light draw as glass
        // where the device blends a tint a channel.
        let blends = Self::draws_glass(device);
        let no_glass = GlassTiles {
            boxes: Vec::new(),
            surfaces: Vec::new(),
            tiles: Vec::new(),
            cells: Vec::new(),
            words: Vec::new(),
        };

        let glass_of = |object: usize| {
            if blends {
                &objects[object].glass
            } else {
                &no_glass
            }
        };

        let object_of: Vec<_> = tiles.placements.iter().map(|&(object, _)| object).collect();

        let placements: Vec<_> = tiles
            .placements
            .iter()
            .map(|&(_, transform)| {
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

        let marched = PlacedBox::list(
            &tiles.placements,
            &(objects.iter())
                .map(|object| object.masks.boxes.as_slice())
                .collect::<Vec<_>>(),
            &tiles.spans,
        );

        let solid = PlacedBox::list(
            &tiles.placements,
            &(objects.iter())
                .map(|object| object.masks.solids.as_slice())
                .collect::<Vec<_>>(),
            &tiles.spans,
        );

        // Each marched box's entries in `crosser_words`, as their first and
        // their count.
        let crossers = PlacedBox::crossers(&marched, &solid);

        let mut sun = (TyVector3F64::Y, TyLinSrgbF64::new(0.0, 0.0, 0.0));
        let mut sky = TyLinSrgbF64::new(0.0, 0.0, 0.0);
        let mut ground = sky;
        let mut suns = 0;

        for light in &tiles.lights {
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

        // Where a material passes light, the sun's fill adds each record
        // cell's tint in a second word.
        let tinted = suns > 0 && tiles.opaque.contains(&false);

        // The words of each object's tiles, and of what its placements'
        // cells take in `lit`.
        let sizes: Vec<_> = (0..objects.len())
            .map(|object| {
                let records: usize = objects[object].masks.recorded.iter().map(Vec::len).sum();
                let glass: usize = glass_of(object).cells.iter().map(Vec::len).sum();
                let placed = object_of.iter().filter(|&&placed| placed == object).count();

                (
                    objects[object].masks.words.len() + glass_of(object).words.len(),
                    (records + glass) * placed,
                )
            })
            .collect();

        // A fragment that marches a box of another placement reads that
        // box's tile, so objects whose boxes share space share a page.
        let mut group: Vec<_> = (0..objects.len()).collect();

        for (placed, entries) in marched.iter().zip(&crossers) {
            for entry in entries {
                let other = (entry.entry & !BoxCrosser::SOLID) as usize;

                let other = match other.checked_sub(marched.len()) {
                    Some(other) => &solid[other],
                    None => &marched[other],
                };

                let (a, b) = (
                    group[object_of[placed.placement as usize]],
                    group[object_of[other.placement as usize]],
                );

                if a != b {
                    for joined in &mut group {
                        if *joined == a.max(b) {
                            *joined = a.min(b);
                        }
                    }
                }
            }
        }

        // The pages: each takes the groups in order that fit what one
        // binding holds of tiles' words and of lit words. A tinted scene's
        // lit words take a binding's first half, and its tints the second.
        // The objects come before their coarser copies, finest first, so a
        // view of copies alone draws from the pages that hold them.
        let most = (page_bytes / 4) as usize;
        let most_lit = (if tinted { most / 2 } else { most }) - TileMasks::SPARE as usize;
        let mut page_of = vec![0; objects.len()];
        let mut filled = (0, 0);
        let mut pages = 1;

        let mut firsts: Vec<_> = (0..objects.len())
            .filter(|&object| group[object] == object)
            .collect();

        firsts.sort_by_key(|&first| tiles.spans[first]);

        for first in firsts {
            let members = || (first..objects.len()).filter(|&object| group[object] == first);

            let (bits, lit) = members().fold((0, 0), |(bits, lit), object| {
                (bits + sizes[object].0, lit + sizes[object].1)
            });

            if bits > most || lit > most_lit {
                return Err(Error::Page {
                    bytes: 4 * bits.max(lit * if tinted { 2 } else { 1 }) as u64,
                    most: page_bytes,
                });
            }

            if filled.0 + bits > most || filled.1 + lit > most_lit {
                pages += 1;
                filled = (0, 0);
            }

            filled = (filled.0 + bits, filled.1 + lit);

            for object in members() {
                page_of[object] = pages - 1;
            }
        }

        let words_bytes = |words: &[u32]| -> Vec<u8> {
            words.iter().flat_map(|word| word.to_le_bytes()).collect()
        };

        // `box_view.wgsl`'s `bits` for each page: each of its objects' mask
        // words, then the object's glass tiles' words. An object keeps
        // where each starts in its page.
        let mut offsets = vec![(0, 0); objects.len()];

        let bits: Vec<Buffer> = (0..pages)
            .map(|page| {
                let held: Vec<_> = (0..objects.len())
                    .filter(|&object| page_of[object] == page)
                    .collect();

                let mut words = 0;

                for &object in &held {
                    let glass = words + objects[object].masks.words.len();

                    offsets[object] = (count(words), count(glass));
                    words = glass + glass_of(object).words.len();
                }

                // A binding cannot be empty.
                let buffer = device.create_buffer(&BufferDescriptor {
                    label: Some("view bits"),
                    size: 4 * words.max(1) as u64,
                    usage: BufferUsages::STORAGE,
                    mapped_at_creation: true,
                });

                let held = held.iter().flat_map(|&object| {
                    (objects[object].masks.words.iter()).chain(&glass_of(object).words)
                });

                let spare = if words == 0 { Some(&0) } else { None };

                buffer
                    .get_mapped_range_mut(..)
                    .expect("a buffer mapped at creation gives its bytes")
                    .slice(..)
                    .write_iter(held.chain(spare).flat_map(|word| word.to_le_bytes()));

                buffer.unmap();

                buffer
            })
            .collect();

        // A box's object and its tile's offset among the object's mask
        // words.
        let tile_of = |placed: &PlacedBox, solid: bool| {
            let object = object_of[placed.placement as usize];
            let masks = &objects[object].masks;

            (
                object,
                if solid {
                    &masks.solid_tiles
                } else {
                    &masks.tiles
                }[placed.brick],
            )
        };

        // `box_view.wgsl`'s `lit` for each page: a word for each record
        // cell of each of its placements' tiles, with the cell's material
        // in its high 8 bits. A hit the depth test drops can read the spare
        // words past its tile's last record cell's.
        let mut lit_places = vec![0; object_of.len()];
        let mut glass_lit: Vec<Vec<u32>> = vec![Vec::new(); object_of.len()];

        let page_lit: Vec<(Vec<usize>, Vec<u32>)> = (0..pages)
            .map(|page| {
                let placed: Vec<_> = (0..object_of.len())
                    .filter(|&placement| page_of[object_of[placement]] == page)
                    .collect();

                let mut lit_words: Vec<u32> = Vec::new();

                for &placement in &placed {
                    lit_places[placement] = count(lit_words.len());

                    lit_words.extend(
                        objects[object_of[placement]]
                            .masks
                            .entries
                            .iter()
                            .flatten()
                            .map(|&entry| u32::from(entry) << 24),
                    );
                }

                // A word for each glass cell follows: the first of each
                // glass tile's, for each placement.
                for &placement in &placed {
                    glass_lit[placement] = glass_of(object_of[placement])
                        .cells
                        .iter()
                        .map(|cells| {
                            let first = count(lit_words.len());

                            lit_words.resize(lit_words.len() + cells.len(), 0);

                            first
                        })
                        .collect();
                }

                lit_words.extend([0; TileMasks::SPARE as usize]);

                (placed, lit_words)
            })
            .collect();

        // How far past a record cell's word its tint lies, in every page.
        let tints = tinted.then(|| {
            let most = page_lit.iter().map(|(_, words)| words.len()).max();

            count(most.expect("a view has a page"))
        });

        // The sun's corner shadows, which the GPU fills into the words' low
        // bits before the view's first draw. A scene with no sun leaves them
        // zero, and its faces skip the sun's term.
        let fill = (suns > 0).then(|| LitFill::new(device, tiles, sun.0));

        let lit_corners: Vec<Buffer> = page_lit
            .iter()
            .map(|(placed, lit_words)| {
                let Some(fill) = &fill else {
                    return Ok(device.create_buffer_init(&BufferInitDescriptor {
                        label: Some("view lit corners"),
                        contents: &words_bytes(lit_words),
                        usage: BufferUsages::STORAGE,
                    }));
                };

                let page = fill.page(device, tiles, placed, blends, tints);

                let glass_places: Vec<_> = (placed.iter())
                    .zip(&page.glass_places)
                    .map(|(&placement, &place)| {
                        glass_lit[placement].first().copied().unwrap_or(place)
                    })
                    .collect();

                let places: Vec<_> = placed
                    .iter()
                    .map(|&placement| lit_places[placement])
                    .collect();

                assert_eq!(
                    (&page.places, &page.glass_places),
                    (&places, &glass_places),
                    "the fill lays out the view's words"
                );

                queue.write_buffer(&page.lit, 0, &words_bytes(lit_words));
                page.fill(device, queue)?;

                Ok(page.lit)
            })
            .collect::<Result<_>>()?;

        drop(page_lit);

        let tints = tints.unwrap_or(0);

        // A material's three `vec4<f32>`s: its base color and metalness, its
        // emission and roughness, and its reflectance at normal incidence
        // and occlusion strength.
        let material_values = |id: u32| {
            let material = &tiles.materials[id as usize];
            let base = material.base_color.color;
            let emission = material.emissive_color * material.emissive_strength;
            let reflectance = material::normal_reflectance(base, material.metallic, material.ior);

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
        };

        // Each tile's first material in `materials`. The tile's materials
        // follow one another there, so a cell's place among them leads to
        // its material's values in one read. Tiles that hold the same
        // materials in the same order share them.
        // A glass material holds the seven `vec4<f32>`s `box_glass.wgsl`'s
        // `past` lists.
        let glass_values = |id: u32| {
            let material = &tiles.materials[id as usize];
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
        };

        // The glass materials come first, where a cell's word reaches them
        // in 12 bits. A tile's first material counts in threes after them.
        let mut materials: Vec<f32> = tiles
            .glass_materials
            .iter()
            .flat_map(|&id| glass_values(id))
            .collect();

        materials.resize(materials.len().next_multiple_of(12), 0.0);

        // The material ids of an object's tile, whose words start at
        // `tile` among the object's mask words.
        let ids_of = |(object, tile): (usize, u32)| {
            let words = &objects[object].masks.words;
            let palette = words[(tile + TileMasks::PALETTE) as usize];
            let ids = (tile + (palette & 0xffff)) as usize;

            &words[ids..ids + (palette >> 16) as usize]
        };

        let mut firsts: HashMap<Vec<u32>, u32> = HashMap::new();
        let mut tile_firsts: HashMap<(usize, u32), u32> = HashMap::new();

        let mut first_material = |tile: (usize, u32)| {
            *tile_firsts.entry(tile).or_insert_with(|| {
                let ids = ids_of(tile).to_vec();

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
        let mut tile_mirrors: HashMap<(usize, u32), bool> = HashMap::new();

        // The cells a mirror ray from a face of a material runs, or zero
        // for a material whose faces cast none.
        let mirror_reach = |id: u32| {
            let values = material_values(id);
            let alpha = (values[7] * values[7]).max(1.0e-3);
            let reflectance = values[8].max(values[9]).max(values[10]);
            let far = (reflectance / Self::MIRROR_CUT - 1.0).max(0.0).sqrt() / alpha;

            if traced && values[7] < 1.0 && far > Self::MIRROR_REACH {
                far
            } else {
                0.0
            }
        };

        // A ray traces an object's quads, so its coarser copies cast none.
        let mut mirror_tile = |tile: (usize, u32)| {
            *tile_mirrors.entry(tile).or_insert_with(|| {
                tiles.spans[tile.0] == 1 && ids_of(tile).iter().any(|&id| mirror_reach(id) > 0.0)
            })
        };

        // A tile glows where one of its materials' emission blooms.
        let mut tile_glows: HashMap<(usize, u32), bool> = HashMap::new();

        let mut glow_tile = |tile: (usize, u32)| {
            *tile_glows.entry(tile).or_insert_with(|| {
                bloom.as_ref().is_some_and(|&(_, threshold)| {
                    ids_of(tile).iter().any(|&id| {
                        let values = material_values(id);

                        values[4].max(values[5]).max(values[6]) > threshold
                    })
                })
            })
        };

        // A box's three `vec4<u32>`s. Its tile's words start past its
        // object's first word in its page. The last two words hold what
        // its fragments read of the tile with no load: the tile's first
        // material and the first of its lit words.
        let mut words_of = |placed: &PlacedBox, solid: bool, more: [u32; 2]| {
            let (object, tile) = tile_of(placed, solid);
            let mut words = placed.words(placed.placement, offsets[object].0 + tile, more);
            let before = objects[object].masks.words[(tile + TileMasks::FIRST) as usize];

            words[10] = first_material((object, tile));
            words[11] = lit_places[placed.placement as usize] + before;

            words
        };

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

                words_of(placed, false, span)
            })
            .collect();

        let solid_words: Vec<_> = solid
            .iter()
            .map(|placed| words_of(placed, true, [0, 0]))
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
            .map(|placed| mirror_tile(tile_of(placed, true)))
            .collect();

        // A marched box's fragment can also shade a hit in a box it
        // crosses.
        let march_mirrors: Vec<_> = marched
            .iter()
            .zip(&crossers)
            .map(|(placed, entries)| {
                mirror_tile(tile_of(placed, false))
                    || entries.iter().any(|entry| {
                        let other = (entry.entry & !BoxCrosser::SOLID) as usize;

                        match other.checked_sub(marched.len()) {
                            Some(other) => solid_mirrors[other],
                            None => mirror_tile(tile_of(&marched[other], false)),
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
            .map(|placed| glow_tile(tile_of(placed, true)))
            .collect();

        let march_glows: Vec<_> = marched
            .iter()
            .zip(&crossers)
            .map(|(placed, entries)| {
                glow_tile(tile_of(placed, false))
                    || entries.iter().any(|entry| {
                        let other = (entry.entry & !BoxCrosser::SOLID) as usize;

                        match other.checked_sub(marched.len()) {
                            Some(other) => solid_glows[other],
                            None => glow_tile(tile_of(&marched[other], false)),
                        }
                    })
            })
            .collect();

        let mut solid_boxes = ViewBoxes::default();
        let mut marched_boxes = ViewBoxes::default();

        let page = |placed: &PlacedBox| page_of[object_of[placed.placement as usize]];

        for (index, (placed, &words)) in solid.iter().zip(&solid_words).enumerate() {
            let (mirror, glow) = (solid_mirrors[index], solid_glows[index]);

            solid_boxes.push(placed, words, page(placed), mirror, glow);
        }

        for (index, (placed, &words)) in marched.iter().zip(&march_words).enumerate() {
            let (mirror, glow) = (march_mirrors[index], march_glows[index]);

            marched_boxes.push(placed, words, page(placed), mirror, glow);
        }

        let crossing: Vec<bool> = crossers.iter().map(|entries| !entries.is_empty()).collect();

        let hide = (marched.len() + solid.len() >= TESTED)
            .then(|| {
                // The corners of each placement's slabs in the world. Bit
                // `a` of a corner's index picks the slab's far side on axis
                // `a`. An object's slabs hide what lies behind them at
                // any level, where a coarser copy's reach past its cells.
                tiles
                    .placements
                    .iter()
                    .filter(|(object, _)| tiles.spans[*object] == 1)
                    .flat_map(|(object, transform)| {
                        objects[*object].slabs.iter().map(|&(low, high)| {
                            [0, 1, 2, 3, 4, 5, 6, 7].map(|corner: u32| {
                                let far =
                                    TyVector3U32::new(corner & 1, corner >> 1 & 1, corner >> 2);

                                transform.transform_point((low + (high - low) * far).as_dvec3())
                            })
                        })
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

        if box_words.is_empty() {
            box_words.resize(12, 0);
        }

        // What a mirror ray traces, on a device with ray queries: the
        // scene's opaque quads, each of one material, and each quad's
        // face under its material's place in `materials`.
        let mut quad_words: Vec<u32> = Vec::new();

        // The bloom's source casts a ray from each glowing hit to the eye,
        // which any quad can hide. A mirror ray alone meets only the quads
        // within its reach.
        let tlas = (mirrors || traced && bloom.is_some()).then(|| {
            let reach = |id: u32| f64::from(mirror_reach(id));

            let quads = tile_quads(tiles, if bloom.is_some() { None } else { Some(&reach) });

            let mut places: HashMap<u32, u32> = HashMap::new();

            for object in &quads {
                for (&id, &face) in object.materials.iter().zip(&object.faces) {
                    let place = *places.entry(id).or_insert_with(|| {
                        let place = materials.len() / 12;

                        materials.extend(material_values(id));

                        u32::try_from(place)
                            .ok()
                            .filter(|&place| place < 1 << 29)
                            .expect("a scene's materials number under 2^29")
                    });

                    quad_words.push(place << 3 | face);
                }
            }

            quad_tlas(device, queue, &quads, &tiles.placements)
        });

        if quad_words.is_empty() {
            quad_words.push(0);
        }

        let glass = PlacedBox::list(
            &tiles.placements,
            &(0..objects.len())
                .map(|object| glass_of(object).boxes.as_slice())
                .collect::<Vec<_>>(),
            &tiles.spans,
        );

        let detail = DetailTree::new(tiles, [&marched, &solid, &glass]);
        let mut glass_boxes = ViewBoxes::default();

        let glass_words: Vec<_> = glass
            .iter()
            .map(|placed| {
                let object = object_of[placed.placement as usize];
                let first = offsets[object].1;
                let GlassTiles {
                    tiles, surfaces, ..
                } = glass_of(object);

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

                glass_boxes.push(placed, words, page(placed), false, false);

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

        // A box that crosses another draws in two parts when it lies near
        // the head, so the draw has room for each such box twice.
        let crossed = crossers
            .iter()
            .filter(|entries| !entries.is_empty())
            .count();

        let glowing = |glows: &[bool]| glows.iter().filter(|&&glows| glows).count();

        let glow_crossed = (crossers.iter().zip(&march_glows))
            .filter(|&(entries, &glows)| glows && !entries.is_empty())
            .count();

        let part = |fragment, faces, module, room| BoxPart {
            fragment,
            faces,
            module,
            room,
            glass: false,
        };

        let mut parts = vec![
            part("held", BoxFaces::Held, near, crossed),
            part("inside", BoxFaces::NearBack, near, 0),
            part("solid", BoxFaces::Front, 0, solid_words.len()),
            part("solid", BoxFaces::Front, near, 0),
            part("front", BoxFaces::Front, 0, march_words.len()),
            part("front", BoxFaces::Front, near, 0),
            part("crossed", BoxFaces::Front, 0, 0),
            part("crossed", BoxFaces::Front, near, 0),
        ];

        // Glass draws last. A box near the head draws in both parts, and
        // each eye takes the one that fits it.
        if glassy.is_some() {
            parts.extend(
                [("glass", BoxFaces::Front), ("glass_held", BoxFaces::Held)].map(
                    |(fragment, faces)| BoxPart {
                        fragment,
                        faces,
                        module: modules.len() - 1,
                        room: glass_words.len(),
                        glass: true,
                    },
                ),
            );
        }

        // Each page binds its tiles' words and its lit words.
        let buffers: Vec<_> = (bits.iter().zip(&lit_corners))
            .map(|(bits, lit_corners)| {
                [
                    (both, BufferBindingType::Uniform, &frame),
                    (both, storage, &places),
                    (ShaderStages::FRAGMENT, storage, bits),
                    (ShaderStages::FRAGMENT, storage, &materials),
                    (ShaderStages::FRAGMENT, storage, &box_list),
                    (ShaderStages::FRAGMENT, storage, &crosser_list),
                    (ShaderStages::FRAGMENT, storage, lit_corners),
                    (ShaderStages::FRAGMENT, storage, &quad_list),
                ]
            })
            .collect();

        let bindings = |halo| -> Vec<_> {
            (buffers.iter())
                .map(|buffers| BoxBindings {
                    buffers,
                    sampled: Some((halo, &bloom_sampler)),
                    tlas: tlas.as_ref(),
                })
                .collect()
        };

        let mut boxes = BoxDraw::new(
            device,
            &modules,
            format,
            &parts,
            &bindings(&halo),
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
                        part("held", BoxFaces::Held, 0, glow_crossed),
                        part("inside", BoxFaces::NearBack, 0, 0),
                        part("solid", BoxFaces::Front, 0, glowing(&solid_glows)),
                        part("front", BoxFaces::Front, 0, glowing(&march_glows)),
                        part("crossed", BoxFaces::Front, 0, 0),
                    ],
                    &bindings(&no_halo),
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
            pages,
            detail,
            finest: 0.0,
            shown: Default::default(),
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

    /// Sets the fewest pixels one of an object's cells spans before a
    /// coarser copy's cells draw in its place. The view draws a place at
    /// the coarsest copy whose cells span that many pixels or fewer, in a
    /// scene whose objects keep copies. Zero draws every object's cells,
    /// as a view starts.
    pub fn set_detail(&mut self, pixels: f64) {
        self.finest = pixels;
        self.sorted_for = None;
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

        // The widest angle a pixel spans: the one on an eye's axis. A
        // clip matrix's first two rows scale by two over the tangents the
        // target spans.
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

        if !holds {
            self.sort(queue, head, reach + slack, pixel);
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
    fn sort(&mut self, queue: &Queue, head: [f32; 3], covered: f32, pixel: f64) {
        let spare = &mut self.spare_keys;

        // A cell a distance away spans a pixel where its side takes
        // `pixel` of the distance.
        let shown = self.detail.as_ref().map(|detail| {
            detail.select(head, (self.finest * pixel) as f32, &mut self.shown);

            &self.shown
        });

        let shown = |list: usize| shown.map(|shown| shown[list].as_slice());

        (self.solid).nearest_first(shown(SOLID), head, covered, &mut self.solid_keys, spare);

        self.near = (self.marched).nearest_first(
            shown(MARCHED),
            head,
            covered,
            &mut self.marched_keys,
            spare,
        );

        if !self.glass.words.is_empty() {
            self.glass_near = (self.glass).nearest_first(
                shown(GLASS),
                head,
                covered,
                &mut self.glass_keys,
                spare,
            );
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

        // The glowing boxes take the view's parts for the halo's draw.
        if let Some(glow) = &mut self.glow {
            let (near, far) = self.marched_keys.split_at(self.near);
            let crossing = &self.crossing;
            let solid = &self.solid;
            let marched = &self.marched;
            let glowing = |boxes: &'_ ViewBoxes, keys: &[u64]| -> Vec<usize> {
                (keys.iter().map(|&key| index_of(key)))
                    .filter(|&index| boxes.glow[index])
                    .collect()
            };

            let crossed = |keys: &[u64], crosses: bool| -> Vec<u64> {
                (keys.iter().copied())
                    .filter(|&key| crossing[index_of(key)] == crosses)
                    .collect()
            };

            // Each of the halo's parts with its boxes.
            let lists = [
                (marched, glowing(marched, &crossed(near, true))),
                (marched, glowing(marched, &crossed(near, false))),
                (solid, glowing(solid, &self.solid_keys)),
                (marched, glowing(marched, &crossed(far, false))),
                (
                    marched,
                    glowing(marched, &crossed(&self.marched_keys, true)),
                ),
            ];

            let mut runs = Vec::new();

            self.sorted.clear();

            for (part, (boxes, indices)) in lists.iter().enumerate() {
                boxes.deal(
                    indices.iter().copied(),
                    part,
                    self.pages,
                    &mut self.sorted,
                    &mut runs,
                );
            }

            glow.boxes.rewrite(queue, &self.sorted, &runs);
        }
    }

    /// Deals the sorted boxes into runs: a part's boxes of one page each.
    /// A part of front faces leaves out the boxes the test hides.
    fn deal(&mut self, queue: &Queue) {
        self.sorted.clear();

        let mut runs = Vec::new();

        for (part, order) in self.order.iter().enumerate() {
            let (boxes, list) = if (SOLID_RUN..FRONT_RUN).contains(&part) {
                (&self.solid, SOLID)
            } else {
                (&self.marched, MARCHED)
            };

            // The parts from the solid ones on draw front faces.
            let hide = self.hide.as_ref().filter(|_| part >= SOLID_RUN);

            let shown = (order.iter().copied())
                .filter(|&index| !hide.is_some_and(|hide| hide.hides(list, index)));

            boxes.deal(shown, part, self.pages, &mut self.sorted, &mut runs);
        }

        // Glass draws farthest first: every box through its front faces,
        // then the boxes an eye can stand in through their back faces. A
        // run ends where the next box's page differs, which keeps the
        // order across pages.
        if !self.glass.words.is_empty() {
            for (part, keys) in [
                (RUNS, &self.glass_keys[..]),
                (RUNS + 1, &self.glass_keys[..self.glass_near]),
            ] {
                let farthest_first = keys.iter().rev().map(|&key| index_of(key));

                (self.glass).deal_in_order(farthest_first, part, &mut self.sorted, &mut runs);
            }
        }

        self.boxes.rewrite(queue, &self.sorted, &runs);
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
