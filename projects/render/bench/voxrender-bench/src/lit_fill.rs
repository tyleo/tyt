use crate::{
    LitPage, PassTerms, TILE, TileMasks, TilePlacement, TileScene, bounds_around, box_corners,
    storage_words, to_word,
};
use ty_math::{TyTransformF64, TyVector3F64, TyVector3U32};
use voxrender::{grid_vector, quantize_direction};
use voxrender_wgpu::GRID_WALK_WGSL;
use wgpu::{
    BindGroupDescriptor, BindGroupEntry, Buffer, BufferDescriptor, BufferUsages, ComputePipeline,
    ComputePipelineDescriptor, Device, PipelineCompilationOptions, ShaderModuleDescriptor,
    ShaderRuntimeChecks, ShaderSource,
    util::{BufferInitDescriptor, DeviceExt},
};

/// The bytes of the two `Steps` that end `grid_walk.wgsl`'s `Placement`.
const STEP_BYTES: usize = 192;

/// The meters a caster's box grows by before the test against a tile's.
/// The margin covers the rounding of the boxes and of each grid's light.
const MARGIN: f64 = 0.01;

/// What fills a scene's corner shadows under one directional light on the
/// GPU: [`LitCorners`](crate::LitCorners)' words, a [`LitPage`] of the
/// scene's placements at a time.
///
/// A compute pass casts the contract's ray from each corner of each face
/// the light can reach, through voxrender's integer walk. A ray walks
/// only the placements that can shadow its tile: those whose box, seen
/// along the light, covers some of the tile's and reaches as far toward
/// the light.
///
/// A material that passes light tints the light a ray carries through it,
/// as the contract's shadow ray pays each surface. A corner is lit where
/// some light reaches it. A record cell's tint holds the mean of what
/// reaches its lit corners in each channel.
///
/// A glass cell is a cell of such a material with a surface. Its word
/// holds the mean of what reaches the corners of its surfaces, from the
/// side of each that the light falls on.
pub struct LitFill {
    pipeline: ComputePipeline,

    /// `grid_walk.wgsl`'s `Placement` for each placement.
    placements: Buffer,

    /// Each object's [`CasterTiles`](crate::CasterTiles)' words.
    casters: Buffer,

    /// `lit_fill.wgsl`'s `Light` for each placement.
    lights: Buffer,

    /// `lit_fill.wgsl`'s `passes`.
    passes: Buffer,

    /// The light's axis after two axes across it.
    axes: [TyVector3F64; 3],

    /// The low and high corners of the box around each placement's grid
    /// along [`axes`](Self::axes).
    spans: Vec<Span>,

    /// How many cells of the object it copies one of each placement's
    /// cells spans.
    sizes: Vec<u32>,
}

/// The low and high corners of a box along a light's axes.
type Span = (TyVector3F64, TyVector3F64);

