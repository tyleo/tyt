use crate::BoxFaces;
use branded_id::U32Id;
use std::collections::HashMap;
use ty_math::{TyVector3F64, TyVector3U32};
use voxcore::BVoxObject;
use voxrender::{RenderPlacement, RenderProjection, RenderScene, RenderView, grid_point};

/// The cells past a box's cells within which a ray's start counts as inside
/// the box. The margin covers a raster's pad and the near plane.
const MARGIN: f64 = 1.0 / 32.0;

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

    /// Whether a ray of the view can start inside the box.
    inside: bool,

    /// How far the box's nearest point lies from the view.
    distance: f64,
}

impl PlacedBox {
    /// Each placement's boxes in `scene` as `view` sees them, and how many
    /// of them come before the first box that draws its back faces.
    /// `boxes` holds each object's boxes as low cells and the corners past
    /// their high cells.
    ///
    /// [`BoxFaces::Back`] lists the boxes in placement order, and every box
    /// draws its back faces. [`BoxFaces::Front`] lists the nearest box
    /// first, then the boxes a ray can start inside, which draw their back
    /// faces.
    pub fn list(
        scene: &RenderScene,
        view: &RenderView,
        boxes: &HashMap<U32Id<BVoxObject>, Vec<(TyVector3U32, TyVector3U32)>>,
        faces: BoxFaces,
    ) -> (Vec<Self>, u32) {
        let mut placed = Vec::new();

        for (index, (_, placement)) in (0u32..).zip(scene.iter_placements()) {
            for (brick, &(low, high)) in boxes[&placement.object_id].iter().enumerate() {
                let (inside, distance) = nearest(view, placement, low, high);

                placed.push(PlacedBox {
                    placement: index,
                    brick,
                    low,
                    high,
                    inside,
                    distance,
                });
            }
        }

        let fronts = match faces {
            BoxFaces::Back => 0,

            BoxFaces::Front => {
                placed.sort_by(|a, b| {
                    a.inside
                        .cmp(&b.inside)
                        .then(a.distance.total_cmp(&b.distance))
                });

                placed.iter().filter(|placed| !placed.inside).count()
            }
        };

        (
            placed,
            u32::try_from(fronts).expect("a scene's boxes number under 2^32"),
        )
    }
}

/// Whether a ray of `view` can start inside the box from `low` to `high` in
/// `placement`'s grid, and how far the box's nearest point lies from the
/// view: from a perspective view's eye, or along an orthographic view's
/// axis.
fn nearest(
    view: &RenderView,
    placement: &RenderPlacement,
    low: TyVector3U32,
    high: TyVector3U32,
) -> (bool, f64) {
    let transform = &placement.transform;
    let low = low.as_dvec3() - MARGIN;
    let high = high.as_dvec3() + MARGIN;
    let position = view.pose.position;

    match view.projection {
        RenderProjection::Perspective { .. } => {
            let eye = grid_point(transform, position);
            let nearest = eye.clamp(low, high);

            (
                nearest == eye,
                (transform.transform_point(nearest) - position).length(),
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

                    forward.dot(transform.transform_point(point) - position)
                })
                .fold(f64::INFINITY, f64::min);

            (depth <= 0.0, depth)
        }
    }
}
