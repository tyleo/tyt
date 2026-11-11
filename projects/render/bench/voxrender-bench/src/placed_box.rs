use branded_id::U32Id;
use std::collections::HashMap;
use ty_math::{TyTransformF64, TyVector3F64, TyVector3U32};
use voxcore::BVoxObject;
use voxrender::{RenderProjection, RenderScene, RenderView, grid_point};

/// The cells past a box's cells within which a ray's start counts as inside
/// the box. The margin covers a raster's pad and the near plane.
const MARGIN: f64 = 1.0 / 32.0;

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

    /// Whether a ray of `view` can start inside the box, and how far the
    /// box's nearest point lies from the view: from a perspective view's
    /// eye, or along an orthographic view's axis.
    pub fn seen(&self, view: &RenderView) -> (bool, f64) {
        let low = self.low.as_dvec3() - MARGIN;
        let high = self.high.as_dvec3() + MARGIN;
        let position = view.pose.position;

        match view.projection {
            RenderProjection::Perspective { .. } => {
                let eye = grid_point(&self.transform, position);
                let nearest = eye.clamp(low, high);

                (
                    nearest == eye,
                    (self.transform.transform_point(nearest) - position).length(),
                )
            }

            RenderProjection::Orthographic { .. } => {
                let forward = view.pose.rotation * -TyVector3F64::Z;

                let depth = (0..8)
                    .map(|corner| {
                        let point = TyVector3F64::new(
                            if corner & 1 == 0 { low.x } else { high.x },
                            if corner & 2 == 0 { low.y } else { high.y },
                            if corner & 4 == 0 { low.z } else { high.z },
                        );

                        forward.dot(self.transform.transform_point(point) - position)
                    })
                    .fold(f64::INFINITY, f64::min);

                (depth <= 0.0, depth)
            }
        }
    }

    /// The low and high corners of the world box around the box.
    pub fn world(&self) -> (TyVector3F64, TyVector3F64) {
        self.world
    }

    /// The box's two `vec4<u32>`s: its low cell and `low`, then the corner
    /// past its high cell and `high`.
    pub fn words(&self, low: u32, high: u32) -> [u32; 8] {
        let [x, y, z] = self.low.to_array();
        let [far_x, far_y, far_z] = self.high.to_array();

        [x, y, z, low, far_x, far_y, far_z, high]
    }

    /// Whether each of `boxes` shares space with a box of another placement
    /// among `boxes` or `others`. The test compares the world boxes around
    /// the boxes, so it can also count a turned box that only comes near.
    pub fn crossings(boxes: &[Self], others: &[Self]) -> Vec<bool> {
        // Every box's low and high x, placement, and index in `boxes`. A
        // sweep along x then compares only boxes whose spans meet.
        let mut spans: Vec<(&Self, Option<usize>)> = boxes
            .iter()
            .enumerate()
            .map(|(index, placed)| (placed, Some(index)))
            .chain(others.iter().map(|placed| (placed, None)))
            .collect();

        spans.sort_by(|(a, _), (b, _)| a.world.0.x.total_cmp(&b.world.0.x));

        let mut crosses = vec![false; boxes.len()];

        for (first, &(a, a_index)) in spans.iter().enumerate() {
            for &(b, b_index) in &spans[first + 1..] {
                if b.world.0.x + TOUCH >= a.world.1.x {
                    break;
                }

                let apart = a.placement == b.placement
                    || (a.world.0 + TOUCH).cmpge(b.world.1).any()
                    || (b.world.0 + TOUCH).cmpge(a.world.1).any();

                if !apart {
                    for index in [a_index, b_index].into_iter().flatten() {
                        crosses[index] = true;
                    }
                }
            }
        }

        crosses
    }
}

#[cfg(test)]
mod tests {
    use crate::{PlacedBox, painted_object, place_object, retain_matte, retain_next_object};
    use std::collections::HashMap;
    use ty_math::{TyQuaternionF64, TyVector3F64, TyVector3U32};
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

        assert_eq!(PlacedBox::crossings(&placed, &[]), [false, true, true]);
        assert_eq!(PlacedBox::crossings(&placed[..1], &placed[1..]), [false]);
        assert_eq!(PlacedBox::crossings(&placed[1..2], &placed[2..]), [true]);
    }
}
