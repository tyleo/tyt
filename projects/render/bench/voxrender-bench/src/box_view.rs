use crate::{
    BObjectTile, BTileObject, BViewBox, BoxBindings, BoxBloom, BoxCrosser, BoxDraw, BoxFaces,
    BoxHide, BoxPart, DetailTree, Error, LISTS, MaskTile, PlacedBox, Result, TileMasks,
    TilePlacement, TileScene, ViewBoxes, ViewEye, ViewLights, ViewLit, ViewMaterials, ViewPages,
    ViewTarget, box_corners, box_view_shader, quad_tlas, storage_words, tile_quads, to_word,
};
use branded_id::{IdVec, UsizeId};
use ty_math::{TyLinSrgbF32, TyLinSrgbF64, TyMatrix4x4F64, TyTransformF64, TyVector3F64};
use voxrender::{RenderBloom, grid_point, tonemap};
use wgpu::{
    AddressMode, Buffer, BufferBindingType, BufferDescriptor, BufferUsages, Color, CommandEncoder,
    Device, Extent3d, Features, FilterMode, Queue, SamplerDescriptor, ShaderModuleDescriptor,
    ShaderRuntimeChecks, ShaderSource, ShaderStages, TextureDescriptor, TextureDimension,
    TextureFormat, TextureUsages, TextureView, TextureViewDescriptor,
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
    shown: [Vec<UsizeId<BViewBox>>; LISTS],

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
    crossing: IdVec<BViewBox, bool>,

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
    order: [Vec<UsizeId<BViewBox>>; RUNS],

    /// The last sort's boxes and keys, kept for their room.
    sorted: Vec<[u32; 12]>,
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
    /// shadows. The view draws to a two-layer target of `format` whose
    /// width and height are `size`.
    ///
    /// The scene's objects fill pages of `page_bytes` bytes, or of what one
    /// of the device's storage bindings holds. Errors as [`ViewPages::new`]
    /// and [`ViewLit::new`] do, and if the scene keeps glass the device
    /// cannot draw.
    pub fn new(
        device: &Device,
        queue: &Queue,
        tiles: &TileScene,
        bloom: RenderBloom,
        format: TextureFormat,
        size: [u32; 2],
        page_bytes: Option<u64>,
    ) -> Result<Self> {
        if tiles.glass && !Self::draws_glass(device) {
            return Err(Error::Glass);
        }

        let page_bytes = page_bytes.unwrap_or(device.limits().max_storage_buffer_binding_size);
        let objects = &tiles.objects;
        let object_of =
            |placed: &PlacedBox| tiles.placements[placed.placement.to_usize_id()].object;

        let marched = PlacedBox::list(tiles, |object| &object.masks.boxes);
        let solid = PlacedBox::list(tiles, |object| &object.masks.solids);
        let glass = PlacedBox::list(tiles, |object| &object.glass.boxes);
        let crossers = PlacedBox::crossers(&marched, &solid);

        // The box a crosser's entry holds.
        let crossed = |crosser: &BoxCrosser| {
            let entry = (crosser.entry & !BoxCrosser::SOLID) as usize;

            match entry.checked_sub(marched.len()) {
                Some(entry) => &solid[entry],
                None => &marched[entry],
            }
        };

        let lights = ViewLights::new(&tiles.lights);
        let toward = lights.sun.map(|(toward, _)| toward);

        // Where a material passes light, the sun's fill adds each record
        // cell's tint in a second word.
        let tinted = toward.is_some() && tiles.opaque.contains(&false);

        let crossing = (marched.iter().zip(&crossers)).flat_map(|(placed, entries)| {
            (entries.iter()).map(|entry| [object_of(placed), object_of(crossed(entry))])
        });

        let pages = ViewPages::new(tiles, crossing, page_bytes, tinted)?;
        let (bits, starts) = pages.bits(device, tiles);
        let lit = ViewLit::new(device, queue, tiles, &pages, toward, tinted)?;

        // The scene blooms where an emission's brightest channel passes
        // the threshold.
        let glows = bloom.strength > 0.0
            && tiles.materials.iter().any(|material| {
                let emission = material.emissive_color * material.emissive_strength;

                emission.red.max(emission.green).max(emission.blue) > bloom.threshold
            });

        let threshold = glows.then_some(bloom.threshold as f32);
        let reach = REACH_SIGMAS * bloom.radius * f64::from(size[0].min(size[1]));
        let bloom = glows.then(|| BoxBloom::new(device, size, bloom));

        // A mirror ray needs a device with ray queries.
        let traced = device.features().contains(Features::EXPERIMENTAL_RAY_QUERY);
        let mut materials = ViewMaterials::new(tiles);

        let traits: Vec<IdVec<BObjectTile, TileTraits>> = (objects.iter())
            .map(|object| {
                let of = |tile: &MaskTile| {
                    let ids = object.masks.palette(tile.offset);
                    let reaches = |&id: &u32| materials.mirror_reach(id, traced) > 0.0;

                    // A ray traces an object's quads, so its coarser copies
                    // cast none.
                    let mirror = object.span == 1 && ids.iter().any(reaches);

                    let glow = threshold.is_some_and(|threshold| {
                        (ids.iter()).any(|&id| materials.emits_over(id, threshold))
                    });

                    TileTraits {
                        material: materials.run(ids),
                        mirror,
                        glow,
                    }
                };

                IdVec::from_vec(object.masks.tiles.iter().map(of).collect())
            })
            .collect();

        let traits = IdVec::<BTileObject, _>::from_vec(traits);

        let traits_of = |placed: &PlacedBox| traits[object_of(placed)][placed.tile];
        let page_of = |placed: &PlacedBox| pages.of[object_of(placed)];

        // A box's three `vec4<u32>`s. Its tile's words start past its
        // object's first word in its page. The last two words hold what
        // its fragments read of the tile with no load: the tile's first
        // material and the first of its lit words.
        let words_of = |placed: &PlacedBox, more: [u32; 2]| {
            let object = object_of(placed);
            let masks = &objects[object].masks;
            let tile = masks.tiles[placed.tile].offset;
            let before = masks.words[(tile + TileMasks::FIRST) as usize];
            let mut words = placed.words(starts[object] + tile, more);

            words[10] = traits_of(placed).material;
            words[11] = lit.places[placed.placement.to_usize_id()] + before;

            words
        };

        let mut crosser_words: Vec<u32> = Vec::new();
        let mut marched_boxes = ViewBoxes::default();
        let mut solid_boxes = ViewBoxes::default();
        let mut glass_boxes = ViewBoxes::default();

        for (placed, entries) in marched.iter().zip(&crossers) {
            let span = [to_word(crosser_words.len() / 8), to_word(entries.len())];

            crosser_words.extend(entries.iter().flat_map(BoxCrosser::words));

            // A marched box's fragment can also shade a hit in a box it
            // crosses.
            let shaded = || (entries.iter().map(crossed)).chain([placed]).map(traits_of);

            let mirror = shaded().any(|traits| traits.mirror);
            let glow = shaded().any(|traits| traits.glow);

            marched_boxes.push(
                placed,
                words_of(placed, span),
                page_of(placed),
                mirror,
                glow,
            );
        }

        for placed in &solid {
            let TileTraits { mirror, glow, .. } = traits_of(placed);

            solid_boxes.push(
                placed,
                words_of(placed, [0, 0]),
                page_of(placed),
                mirror,
                glow,
            );
        }

        for placed in &glass {
            let object = object_of(placed);
            let tile = &objects[object].glass.tiles[placed.tile];

            // An object's glass words follow its mask words.
            let first = starts[object] + to_word(objects[object].masks.words.len());

            // The box around the tile's surfaces, 4 bits a side: its low
            // cell in the tile, then its high cell.
            let (low, high) = tile.surfaces;
            let [x, y, z] = (low % 16).to_array();
            let [far_x, far_y, far_z] = ((high - 1) % 16).to_array();
            let surfaces = x | y << 4 | z << 8 | far_x << 12 | far_y << 16 | far_z << 20;
            let mut words = placed.words(first + tile.offset, [surfaces, 0]);

            // The first of the tile's words in `lit`.
            words[11] = lit.glass[placed.placement.to_usize_id()][placed.tile];

            glass_boxes.push(placed, words, page_of(placed), false, false);
        }

        let crossing: Vec<bool> = crossers.iter().map(|entries| !entries.is_empty()).collect();
        let crossing = IdVec::<BViewBox, _>::from_vec(crossing);
        let hide = hide_of(tiles, [(&marched, &marched_boxes), (&solid, &solid_boxes)]);
        let detail = DetailTree::new(tiles, [&marched, &solid, &glass]);

        // The boxes an eye can stand in take the mirror's module wherever
        // the scene holds a mirror.
        let mirrors = (solid_boxes.mirror.iter())
            .chain(&marched_boxes.mirror)
            .any(|&mirror| mirror);

        // What a mirror ray traces, on a device with ray queries: the
        // scene's opaque quads, each of one material, and each quad's
        // face under its material's place in `materials`.
        let mut quad_words: Vec<u32> = Vec::new();

        // The bloom's source casts a ray from each glowing hit to the eye,
        // which any quad can hide. A mirror ray alone meets only the quads
        // within its reach.
        let tlas = (mirrors || traced && bloom.is_some()).then(|| {
            let reach = |id: u32| f64::from(materials.mirror_reach(id, traced));
            let quads = tile_quads(tiles, if bloom.is_some() { None } else { Some(&reach) });

            for object in &quads {
                for (&id, &face) in object.materials.iter().zip(&object.faces) {
                    quad_words.push(materials.single(id) << 3 | face);
                }
            }

            quad_tlas(device, queue, &quads, &tiles.placements)
        });

        let init = |label, contents: &[u8], usage| {
            device.create_buffer_init(&BufferInitDescriptor {
                label: Some(label),
                contents,
                usage,
            })
        };

        let frame = init(
            "view frame",
            &frame_bytes(&lights, tiles.placements.len(), size),
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );
        let material_list = init(
            "view materials",
            bytemuck::cast_slice(&materials.finish()),
            BufferUsages::STORAGE,
        );

        // `box_view.wgsl`'s `boxes`: the marched boxes, then the solid ones.
        let box_words = [marched_boxes.words.as_slice(), solid_boxes.words.as_slice()].concat();

        // A shader's array has room for one element at the least.
        let quad_list = storage_words(device, "view quads", &quad_words, 1);
        let box_list = storage_words(device, "view boxes", box_words.as_flattened(), 12);
        let crosser_list = storage_words(device, "view crossers", &crosser_words, 8);

        let placements: Vec<_> = (tiles.placements.iter())
            .map(|&TilePlacement { transform, .. }| {
                let grid_to_world = TyMatrix4x4F64::from_scale_rotation_translation(
                    transform.scale,
                    transform.rotation,
                    transform.position,
                );

                (transform, grid_to_world)
            })
            .collect();

        let place_bytes = place_bytes(&placements);

        let places = device.create_buffer(&BufferDescriptor {
            label: Some("view places"),
            size: place_bytes.len().max(2 * PLACE_BYTES) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // SAFETY: a fragment reads only within a tile's words because each
        // step keeps its cell inside the tile's box. Its loop ends within the
        // box's extent. A cell's place among its tile's record cells ends
        // within the tile's spare records. Its record then lies in the tile's
        // words, and its word in `lit` lies within the spare words that
        // buffer ends with. That word's material is under 256 and indexes
        // `materials` within the spare values that buffer ends with. A box's
        // entries in `crossers` index `boxes`. The stages index `places` by
        // an eye and a box's placement.
        let module = |traced, source, glass| unsafe {
            device.create_shader_module_trusted(
                ShaderModuleDescriptor {
                    label: Some(if traced {
                        "box view mirrors"
                    } else {
                        "box view"
                    }),
                    source: ShaderSource::Wgsl(
                        box_view_shader(traced, threshold, source, glass, lit.tints).into(),
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
        let glassy = (!glass.is_empty()).then(|| module(false, None, true));
        let near = usize::from(mirror.is_some());
        let modules: Vec<_> = [Some(&plain), mirror.as_ref(), glassy.as_ref()]
            .into_iter()
            .flatten()
            .collect();

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

        let halo = bloom.as_ref().map_or(&no_halo, BoxBloom::halo).clone();

        let bloom_sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("view bloom"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            ..SamplerDescriptor::default()
        });

        let count = |flags: &[bool]| flags.iter().filter(|&&flag| flag).count();

        let part = |fragment, faces, module, room| BoxPart {
            fragment,
            faces,
            module,
            room,
            glass: false,
        };

        // A box that crosses another draws in two parts when it lies near
        // the head, so the draw has room for each such box twice.
        let mut parts = vec![
            part("held", BoxFaces::Held, near, count(crossing.as_slice())),
            part("inside", BoxFaces::NearBack, near, 0),
            part("solid", BoxFaces::Front, 0, solid.len()),
            part("solid", BoxFaces::Front, near, 0),
            part("front", BoxFaces::Front, 0, marched.len()),
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
                        room: glass.len(),
                        glass: true,
                    },
                ),
            );
        }

        let storage = BufferBindingType::Storage { read_only: true };
        let both = ShaderStages::VERTEX | ShaderStages::FRAGMENT;

        // Each page binds its tiles' words and its lit words.
        let buffers: Vec<_> = (bits.iter().zip(&lit.buffers))
            .map(|(bits, lit)| {
                [
                    (both, BufferBindingType::Uniform, &frame),
                    (both, storage, &places),
                    (ShaderStages::FRAGMENT, storage, bits),
                    (ShaderStages::FRAGMENT, storage, &material_list),
                    (ShaderStages::FRAGMENT, storage, &box_list),
                    (ShaderStages::FRAGMENT, storage, &crosser_list),
                    (ShaderStages::FRAGMENT, storage, lit),
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

        let mut boxes = BoxDraw::new(device, &modules, format, &parts, &bindings(&halo));

        let glow = bloom.map(|bloom| {
            boxes.cover(device, &plain, "place_sky", "sky_halo", format);

            // The source's ray needs the ray queries the TLAS came with.
            let source = module(tlas.is_some(), Some(bloom.scale() as f32), false);
            let half = size.map(|side| side.div_ceil(2));

            let glow_crossed = (crossing.iter().zip(&marched_boxes.glow))
                .filter(|&(&crosses, &glows)| crosses && glows)
                .count();

            let parts = [
                part("held", BoxFaces::Held, 0, glow_crossed),
                part("inside", BoxFaces::NearBack, 0, 0),
                part(
                    "solid",
                    BoxFaces::Front,
                    0,
                    count(solid_boxes.glow.as_slice()),
                ),
                part(
                    "front",
                    BoxFaces::Front,
                    0,
                    count(marched_boxes.glow.as_slice()),
                ),
                part("crossed", BoxFaces::Front, 0, 0),
            ];

            Glow {
                boxes: BoxDraw::new(
                    device,
                    &[&source],
                    BoxBloom::FORMAT,
                    &parts,
                    &bindings(&no_halo),
                ),
                depth: texture(
                    "glow depth",
                    half,
                    Self::DEPTH,
                    TextureUsages::RENDER_ATTACHMENT,
                ),
                reach,
                bloom,
            }
        });

        let [red, green, blue] = [lights.sky.red, lights.sky.green, lights.sky.blue];
        let above = tonemap(TyLinSrgbF32::new(red as f32, green as f32, blue as f32));

        Ok(BoxView {
            boxes,
            glow,
            pages: pages.count,
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
            let index = ViewBoxes::index_of(key);
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
            let index = ViewBoxes::index_of(key);
            let mirror = usize::from(self.solid.mirror[index]);

            self.order[SOLID_RUN + mirror].push(index);
        }

        // The glowing boxes take the view's parts for the halo's draw.
        if let Some(glow) = &mut self.glow {
            let (near, far) = self.marched_keys.split_at(self.near);
            let crossing = &self.crossing;
            let solid = &self.solid;
            let marched = &self.marched;
            let glowing = |boxes: &'_ ViewBoxes, keys: &[u64]| -> Vec<UsizeId<BViewBox>> {
                (keys.iter().map(|&key| ViewBoxes::index_of(key)))
                    .filter(|&index| boxes.glow[index])
                    .collect()
            };

            let crossed = |keys: &[u64], crosses: bool| -> Vec<u64> {
                (keys.iter().copied())
                    .filter(|&key| crossing[ViewBoxes::index_of(key)] == crosses)
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
                .filter(|&index| !hide.is_some_and(|hide| hide.hides(list, index.to_usize())));

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
                let farthest_first = keys.iter().rev().map(|&key| ViewBoxes::index_of(key));

                (self.glass).deal_in_order(farthest_first, part, &mut self.sorted, &mut runs);
            }
        }

        self.boxes.rewrite(queue, &self.sorted, &runs);
    }

    /// Encodes a frame of the boxes [`prepare`](Self::prepare) sorted over
    /// `target`, whose color the first pass clears to `clear`. A scene that
    /// blooms draws its halo's source first.
    pub fn encode(&self, encoder: &mut CommandEncoder, target: &ViewTarget<'_>, clear: Color) {
        if let Some(glow) = &self.glow {
            let source = ViewTarget {
                color: glow.bloom.source(),
                depth: &glow.depth,
                keep_depth: false,
            };

            glow.boxes.encode(encoder, "glow", &source, Color::BLACK);
            glow.bloom.run(encoder);
        }

        self.boxes.encode(encoder, "eyes", target, clear);
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

/// What a view's boxes hold of one of an object's tiles.
#[derive(Clone, Copy)]
struct TileTraits {
    /// The place of the tile's first material among the view's.
    material: u32,

    /// Whether a fragment of the tile can cast a mirror ray.
    mirror: bool,

    /// Whether a fragment of the tile can meet an emission that blooms.
    glow: bool,
}

/// The test that keeps hidden boxes from drawing, for a scene of
/// [`TESTED`] boxes or more with a slab to hide one. `lists` holds a view's
/// marched boxes and then its solid ones, each as placed and as sorted.
fn hide_of(tiles: &TileScene, lists: [(&Vec<PlacedBox>, &ViewBoxes); 2]) -> Option<BoxHide> {
    let boxes: usize = lists.iter().map(|(placed, _)| placed.len()).sum();

    if boxes < TESTED {
        return None;
    }

    // Each placement's slabs in the world. An object's coarser copy keeps
    // none.
    let slabs: Vec<_> = (tiles.placements.iter())
        .flat_map(|TilePlacement { object, transform }| {
            tiles.objects[*object].slabs.iter().map(|&(low, high)| {
                box_corners(low.as_dvec3(), high.as_dvec3())
                    .map(|corner| transform.transform_point(corner))
            })
        })
        .collect();

    if slabs.is_empty() {
        return None;
    }

    // A list's boxes come a placement at a time.
    let lists = lists.map(|(placed, boxes)| {
        let runs = placed
            .chunk_by(|a, b| a.placement == b.placement)
            .map(<[PlacedBox]>::len)
            .collect();

        (boxes.bounds.as_vec().clone(), runs)
    });

    Some(BoxHide::new(slabs, lists.into()))
}

/// `box_view.wgsl`'s `Frame` for a target whose width and height are
/// `size`, with zeros where a frame writes its eyes.
fn frame_bytes(lights: &ViewLights, placements: usize, size: [u32; 2]) -> Vec<u8> {
    let black = TyLinSrgbF64::new(0.0, 0.0, 0.0);
    let (toward, sun) = lights.sun.unwrap_or((TyVector3F64::Y, black));
    let rgb = |light: TyLinSrgbF64| [light.red, light.green, light.blue];
    let mut frame = Vec::new();

    for light in [
        toward.to_array(),
        rgb(sun),
        rgb(lights.sky),
        rgb(lights.ground),
    ] {
        for value in light.into_iter().chain([0.0]) {
            frame.extend((value as f32).to_le_bytes());
        }
    }

    // The placements' count, then what a pixel spans of the target.
    for value in [to_word(placements), 0] {
        frame.extend(value.to_le_bytes());
    }

    for side in size {
        frame.extend((1.0 / side as f32).to_le_bytes());
    }

    frame.resize(FRAME_BYTES, 0);

    frame
}

/// `box_view.wgsl`'s `Place` for each of `placements`, once an eye. A
/// `Place` ends with what holds from frame to frame: the world direction of
/// each of its grid's axes with the meters a cell spans along it. Zeros
/// stand where a frame writes its matrix and its ray.
fn place_bytes(placements: &[(TyTransformF64, TyMatrix4x4F64)]) -> Vec<u8> {
    let mut bytes = vec![0; placements.len() * 2 * PLACE_BYTES];
    let each = placements.iter().chain(placements);

    for (place, (transform, _)) in bytes.chunks_exact_mut(PLACE_BYTES).zip(each) {
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

    bytes
}
