use crate::BoxCrosser;
use branded_id::U32Id;
use std::collections::HashMap;
use ty_math::{TyTransformF64, TyVector3F32, TyVector3F64, TyVector3U32};
use voxcore::BVoxObject;
use voxrender::{RenderScene, grid_point};

/// The meters two boxes can share along an axis and still count as apart,
/// so boxes that only touch do.
const TOUCH: f64 = 1.0e-4;

/// One placement's box around the live cells of one of its object's
/// bricks.
pub struct PlacedBox {
    /// The placement's index in the scene's placement order.
    pub placement: u32,

    /// The box's index among its object's boxes.
    pub brick: usize,

    /// The box's low cell in the placement's grid.
    pub low: TyVector3U32,

    /// The corner past the box's high cell.
    pub high: TyVector3U32,

    /// The placement's transform.
    transform: TyTransformF64,

    /// The low and high corners of the world box around the box.
    world: (TyVector3F64, TyVector3F64),
}

impl PlacedBox {
    /// Each placement's boxes in `scene`, in placement order. `boxes` holds
    /// each object's boxes as low cells and the corners past their high
    /// cells.
    pub fn list(
        scene: &RenderScene,
        boxes: &HashMap<U32Id<BVoxObject>, Vec<(TyVector3U32, TyVector3U32)>>,
    ) -> Vec<Self> {
        let mut placed = Vec::new();

        for (index, (_, placement)) in (0u32..).zip(scene.iter_placements()) {
            for (brick, &(low, high)) in boxes[&placement.object_id].iter().enumerate() {
                let corners = (0..8).map(|corner| {
                    let pick = |bit: u32, low: u32, high: u32| {
                        f64::from(if corner & bit == 0 { low } else { high })
                    };

                    placement.transform.transform_point(TyVector3F64::new(
                        pick(1, low.x, high.x),
                        pick(2, low.y, high.y),
                        pick(4, low.z, high.z),
                    ))
                });

                let world = corners.fold(
                    (
                        TyVector3F64::splat(f64::INFINITY),
                        TyVector3F64::splat(f64::NEG_INFINITY),
                    ),
                    |(least, most), corner| (least.min(corner), most.max(corner)),
                );

                placed.push(PlacedBox {
                    placement: index,
                    brick,
                    low,
                    high,
                    transform: placement.transform,
                    world,
                });
            }
        }

        placed
    }

    /// The low and high corners of the world box around the box.
    pub fn world(&self) -> (TyVector3F64, TyVector3F64) {
        self.world
    }

    /// The box's three `vec4<u32>`s: its low cell and `low`, the corner past
    /// its high cell and `high`, then `more` and two zeros.
    pub fn words(&self, low: u32, high: u32, more: [u32; 2]) -> [u32; 12] {
        let [x, y, z] = self.low.to_array();
        let [far_x, far_y, far_z] = self.high.to_array();
        let [first, count] = more;

        [x, y, z, low, far_x, far_y, far_z, high, first, count, 0, 0]
    }

    /// Whether some axis parts the box from `other`: an axis along which
    /// their spans share at most [`TOUCH`]. Two boxes share space unless
    /// one of 15 axes parts them: either box's three, and the cross of each
    /// pair of them.
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
    /// space with it. A crosser's entry holds a box's index in `marched`
    /// followed by `solid`.
    pub fn crossers(marched: &[Self], solid: &[Self]) -> Vec<Vec<BoxCrosser>> {
        // Every box with its index. A sweep along x then compares only
        // boxes whose spans meet.
        let mut spans: Vec<(&Self, usize)> = marched.iter().chain(solid).zip(0..).collect();

        spans.sort_by(|(a, _), (b, _)| a.world.0.x.total_cmp(&b.world.0.x));

        let mut crossers = vec![Vec::new(); marched.len()];

        let entry = |at: usize| {
            let index = u32::try_from(at).expect("a scene's boxes number under 2^31");

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
                            entry: entry(other_index),
                            low,
                            high,
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
        let (other_low, other_high) = (other.low.as_dvec3(), other.high.as_dvec3());

        let (least, most) = (0..8)
            .map(|corner| {
                let pick = |bit: u32, low: f64, high: f64| {
                    if corner & bit == 0 { low } else { high }
                };

                let point = TyVector3F64::new(
                    pick(1, other_low.x, other_high.x),
                    pick(2, other_low.y, other_high.y),
                    pick(4, other_low.z, other_high.z),
                );

                grid_point(&self.transform, other.transform.transform_point(point))
            })
            .fold(
                (
                    TyVector3F64::splat(f64::INFINITY),
                    TyVector3F64::splat(f64::NEG_INFINITY),
                ),
                |(least, most), corner| (least.min(corner), most.max(corner)),
            );

        let tile = (self.low / 16 * 16).as_dvec3();

        (
            (least.max(self.low.as_dvec3()) - tile).as_vec3(),
            (most.min(self.high.as_dvec3()) - tile).as_vec3(),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        BoxCrosser, PlacedBox, painted_object, place_object, retain_matte, retain_next_object,
    };
    use std::collections::HashMap;
    use ty_math::{TyQuaternionF64, TyVector3F32, TyVector3F64, TyVector3U32};
    use voxrender::RenderScene;

    #[test]
    fn only_boxes_of_two_placements_that_share_space_cross() {
        let mut scene = RenderScene::default();
        let material = retain_matte(&mut scene, [0.5, 0.5, 0.5]);

        let cube = retain_next_object(
            &mut scene,
            painted_object("cube", TyVector3U32::splat(16), |_| Some(material)),
        );

        // The second cube touches the first. The third cuts into the second.
        for x in [0.0, 16.0, 24.0] {
            place_object(
                &mut scene,
                cube,
                TyVector3F64::new(x, 0.0, 0.0),
                TyQuaternionF64::IDENTITY,
                1.0,
            );
        }

        let boxes = HashMap::from([(cube, vec![(TyVector3U32::ZERO, TyVector3U32::splat(16))])]);
        let placed = PlacedBox::list(&scene, &boxes);

        // The second and third cubes share the cells from x = 24 to 32:
        // the second's 8 to 16, and the third's 0 to 8.
        let shared = |entry, low: f32, high: f32| BoxCrosser {
            entry,
            low: TyVector3F32::new(low, 0.0, 0.0),
            high: TyVector3F32::new(high, 16.0, 16.0),
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
