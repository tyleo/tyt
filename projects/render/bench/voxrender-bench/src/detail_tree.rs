use crate::{PlacedBox, TileScene};
use std::ops::Range;
use ty_math::{TyVector3F64, TyVector3U32};

/// The cells along each side of a tile.
const TILE: u32 = 16;

/// The lists of a view's boxes a tree's tiles hold: the marched boxes, the
/// solid ones, and the glass ones.
pub const LISTS: usize = 3;

/// A scene's tiles and those of its objects' coarser copies, as a view
/// picks which of them to draw. A tile of a copy stands over the eight
/// tiles of the copy a level finer that fill its space. A view draws a tile
/// whose cells span few enough pixels, and the tiles under it otherwise.
/// Every place then draws at one level.
pub struct DetailTree {
    /// Each tile, the tiles under one following one another. The scene's
    /// coarsest tiles come first.
    tiles: Vec<Tile>,

    /// How many of [`tiles`](Self::tiles) are the scene's coarsest.
    roots: usize,
}

struct Tile {
    /// The low and then the high corner of the world box around the tile.
    bounds: [f32; 6],

    /// The meters one of the tile's cells spans along its widest axis.
    cell: f32,

    /// The tiles under the tile among the tree's.
    under: Range<u32>,

    /// The tile's boxes among each of a view's lists.
    boxes: [Range<u32>; LISTS],
}

impl DetailTree {
    /// The tree of `tiles`' tiles, or `None` for a scene whose objects
    /// keep no coarser copy, where every tile draws.
    ///
    /// # Arguments
    ///
    /// 1. `lists`: a view's marched, solid, and glass boxes, each as
    ///    [`PlacedBox::list`] gives them
    pub fn new(tiles: &TileScene, lists: [&[PlacedBox]; LISTS]) -> Option<Self> {
        if tiles.spans.iter().all(|&span| span == 1) {
            return None;
        }

        // The first box of each placement in each list.
        let firsts = lists.map(|list| {
            (0..tiles.placements.len() as u32)
                .map(|placement| list.partition_point(|placed| placed.placement < placement))
                .collect::<Vec<_>>()
        });

        // The boxes in each list of the tile at `tile` among a placement's
        // object's, or `None` for a tile with no box.
        let boxes_of = |placement: usize, tile: TyVector3U32| {
            let object = &tiles.objects[tiles.placements[placement].0];
            let masks = &object.masks;

            // The boxes of a tile follow one another in its object's list.
            let held = |places: &[u32], place: Option<usize>, first: usize| {
                let Some(place) = place else {
                    return first..first;
                };

                let low = places.partition_point(|&held| (held as usize) < place);
                let high = places.partition_point(|&held| held as usize <= place);

                first + low..first + high
            };

            let opaque = object.tile_at(tile);
            let glass = object.glass_at(tile);

            let ranges = [
                held(&masks.places, opaque, firsts[0][placement]),
                held(&masks.solid_places, opaque, firsts[1][placement]),
                // A glass tile draws one box, unless the view draws none.
                glass.map_or(0..0, |glass| {
                    let first = firsts[2][placement] + glass;

                    first..(first + 1).min(lists[2].len())
                }),
            ];

            let count = |range: Range<usize>| {
                let index =
                    |index: usize| u32::try_from(index).expect("a view's boxes number under 2^32");

                index(range.start)..index(range.end)
            };

            (opaque.is_some() || glass.is_some()).then(|| ranges.map(count))
        };

        let tile_of = |placement: usize, tile: TyVector3U32, boxes: [Range<u32>; LISTS]| {
            let (object, transform) = &tiles.placements[placement];
            let low = tile * TILE;
            let high = (low + TILE).min(tiles.objects[*object].bounds);

            let (least, most) = (0..8u32)
                .map(|corner| {
                    let far = TyVector3U32::new(corner & 1, corner >> 1 & 1, corner >> 2);

                    transform.transform_point((low + (high - low) * far).as_dvec3())
                })
                .fold(
                    (
                        TyVector3F64::splat(f64::INFINITY),
                        TyVector3F64::splat(f64::NEG_INFINITY),
                    ),
                    |(least, most), corner| (least.min(corner), most.max(corner)),
                );

            let [low_x, low_y, low_z] = least.as_vec3().to_array();
            let [high_x, high_y, high_z] = most.as_vec3().to_array();

            Tile {
                bounds: [low_x, low_y, low_z, high_x, high_y, high_z],
                cell: transform.scale.abs().max_element() as f32,
                under: 0..0,
                boxes,
            }
        };

        // Each tile so far with its placement and its place among its
        // object's tiles.
        let mut tree: Vec<Tile> = Vec::new();
        let mut placed: Vec<(usize, TyVector3U32)> = Vec::new();

        // A placement with no coarser copy after it draws its object's
        // coarsest. The placement before one that has a copy draws the
        // copy a level finer.
        for placement in (0..tiles.placements.len()).filter(|&at| tiles.coarser(at).is_none()) {
            let counts = tiles.objects[tiles.placements[placement].0].counts();

            for x in 0..counts.x {
                for y in 0..counts.y {
                    for z in 0..counts.z {
                        let tile = TyVector3U32::new(x, y, z);

                        if let Some(boxes) = boxes_of(placement, tile) {
                            tree.push(tile_of(placement, tile, boxes));
                            placed.push((placement, tile));
                        }
                    }
                }
            }
        }

        let roots = tree.len();
        let mut next = 0;

        while next < tree.len() {
            let (placement, tile) = placed[next];
            let first = tree.len();

            if let Some(finer) = placement
                .checked_sub(1)
                .filter(|&finer| tiles.coarser(finer) == Some(placement))
            {
                let counts = tiles.objects[tiles.placements[finer].0].counts();

                for corner in 0..8u32 {
                    let under =
                        tile * 2 + TyVector3U32::new(corner >> 2, corner >> 1 & 1, corner & 1);

                    if under.cmplt(counts).all()
                        && let Some(boxes) = boxes_of(finer, under)
                    {
                        tree.push(tile_of(finer, under, boxes));
                        placed.push((finer, under));
                    }
                }
            }

            let index =
                |index: usize| u32::try_from(index).expect("a scene's tiles number under 2^32");

            tree[next].under = index(first)..index(tree.len());
            next += 1;
        }

        Some(DetailTree { tiles: tree, roots })
    }