impl LitFill {
    /// Uploads what every page's fill of `tiles` reads, for a light along
    /// the unit `toward`.
    pub fn new(device: &Device, tiles: &TileScene, toward: TyVector3F64) -> Self {
        let mut caster_words = Vec::new();
        let mut offsets = Vec::new();

        for object in &tiles.objects {
            offsets.push(to_word(caster_words.len()));
            caster_words.extend(&object.casters.words);
        }

        // `lit_fill.wgsl`'s `passes`: three `vec4<f32>`s for each key, a
        // material's place plus one. Key zero passes the light whole.
        let passes: Vec<f32> = [[0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]]
            .into_iter()
            .chain(tiles.materials.iter().map(|material| {
                let PassTerms {
                    most,
                    uncovered,
                    reflectance,
                    rise,
                } = PassTerms::new(material);

                [
                    most.red,
                    most.green,
                    most.blue,
                    uncovered,
                    reflectance.red,
                    reflectance.green,
                    reflectance.blue,
                    0.0,
                    rise.red,
                    rise.green,
                    rise.blue,
                    0.0,
                ]
            }))
            .flatten()
            .map(|value| value as f32)
            .collect();

        // The fill leaves a view's steps in a `Placement` unread.
        let mut placement_bytes = Vec::new();
        let mut lights = Vec::new();

        for TilePlacement { object, transform } in &tiles.placements {
            let bounds = tiles.objects[*object].bounds.to_array();
            let rotation = transform.rotation.to_array();

            placement_bytes.extend(bounds.into_iter().chain([0]).flat_map(u32::to_le_bytes));
            placement_bytes.extend(
                [offsets[*object], 0, 0, 0]
                    .into_iter()
                    .flat_map(u32::to_le_bytes),
            );

            let floats = (transform.scale.to_array().into_iter().chain([0.0]))
                .chain(transform.position.to_array().into_iter().chain([0.0]))
                .chain(rotation);

            placement_bytes.extend(floats.flat_map(|value| (value as f32).to_le_bytes()));
            placement_bytes.resize(placement_bytes.len() + STEP_BYTES, 0);

            let [x, y, z] = quantize_direction(grid_vector(transform, toward));

            let fades = [TyVector3F64::X, TyVector3F64::Y, TyVector3F64::Z].map(|axis| {
                let cosine = (transform.rotation * axis).dot(toward).abs();

                (1.0 - cosine).powi(5) as f32
            });

            lights.extend([x, y, z, 0].into_iter().flat_map(i32::to_le_bytes));
            lights.extend(fades.into_iter().chain([0.0]).flat_map(f32::to_le_bytes));
        }

        // A binding cannot be empty.
        if placement_bytes.is_empty() {
            placement_bytes.resize(80 + STEP_BYTES, 0);
            lights.resize(32, 0);
        }

        // Each span reaches along the light's axis and two axes across it.
        let across = toward.cross(if toward.x.abs() < 0.5 {
            TyVector3F64::X
        } else {
            TyVector3F64::Y
        });
        let across = across.normalize();
        let axes = [across, toward.cross(across), toward];

        let spans = (tiles.placements.iter())
            .map(|TilePlacement { object, transform }| {
                span(
                    &axes,
                    transform,
                    TyVector3U32::ZERO,
                    tiles.objects[*object].bounds,
                )
            })
            .collect();

        let init = |label, contents: &[u8]| {
            device.create_buffer_init(&BufferInitDescriptor {
                label: Some(label),
                contents,
                usage: BufferUsages::STORAGE,
            })
        };

        // SAFETY: an invocation past a page's cells returns. A cell
        // indexes its placed tile, which indexes its placement and its
        // casters. The walk reads a cell only inside its placement's grid,
        // and each of its loops ends within a grid's extent. A cell inside
        // a grid lies in one of its tiles, whose word leads to the tile's
        // words. A cell's key indexes `passes`, which holds each key up to
        // the greatest.
        let module = unsafe {
            device.create_shader_module_trusted(
                ShaderModuleDescriptor {
                    label: Some("lit fill"),
                    source: ShaderSource::Wgsl(
                        format!("{GRID_WALK_WGSL}\n{}", include_str!("lit_fill.wgsl")).into(),
                    ),
                },
                ShaderRuntimeChecks::unchecked(),
            )
        };

        LitFill {
            pipeline: device.create_compute_pipeline(&ComputePipelineDescriptor {
                label: Some("lit fill"),
                layout: None,
                module: &module,
                entry_point: Some("main"),
                compilation_options: PipelineCompilationOptions::default(),
                cache: None,
            }),
            placements: init("lit fill placements", &placement_bytes),
            casters: storage_words(device, "lit fill cells", &caster_words, 8),
            lights: init("lit fill lights", &lights),
            passes: init("lit fill passes", bytemuck::cast_slice(&passes)),
            axes,
            spans,
            sizes: (tiles.placements.iter())
                .map(|placed| tiles.objects[placed.object].span)
                .collect(),
        }
    }

