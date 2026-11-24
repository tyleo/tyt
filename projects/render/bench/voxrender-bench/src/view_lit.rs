use crate::{LitFill, Result, TileMasks, TileScene, ViewPages, to_word};
use ty_math::TyVector3F64;
use wgpu::{
    Buffer, BufferUsages, Device, Queue,
    util::{BufferInitDescriptor, DeviceExt},
};

/// A view's lit words: a word for each record cell of each placement's
/// tiles, then one for each of its glass cells. A record cell's word holds
/// its material in its high 8 bits, as its place among its tile's. The low
/// bits take the sun's corner shadows.
pub struct ViewLit {
    /// `box_view.wgsl`'s `lit` for each page. Each ends with
    /// [`TileMasks::SPARE`] words, which a hit the depth test drops can
    /// read past its tile's last record cell's.
    pub buffers: Vec<Buffer>,

    /// The first of each placement's words in its page's.
    pub places: Vec<u32>,

    /// The first word of each of each placement's glass tiles in its
    /// page's.
    pub glass: Vec<Vec<u32>>,

    /// How far past a record cell's word its tint lies, or zero for a
    /// scene the sun's light takes no tint in.
    pub tints: u32,
}

impl ViewLit {
    /// Lays out the lit words of `tiles`' placements, a page of `pages` in
    /// a buffer, and has the GPU fill the shadows of a sun along the unit
    /// `toward`. A scene with no sun leaves them zero, and its faces skip
    /// the sun's term. In a `tinted` scene, the fill adds each record
    /// cell's tint in a second word. Errors if waiting on the fill fails.
    pub fn new(
        device: &Device,
        queue: &Queue,
        tiles: &TileScene,
        pages: &ViewPages,
        toward: Option<TyVector3F64>,
        tinted: bool,
    ) -> Result<Self> {
        let placements = &tiles.placements;
        let mut places = vec![0; placements.len()];
        let mut glass: Vec<Vec<u32>> = vec![Vec::new(); placements.len()];

        // Each page's placements and words.
        let laid: Vec<(Vec<usize>, Vec<u32>)> = (0..pages.count)
            .map(|page| {
                let placed: Vec<_> = (0..placements.len())
                    .filter(|&placement| pages.of[placements[placement].object] == page)
                    .collect();

                let object = |placement: usize| &tiles.objects[placements[placement].object];
                let mut words: Vec<u32> = Vec::new();

                for &placement in &placed {
                    places[placement] = to_word(words.len());

                    words.extend(
                        (object(placement).masks.tiles.iter())
                            .flat_map(|tile| &tile.entries)
                            .map(|&entry| u32::from(entry) << 24),
                    );
                }

                for &placement in &placed {
                    glass[placement] = (object(placement).glass.tiles.iter())
                        .map(|tile| {
                            let first = to_word(words.len());

                            words.resize(words.len() + tile.cells.len(), 0);

                            first
                        })
                        .collect();
                }

                words.extend([0; TileMasks::SPARE as usize]);

                (placed, words)
            })
            .collect();

        // A tint lies as far past its cell's word in every page.
        let tints = tinted.then(|| {
            let most = laid.iter().map(|(_, words)| words.len()).max();

            to_word(most.expect("a view has a page"))
        });

        let fill = toward.map(|toward| LitFill::new(device, tiles, toward));

        let buffers = laid
            .iter()
            .map(|(placed, words)| {
                let Some(fill) = &fill else {
                    return Ok(device.create_buffer_init(&BufferInitDescriptor {
                        label: Some("view lit corners"),
                        contents: bytemuck::cast_slice(words),
                        usage: BufferUsages::STORAGE,
                    }));
                };

                let page = fill.page(device, tiles, placed, tints);

                // A placement with no glass tile takes the fill's word for
                // where its first would lie.
                let firsts: Vec<_> = placed.iter().map(|&placement| places[placement]).collect();
                let glass_firsts: Vec<_> = (placed.iter().zip(&page.glass_places))
                    .map(|(&placement, &place)| glass[placement].first().copied().unwrap_or(place))
                    .collect();

                assert_eq!(
                    (&page.places, &page.glass_places),
                    (&firsts, &glass_firsts),
                    "the fill lays out the view's words"
                );

                queue.write_buffer(&page.lit, 0, bytemuck::cast_slice(words));
                page.fill(device, queue)?;

                Ok(page.lit)
            })
            .collect::<Result<_>>()?;

        Ok(ViewLit {
            buffers,
            places,
            glass,
            tints: tints.unwrap_or(0),
        })
    }
}
