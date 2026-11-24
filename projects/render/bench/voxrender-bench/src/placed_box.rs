use crate::{
    BObjectTile, BTilePlacement, BoxCrosser, BoxWords, ObjectTiles, TileBox, TilePlacement,
    TileScene, bounds_around, box_corners, to_word,
};
use branded_id::{IteratorExt, U32Id, UsizeId};
use ty_math::{TyTransformF64, TyVector3F32, TyVector3F64, TyVector3U32};
use voxrender::grid_point;

/// The meters two boxes can share along an axis and still count as apart,
/// so boxes that only touch do.
const TOUCH: f64 = 1.0e-4;

/// One placement's box of cells within one of its object's tiles.
pub struct PlacedBox {
    pub placement: U32Id<BTilePlacement>,

    /// The tile among its object's tiles of the box's kind.
    pub tile: UsizeId<BObjectTile>,

    /// The box's low cell in the placement's grid.
    pub low: TyVector3U32,

    /// The corner past the box's high cell.
    pub high: TyVector3U32,

    /// How many of its object's cells one of the box's cells spans, over
    /// one for a box of an object's coarser copy.
    pub span: u32,

    /// The placement's transform.
    transform: TyTransformF64,

    world: (TyVector3F64, TyVector3F64),
}

impl PlacedBox {
    /// The boxes `boxes` gives for the object of each of `tiles`'
    /// placements, in the placements' order.
    pub fn list(tiles: &TileScene, boxes: impl Fn(&ObjectTiles) -> &[TileBox]) -> Vec<Self> {
        let mut placed = Vec::new();

        for (placement, placed_object) in tiles.placements.iter().enumerate_ids() {
            let TilePlacement { object, transform } = placed_object;
            let object = &tiles.objects[*object];

            for &TileBox { cells, tile } in boxes(object) {
                let (low, high) = cells;

                let corners = box_corners(low.as_dvec3(), high.as_dvec3())
                    .map(|corner| transform.transform_point(corner));

                placed.push(PlacedBox {
                    placement,
                    tile,
                    low,
                    high,
                    span: object.span,
                    transform: *transform,
                    world: bounds_around(corners),
                });
            }
        }

        placed
    }

    /// The low and high corners of the world box around the box.
    pub fn world(&self) -> (TyVector3F64, TyVector3F64) {
        self.world
    }

    /// The box's words for a tile whose words start at `tile` in its
    /// page's, with zeros for its material and its lit words.
    pub fn words(&self, tile: u32, more: [u32; 2]) -> BoxWords {
        BoxWords {
            low: self.low.to_array(),
            placement: self.placement.to_u32(),
            high: self.high.to_array(),
            tile,
            more,
            material: 0,
            lit: 0,
        }
    }

    /// Whether some axis parts the box from `other`. Their spans along a
    /// parting axis share at most [`TOUCH`]. The test tries 15 axes: each
    /// box's three and the cross of each pair.
    fn parted_from(&self, other: &Self) -> bool {
        let (center, axes, halves) = self.oriented();
        let (other_center, other_axes, other_halves) = other.oriented();
        let between = other_center - center;

        let parts = |axis: TyVector3F64| {
            let reach = |axes: &[TyVector3F64; 3], halves: TyVector3F64| -> f64 {
                (0..3).map(|at| axes[at].dot(axis).abs() * halves[at]).sum()
            };

            between.dot(axis).abs() + TOUCH
                >= reach(&axes, halves) + reach(&other_axes, other_halves)
        };

        let crosses = axes.iter().flat_map(|&axis| {
            other_axes
                .iter()
                .map(move |&other_axis| axis.cross(other_axis))
        });

        // The cross of two axes that run alike has no direction.
        axes.iter()
            .chain(&other_axes)
            .copied()
            .chain(crosses.filter_map(|axis| axis.try_normalize()))
            .any(parts)
    }

    /// The box in the world: its center, its axes, and its half extent
    /// along each.
    fn oriented(&self) -> (TyVector3F64, [TyVector3F64; 3], TyVector3F64) {
        let low = self.low.as_dvec3();
        let high = self.high.as_dvec3();

        (
            self.transform.transform_point((low + high) / 2.0),
            [TyVector3F64::X, TyVector3F64::Y, TyVector3F64::Z]
                .map(|axis| self.transform.rotation * axis),
            (high - low) / 2.0 * self.transform.scale,
        )
    }