    /// Uploads what the fill of some of `tiles`' placements reads, and
    /// makes the buffer their words fill.
    ///
    /// # Arguments
    ///
    /// 1. `tiles`: the scene the fill was built for
    /// 2. `placements`: the placements, as their places among the scene's
    /// 3. `tints`: for a scene with a material that passes light, how far
    ///    past a record cell's word its tint lies, no less than the page's
    ///    cells plus [`TileMasks::SPARE`]
    pub fn page(
        &self,
        device: &Device,
        tiles: &TileScene,
        placements: &[usize],
        tints: Option<u32>,
    ) -> LitPage {
        let mut places = Vec::new();
        let mut glass_places = Vec::new();
        let mut tile_words: Vec<u32> = Vec::new();
        let mut casters: Vec<u32> = Vec::new();
        let mut recorded: Vec<u32> = Vec::new();

        // The opaque tiles' cells of every placement come first, then the
        // glass tiles'.
        for glassy in [false, true] {
            for &index in placements {
                let TilePlacement { object, transform } = &tiles.placements[index];
                let object = &tiles.objects[*object];

                if glassy {
                    glass_places.push(to_word(recorded.len()));
                } else {
                    places.push(to_word(recorded.len()));
                }

                // Each tile's low corner and the indices of its cells that
                // take a word.
                let cells: Vec<(TyVector3U32, &Vec<u16>)> = if glassy {
                    (object.glass.boxes.iter().zip(&object.glass.tiles))
                        .map(|(drawn, tile)| (drawn.cells.0 / TILE * TILE, &tile.cells))
                        .collect()
                } else {
                    (object.masks.tiles.iter())
                        .map(|tile| (tile.corner, &tile.recorded))
                        .collect()
                };

                for (corner, cells) in cells {
                    let tile = u32::try_from(tile_words.len() / 8)
                        .ok()
                        .filter(|&tile| tile < 1 << 20)
                        .expect("a page's placed tiles number under 2^20");

                    let (least, most) = span(
                        &self.axes,
                        transform,
                        corner,
                        (corner + TILE).min(object.bounds),
                    );
                    let first = casters.len();

                    // A caster's box covers some of the tile's across the light
                    // and reaches as far toward it. Its cells span as many
                    // of its object's as the tile's do: an object's coarser
                    // copy shadows that copy's cells alone.
                    let size = self.sizes[index];

                    casters.extend((0u32..).zip(&self.spans).zip(&self.sizes).filter_map(
                        |((caster, (low, high)), &cast)| {
                            let covers = (0..2).all(|axis| {
                                low[axis] - MARGIN <= most[axis]
                                    && high[axis] + MARGIN >= least[axis]
                            });

                            (cast == size && covers && high[2] + MARGIN >= least[2])
                                .then_some(caster)
                        },
                    ));

                    tile_words.extend([
                        corner.x,
                        corner.y,
                        corner.z,
                        to_word(index),
                        to_word(first),
                        to_word(casters.len() - first),
                        0,
                        0,
                    ]);

                    recorded.extend(cells.iter().map(|&cell| tile << 12 | u32::from(cell)));
                }
            }
        }

        let cells = to_word(recorded.len());
        let words = cells + TileMasks::SPARE;

        assert!(
            tints.is_none_or(|tints| tints >= words),
            "a tint lies past the page's words"
        );

        let frame = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("lit fill frame"),
            contents: bytemuck::cast_slice(&[cells, 0, tints.unwrap_or(0), 0]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let tile_words = storage_words(device, "lit fill tiles", &tile_words, 8);
        let casters = storage_words(device, "lit fill casters", &casters, 8);
        let recorded = storage_words(device, "lit fill record cells", &recorded, 8);

        let lit = device.create_buffer(&BufferDescriptor {
            label: Some("lit corners"),
            size: u64::from(tints.unwrap_or(0) + words) * 4,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_SRC | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("lit fill"),
            layout: &self.pipeline.get_bind_group_layout(0),
            entries: &[
                &frame,
                &self.placements,
                &self.casters,
                &self.lights,
                &tile_words,
                &casters,
                &recorded,
                &lit,
                &self.passes,
            ]
            .iter()
            .zip(0u32..)
            .map(|(buffer, binding)| BindGroupEntry {
                binding,
                resource: buffer.as_entire_binding(),
            })
            .collect::<Vec<_>>(),
        });

        LitPage {
            places,
            glass_places,
            lit,
            pipeline: self.pipeline.clone(),
            bind_group,
            frame,
            cells,
        }
    }
}

/// The low and high corners, along `axes`, of the box around the cells
/// from `low` to before `high` of a grid that `transform` places.
fn span(
    axes: &[TyVector3F64; 3],
    transform: &TyTransformF64,
    low: TyVector3U32,
    high: TyVector3U32,
) -> Span {
    bounds_around(box_corners(low.as_dvec3(), high.as_dvec3()).map(|corner| {
        let point = transform.transform_point(corner);

        TyVector3F64::from_array(axes.map(|axis| axis.dot(point)))
    }))
}
