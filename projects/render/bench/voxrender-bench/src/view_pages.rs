use crate::{BTileObject, BViewPage, Error, Result, TileMasks, TileScene, to_word};
use branded_id::{IdVec, UsizeId};
use wgpu::{Buffer, BufferDescriptor, BufferUsages, Device};

/// The pages a view's objects fill, one after another. A page holds its
/// objects' tiles in one buffer and its placements' lit words in another,
/// each within what one storage binding holds.
pub struct ViewPages {
    /// The page that holds each of the scene's objects.
    pub of: IdVec<BTileObject, UsizeId<BViewPage>>,

    /// How many pages the scene fills, at least one.
    pub count: usize,
}

impl ViewPages {
    /// Deals `tiles`' objects into pages of `page_bytes` bytes. Errors if
    /// the objects that share a page pass one.
    ///
    /// # Arguments
    ///
    /// 1. `crossing`: each pair of objects whose boxes share space. They
    ///    take one page, because a fragment that marches a box of another
    ///    placement reads that box's tile
    /// 2. `tinted`: whether a page's lit words take half a binding and
    ///    leave the rest to their tints
    pub fn new(
        tiles: &TileScene,
        crossing: impl IntoIterator<Item = [UsizeId<BTileObject>; 2]>,
        page_bytes: u64,
        tinted: bool,
    ) -> Result<Self> {
        // The objects by their places, which a group's members follow one
        // another in.
        let objects = tiles.objects.as_slice();
        let mut placed = vec![0; objects.len()];

        for placement in &tiles.placements {
            placed[placement.object.to_usize()] += 1;
        }

        // The words of each object's tiles, and of what its placements'
        // cells take in lit words.
        let sizes: Vec<(usize, usize)> = (objects.iter().zip(&placed))
            .map(|(object, placed)| {
                let masks = &object.masks;
                let records: usize = masks.tiles.iter().map(|tile| tile.recorded.len()).sum();
                let glass: usize = object.glass.tiles.iter().map(|tile| tile.cells.len()).sum();

                (
                    masks.words.len() + object.glass.words.len(),
                    (records + glass) * placed,
                )
            })
            .collect();

        // Each object's group, as the first of the objects it shares a page
        // with.
        let mut group: Vec<_> = (0..objects.len()).collect();

        for [a, b] in crossing {
            let (a, b) = (group[a.to_usize()], group[b.to_usize()]);

            if a != b {
                for joined in &mut group {
                    if *joined == a.max(b) {
                        *joined = a.min(b);
                    }
                }
            }
        }

        // The objects come before their coarser copies, finest first, so a
        // view of copies alone draws from the pages that hold them.
        let mut firsts: Vec<_> = (0..objects.len())
            .filter(|&object| group[object] == object)
            .collect();

        firsts.sort_by_key(|&first| objects[first].span);

        let most = (page_bytes / 4) as usize;
        let most_lit = (if tinted { most / 2 } else { most }) - TileMasks::SPARE as usize;
        let mut of = vec![UsizeId::from_usize(0); objects.len()];
        let mut filled = (0, 0);
        let mut count = 1;

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
                count += 1;
                filled = (0, 0);
            }

            filled = (filled.0 + bits, filled.1 + lit);

            for object in members() {
                of[object] = UsizeId::from_usize(count - 1);
            }
        }

        Ok(ViewPages {
            of: IdVec::from_vec(of),
            count,
        })
    }

    /// `box_view.wgsl`'s `bits` for each page: each of its objects' mask
    /// words, then the object's glass tiles' words. Also returns where each
    /// object's words start in its page's.
    pub fn bits(
        &self,
        device: &Device,
        tiles: &TileScene,
    ) -> (Vec<Buffer>, IdVec<BTileObject, u32>) {
        let mut starts: Vec<u32> = vec![0; tiles.objects.len()];

        let buffers = (0..self.count)
            .map(|page| {
                let held: Vec<_> = (tiles.objects.iter().zip(&self.of).zip(0..))
                    .filter(|((_, of), _)| of.to_usize() == page)
                    .map(|((object, _), index)| (object, index))
                    .collect();

                let mut words = 0;

                for &(object, index) in &held {
                    starts[index] = to_word(words);
                    words += object.masks.words.len() + object.glass.words.len();
                }

                // A binding cannot be empty.
                let buffer = device.create_buffer(&BufferDescriptor {
                    label: Some("view bits"),
                    size: 4 * words.max(1) as u64,
                    usage: BufferUsages::STORAGE,
                    mapped_at_creation: true,
                });

                let held = (held.iter())
                    .flat_map(|(object, _)| object.masks.words.iter().chain(&object.glass.words));

                let spare = if words == 0 { Some(&0) } else { None };

                // The words go straight to the buffer, where a landscape's
                // would not fit a second time.
                buffer
                    .get_mapped_range_mut(..)
                    .expect("a buffer mapped at creation gives its bytes")
                    .slice(..)
                    .write_iter(held.chain(spare).flat_map(|word| word.to_le_bytes()));

                buffer.unmap();

                buffer
            })
            .collect();

        (buffers, IdVec::from_vec(starts))
    }
}
