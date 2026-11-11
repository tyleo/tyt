use ty_math::{TyMatrix4x4F64, TyVector3F64};
use voxrender::{RenderProjection, RenderScene, RenderView, RenderViewRays};

/// The perspective's near plane in world units. Reversed depth keeps f32's
/// relative precision at every distance. The plane can then sit far nearer
/// than any cell.
const NEAR: f64 = 1.0 / 1024.0 / 1024.0;

/// The matrix that carries a world point to clip space so that each pixel
/// center samples the ray `RenderViewRays` casts through it.
///
/// A perspective ray reaches `origin + X dx + Y dy + T d` from the steps of
/// [`RenderViewRays::direction_steps`], where `T` is the depth along the
/// view and `(X, Y)` is the pixel times `T`. An orthographic ray reaches
/// `origin + x ox + y oy + t d` from the steps of
/// [`RenderViewRays::origin_steps`], where `t` is its distance. Inverting the
/// steps gives the pixel. The depth runs reversed.
pub fn world_to_clip(
    scene: &RenderScene,
    view: &RenderView,
    width: u32,
    height: u32,
) -> TyMatrix4x4F64 {
    let rays = RenderViewRays::new(view, width, height);
    let [origin, origin_x, origin_y] = rays.origin_steps();
    let [direction, direction_x, direction_y] = rays.direction_steps();

    let w = f64::from(width);
    let h = f64::from(height);

    let steps = |x: TyVector3F64, y: TyVector3F64, z: TyVector3F64| {
        TyMatrix4x4F64::from_cols(
            x.extend(0.0),
            y.extend(0.0),
            z.extend(0.0),
            origin.extend(1.0),
        )
        .inverse()
    };

    // Each row gives one clip component.
    let (to_pixels, rows) = match view.projection {
        RenderProjection::Perspective { .. } => (
            steps(direction_x, direction_y, direction),
            [
                [2.0 / w, 0.0, 1.0 / w - 1.0, 0.0],
                [0.0, -2.0 / h, 1.0 - 1.0 / h, 0.0],
                [0.0, 0.0, 0.0, NEAR],
                [0.0, 0.0, 1.0, 0.0],
            ],
        ),

        RenderProjection::Orthographic { .. } => {
            let to_pixels = steps(origin_x, origin_y, direction);

            // The far plane sits past every placement's deepest corner.
            let far = scene
                .iter_placements()
                .flat_map(|(_, placement)| {
                    let bounds = scene
                        .object(placement.object_id)
                        .expect("a placement's object is one of the scene's")
                        .bounds()
                        .as_dvec3();

                    (0..8).map(move |corner| {
                        let point = TyVector3F64::new(
                            if corner & 1 == 0 { 0.0 } else { bounds.x },
                            if corner & 2 == 0 { 0.0 } else { bounds.y },
                            if corner & 4 == 0 { 0.0 } else { bounds.z },
                        );

                        to_pixels
                            .transform_point3(placement.transform.transform_point(point))
                            .z
                    })
                })
                .fold(0.0, f64::max)
                + 1.0;

            (
                to_pixels,
                [
                    [2.0 / w, 0.0, 0.0, 1.0 / w - 1.0],
                    [0.0, -2.0 / h, 0.0, 1.0 - 1.0 / h],
                    [0.0, 0.0, -1.0 / far, 1.0],
                    [0.0, 0.0, 0.0, 1.0],
                ],
            )
        }
    };

    TyMatrix4x4F64::from_cols_array_2d(&rows).transpose() * to_pixels
}
