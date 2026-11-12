use crate::{
    BoxBindings, BoxCrosser, BoxDraw, BoxFaces, BoxPart, HitTargets, LitFill, MaterialGrid,
    PlacedBox, QuadTlas, Result, TileMasks, ViewEye, box_view_shader,
};
use branded_id::U32Id;
use std::{collections::HashMap, num::NonZeroU32};
use ty_math::{TyLinSrgbF32, TyLinSrgbF64, TyMatrix4x4F64, TyTransformF64, TyVector3F64};
use voxcore::material;
use voxrender::{RenderLight, RenderScene, grid_point, tonemap};
use voxsurface::{SurfaceMethod, mesh_grid};
use wgpu::{
    AddressMode, Buffer, BufferBindingType, BufferDescriptor, BufferUsages, Color, Device,
    Extent3d, Features, FilterMode, Queue, RenderPass, SamplerDescriptor, ShaderModuleDescriptor,
    ShaderRuntimeChecks, ShaderSource, ShaderStages, TextureDescriptor, TextureDimension,
    TextureFormat, TextureUsages, TextureViewDescriptor,
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

/// The meters past the eyes within which a box counts as holding an eye,
/// in a headset's meters. The reach covers the near plane.
const REACH: f64 = 0.02;

/// The meters a head moves from where the boxes last sorted before they
/// sort again, in a headset's meters.
const SLACK: f64 = 0.25;

/// A shaded view of a scene's boxes for both eyes of a headset: the box
/// raster with solid boxes, a mask march, and face depth. Each frame takes
/// its eyes and draws in multiview passes over a two-layer target with
/// depth in [`BoxView::DEPTH`]. The boxes draw nearest first. They sort
/// again once the head has moved [`SLACK`] from where they last sorted.
///
/// A hit shades as the contract's opaque surface under the scene's first
/// sun and its hemispheres, with corner occlusion and the sun's corner
/// shadows. On a device with ray queries a face's mirror ray traces the
/// scene's opaque quads. The quad it meets shades flat with no shadow.
/// Elsewhere a face reflects the sky alone. Each fragment
/// adds the bloom at its pixel before the tonemap, from a texture a quarter
/// of the target's size that stays black until a pass fills it.
pub struct BoxView {
    boxes: BoxDraw,

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

    /// Whether each box's fragments can cast a mirror ray.
    mirror: Vec<bool>,
}

impl ViewBoxes {
    fn push(&mut self, placed: &PlacedBox, words: [u32; 12], mirror: bool) {
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
    }

    /// The keys of `keys` whose boxes have `mirror`.
    fn of<'a>(
        &'a self,
        keys: impl IntoIterator<Item = u64> + 'a,
        mirror: bool,
    ) -> impl Iterator<Item = u64> + 'a {
        keys.into_iter()
            .filter(move |&key| self.mirror[index_of(key)] == mirror)
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
    /// A mirror ray runs to where what it meets would show by this share
    /// of its face's light. A face that reflects less casts none.
    pub const MIRROR_CUT: f32 = 1.0 / 64.0;

    /// The least cells a mirror ray reaches. A face whose ray would end
    /// sooner casts none: what touches it shows too faintly to earn a ray.
    pub const MIRROR_REACH: f32 = 2.0;

    /// The format of the depth a pass that draws the view clears to zero.
    pub const DEPTH: TextureFormat = HitTargets::DEPTH;

    /// Builds the view of `scene` for a two-layer target of `format` and
    /// of `size`, its width and height, and submits the fill of the sun's
    /// corner shadows. Errors if a tile holds over 256 materials, or if the
    /// scene has a sun and no view.
    pub fn new(
        device: &Device,
        queue: &Queue,
        scene: &RenderScene,
        format: TextureFormat,
        size: [u32; 2],
    ) -> Result<Self> {
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

        for (object_id, object) in scene.iter_objects() {
            let masks = TileMasks::new(object, true, Some(&opaque))?;

            offsets.insert(
                object_id,
                u32::try_from(words.len()).expect("a scene's mask words number under 2^32"),
            );

            words.extend(masks.words);
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

        lit_words.extend([0; TileMasks::SPARE as usize]);

        // The sun's corner shadows, which the GPU fills into the words' low
        // bits before the view's first draw. A scene with no sun leaves them
        // zero, and its faces skip the sun's term.
        let lit_corners = if suns > 0 {
            let fill = LitFill::new(device, scene, sun.0, &object_recorded)?;

            assert_eq!(
                fill.places, lit_places,
                "the fill lays out the view's words"
            );

            queue.write_buffer(&fill.lit, 0, &words_bytes(&lit_words));
            fill.fill(device, queue);

            fill.lit
        } else {
            device.create_buffer_init(&BufferInitDescriptor {
                label: Some("view lit corners"),
                contents: &words_bytes(&lit_words),
                usage: BufferUsages::STORAGE,
            })
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
        let mut materials: Vec<f32> = Vec::new();
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

        let mut solid_boxes = ViewBoxes::default();
        let mut marched_boxes = ViewBoxes::default();

        for ((placed, &words), &mirror) in solid.iter().zip(&solid_words).zip(&solid_mirrors) {
            solid_boxes.push(placed, words, mirror);
        }

        for ((placed, &words), &mirror) in marched.iter().zip(&march_words).zip(&march_mirrors) {
            marched_boxes.push(placed, words, mirror);
        }

        let crossing: Vec<bool> = crossers.iter().map(|entries| !entries.is_empty()).collect();

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

        let tlas = mirrors.then(|| {
            let quads = QuadTlas::new(
                device,
                queue,
                scene,
                |object| {
                    mesh_grid(
                        &MaterialGrid {
                            object,
                            opaque: &opaque,
                        },
                        SurfaceMethod::Greedy,
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
        let module = |traced| unsafe {
            device.create_shader_module_trusted(
                ShaderModuleDescriptor {
                    label: Some(if traced {
                        "box view mirrors"
                    } else {
                        "box view"
                    }),
                    source: ShaderSource::Wgsl(box_view_shader(traced).into()),
                },
                ShaderRuntimeChecks::unchecked(),
            )
        };

        // A mirror ray's query costs each fragment of its module, so only
        // the boxes that can cast one draw through it.
        let plain = module(false);
        let mirror = mirrors.then(|| module(true));
        let modules: Vec<_> = [Some(&plain), mirror.as_ref()]
            .into_iter()
            .flatten()
            .collect();
        let near = modules.len() - 1;

        let storage = BufferBindingType::Storage { read_only: true };
        let both = ShaderStages::VERTEX | ShaderStages::FRAGMENT;

        // The bloom each eye's fragments add. wgpu clears it to black.
        let bloom = device
            .create_texture(&TextureDescriptor {
                label: Some("view bloom"),
                size: Extent3d {
                    width: size[0].div_ceil(4),
                    height: size[1].div_ceil(4),
                    depth_or_array_layers: 2,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba16Float,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            })
            .create_view(&TextureViewDescriptor::default());

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

        let part = |fragment, faces, module, boxes| BoxPart {
            fragment,
            faces,
            module,
            boxes,
        };

        let boxes = BoxDraw::new(
            device,
            &modules,
            format,
            &[
                part("held", BoxFaces::Held, near, held_words),
                part("inside", BoxFaces::NearBack, near, Vec::new()),
                part("solid", BoxFaces::Front, 0, solid_words),
                part("solid", BoxFaces::Front, near, Vec::new()),
                part("front", BoxFaces::Front, 0, march_words),
                part("front", BoxFaces::Front, near, Vec::new()),
                part("crossed", BoxFaces::Front, 0, Vec::new()),
                part("crossed", BoxFaces::Front, near, Vec::new()),
            ],
            &BoxBindings {
                buffers: &[
                    (both, BufferBindingType::Uniform, &frame),
                    (both, storage, &places),
                    (ShaderStages::FRAGMENT, storage, &bits),
                    (ShaderStages::FRAGMENT, storage, &materials),
                    (ShaderStages::FRAGMENT, storage, &box_list),
                    (ShaderStages::FRAGMENT, storage, &crosser_list),
                    (ShaderStages::FRAGMENT, storage, &lit_corners),
                    (ShaderStages::FRAGMENT, storage, &quad_list),
                ],
                sampled: Some((&bloom, &bloom_sampler)),
                tlas: tlas.as_ref(),
            },
            NonZeroU32::new(0b11),
        );

        let above = tonemap(TyLinSrgbF32::new(
            sky.red as f32,
            sky.green as f32,
            sky.blue as f32,
        ));

        Ok(BoxView {
            boxes,
            frame,
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

    /// Readies the next draw for `eyes`. The boxes sort again when the head
    /// has left the last sort's reach. A meter of the headset's spans `scale`
    /// of the scene's meters.
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
    }

    /// Sorts the boxes nearest to `head` first into their runs. A marched
    /// box within `covered` of `head` draws its back faces because an eye
    /// can come to stand inside it before the next sort. Such a box that
    /// crosses another placement's box draws its back faces at the nearest
    /// depth for an eye it holds, and its front faces for one it does not.
    /// Every solid box draws its front faces because a ray that starts
    /// inside one sees none of it.
    fn sort(&mut self, queue: &Queue, head: [f32; 3], covered: f32) {
        self.solid
            .nearest_first(head, covered, &mut self.solid_keys);

        let within = self
            .marched
            .nearest_first(head, covered, &mut self.marched_keys);
        let (near, far) = self.marched_keys.split_at(within);
        let crossing = &self.crossing;

        self.sorted.clear();

        let solid = &self.solid;
        let marched = &self.marched;
        let plain = || crossing_keys(far, crossing, false);
        let crossed = || crossing_keys(&self.marched_keys, crossing, true);

        // Each run of front faces draws twice: the boxes with no mirror,
        // then those with one.
        let counts = [
            marched.extend(crossing_keys(near, crossing, true), &mut self.sorted),
            marched.extend(crossing_keys(near, crossing, false), &mut self.sorted),
            solid.extend(
                solid.of(self.solid_keys.iter().copied(), false),
                &mut self.sorted,
            ),
            solid.extend(
                solid.of(self.solid_keys.iter().copied(), true),
                &mut self.sorted,
            ),
            marched.extend(marched.of(plain(), false), &mut self.sorted),
            marched.extend(marched.of(plain(), true), &mut self.sorted),
            marched.extend(marched.of(crossed(), false), &mut self.sorted),
            marched.extend(marched.of(crossed(), true), &mut self.sorted),
        ];

        self.boxes.rewrite(queue, &self.sorted, &counts);
        self.sorted_for = Some((head, covered));
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
}