    /// Lists the boxes that draw for a head at `head`, each list's as
    /// their places in it, after clearing `shown`. A tile draws where one
    /// of its cells spans `finest` or less of its distance from the head,
    /// or where no tile lies under it. The tiles under it draw otherwise.
    pub fn select(&self, head: [f32; 3], finest: f32, shown: &mut [Vec<usize>; LISTS]) {
        for list in shown.iter_mut() {
            list.clear();
        }

        let mut open: Vec<u32> = (0..self.roots as u32).collect();

        while let Some(at) = open.pop() {
            let tile = &self.tiles[at as usize];

            let square: f32 = (0..3)
                .map(|axis| {
                    let off =
                        head[axis].clamp(tile.bounds[axis], tile.bounds[axis + 3]) - head[axis];

                    off * off
                })
                .sum();

            if tile.under.is_empty() || tile.cell <= finest * square.sqrt() {
                for (list, boxes) in shown.iter_mut().zip(&tile.boxes) {
                    list.extend(boxes.start as usize..boxes.end as usize);
                }
            } else {
                open.extend(tile.under.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{CellGrid, DetailTree, PlacedBox, TileScene};
    use ty_math::{TyTransformF64, TyVector3F64, TyVector3U32};

    /// A floor 64 cells along x and z, one cell deep, with its coarser
    /// copies, placed a meter a cell from the origin. Returns the scene and
    /// its marched and solid boxes.
    fn floor() -> (TileScene, Vec<PlacedBox>, Vec<PlacedBox>) {
        let mut tiles = TileScene::new(false, true);
        let bounds = TyVector3U32::new(64, 1, 64);

        tiles.opaque.push(true);
        tiles.materials.push(Default::default());

        let grid = CellGrid {
            bounds,
            cells: vec![1; 64 * 64],
        };

        let object = tiles.push_object(&grid, [0; 3]).unwrap();

        tiles.place_object(object, TyTransformF64::IDENTITY);

        let list = |solid: bool| {
            let boxes: Vec<_> = (tiles.objects.iter())
                .map(|object| {
                    if solid {
                        object.masks.solids.as_slice()
                    } else {
                        object.masks.boxes.as_slice()
                    }
                })
                .collect();

            PlacedBox::list(&tiles.placements, &boxes, &tiles.spans)
        };

        let (marched, solid) = (list(false), list(true));

        (tiles, marched, solid)
    }

    #[test]
    fn a_floor_keeps_two_copies_down_to_one_tile() {
        let (tiles, _, _) = floor();

        assert_eq!(tiles.spans, [1, 2, 4]);
        assert_eq!(tiles.objects[2].bounds, TyVector3U32::new(16, 1, 16));
        assert_eq!(tiles.placements.len(), 3);
        assert_eq!(tiles.placements[2].1.scale, TyVector3F64::splat(4.0));
        assert_eq!((tiles.coarser(0), tiles.coarser(2)), (Some(1), None));
    }

    #[test]
    fn a_far_head_draws_the_coarsest_tile_and_a_near_one_the_finest() {
        let (tiles, marched, solid) = floor();
        let tree = DetailTree::new(&tiles, [&marched, &solid, &[]]).unwrap();
        let mut shown = [Vec::new(), Vec::new(), Vec::new()];

        // The boxes of each copy among the solid boxes: 16 tiles, 4, and 1.
        let of = |placement: u32| -> Vec<usize> {
            (0..solid.len())
                .filter(|&index| solid[index].placement == placement)
                .collect()
        };

        assert_eq!((of(0).len(), of(1).len(), of(2).len()), (16, 4, 1));

        // From far above, a cell of the coarsest copy spans under the
        // finest a cell may.
        tree.select([32.0, 1000.0, 32.0], 0.01, &mut shown);
        assert_eq!(shown[1], of(2));

        // With no cell too fine to draw, every tile of the object draws.
        tree.select([32.0, 1000.0, 32.0], 0.0, &mut shown);
        shown[1].sort_unstable();
        assert_eq!(shown[1], of(0));

        // A head over one corner draws that corner fine and the rest
        // coarser, and every place draws once.
        tree.select([0.0, 8.0, 0.0], 0.05, &mut shown);

        let cells: u32 = (shown[1].iter())
            .map(|&index| {
                let extent = solid[index].high - solid[index].low;

                extent.x * extent.z * solid[index].span * solid[index].span
            })
            .sum();

        assert_eq!(cells, 64 * 64);
        assert!(shown[1].iter().any(|&index| solid[index].span == 1));
        assert!(shown[1].iter().any(|&index| solid[index].span > 1));
    }
}