    /// For each of `marched`, the boxes of other placements that share
    /// space with it and span as many cells of their objects. Boxes of
    /// unlike spans never draw over one another's cells because a view draws
    /// an object or one of its coarser copies in a place, never both.
    pub fn crossers(marched: &[Self], solid: &[Self]) -> Vec<Vec<BoxCrosser>> {
        // Every box with its index. A sweep along x then compares only
        // boxes whose spans meet.
        let mut spans: Vec<(&Self, usize)> = marched.iter().chain(solid).zip(0..).collect();

        spans.sort_by(|(a, _), (b, _)| a.world.0.x.total_cmp(&b.world.0.x));

        let mut crossers = vec![Vec::new(); marched.len()];

        let entry = |at: usize| {
            let index = to_word(at);

            assert!(
                index < BoxCrosser::SOLID,
                "a scene's boxes number under 2^31"
            );

            if at < marched.len() {
                index
            } else {
                index | BoxCrosser::SOLID
            }
        };

        for (first, &(a, a_index)) in spans.iter().enumerate() {
            for &(b, b_index) in &spans[first + 1..] {
                if b.world.0.x + TOUCH >= a.world.1.x {
                    break;
                }

                let apart = a.placement == b.placement
                    || a.span != b.span
                    || (a.world.0 + TOUCH).cmpge(b.world.1).any()
                    || (b.world.0 + TOUCH).cmpge(a.world.1).any()
                    || a.parted_from(b);

                if apart {
                    continue;
                }

                for (own, own_index, other, other_index) in
                    [(a, a_index, b, b_index), (b, b_index, a, a_index)]
                {
                    if let Some(of_own) = crossers.get_mut(own_index) {
                        let (low, high) = own.shared_with(other);

                        of_own.push(BoxCrosser {
                            low: low.to_array(),
                            entry: entry(other_index),
                            high: high.to_array(),
                            pad: 0,
                        });
                    }
                }
            }
        }

        crossers
    }

    /// The low and high corners of a box around the space the box shares
    /// with `other`, in the cells of the box's tile. The box holds all of
    /// that space and can hold more.
    fn shared_with(&self, other: &Self) -> (TyVector3F32, TyVector3F32) {
        let corners = box_corners(other.low.as_dvec3(), other.high.as_dvec3());

        let (least, most) =
            bounds_around(corners.map(|corner| {
                grid_point(&self.transform, other.transform.transform_point(corner))
            }));

        let tile = (self.low / 16 * 16).as_dvec3();

        (
            (least.max(self.low.as_dvec3()) - tile).as_vec3(),
            (most.min(self.high.as_dvec3()) - tile).as_vec3(),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{BoxCrosser, CellGrid, PlacedBox, TileScene};
    use ty_math::{TyTransformF64, TyVector3F64, TyVector3U32};

    #[test]
    fn only_boxes_of_two_placements_that_share_space_cross() {
        // Three placements of a cube. The second touches the first. The
        // third cuts into the second.
        let mut tiles = TileScene::new(false, false);

        tiles.opaque.push(true);
        tiles.materials.push(Default::default());

        let grid = CellGrid {
            bounds: TyVector3U32::splat(16),
            cells: vec![1; 16 * 16 * 16],
        };

        let cube = tiles.push_object(&grid, [0; 3]).unwrap();

        for x in [0.0, 16.0, 24.0] {
            let position = TyVector3F64::new(x, 0.0, 0.0);

            tiles.place_object(cube, TyTransformF64::from_translation(position));
        }

        // A tile of live cells alone gives one solid box.
        let placed = PlacedBox::list(&tiles, |object| &object.masks.solids);

        assert_eq!(placed.len(), 3);

        // The second and third cubes share the cells from x = 24 to 32:
        // the second's 8 to 16, and the third's 0 to 8.
        let shared = |entry, low: f32, high: f32| BoxCrosser {
            low: [low, 0.0, 0.0],
            entry,
            high: [high, 16.0, 16.0],
            pad: 0,
        };

        assert_eq!(
            PlacedBox::crossers(&placed, &[]),
            [
                vec![],
                vec![shared(2, 8.0, 16.0)],
                vec![shared(1, 0.0, 8.0)]
            ]
        );
        assert_eq!(PlacedBox::crossers(&placed[..1], &placed[1..]), [vec![]]);
        assert_eq!(
            PlacedBox::crossers(&placed[1..2], &placed[2..]),
            [vec![shared(1 | BoxCrosser::SOLID, 8.0, 16.0)]]
        );
    }
}
