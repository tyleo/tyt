use crate::{Error, Result};
use branded_id::U32Id;
use ty_math::{TyPoseF64, TyVector3F64};
use voxrender::{BRenderView, RenderProjection, RenderScene, RenderView};

/// The meters between the eyes, near the middle of a Quest 3's range.
const EYES_APART: f64 = 0.064;

/// The degrees from a Quest 3's left eye's axis to its left, right, up, and
/// down edges, as its runtime gives them. The right eye mirrors them.
const EDGES: [f64; 4] = [-54.0, 40.0, 44.0, -55.0];

/// Adds a Quest 3's two eyes to `scene`, the left eye first, and returns
/// their views' ids. The eyes sit to either side of the first view's
/// position and take its rotation. Errors unless the scene has a view.
pub fn retain_headset_views(scene: &mut RenderScene) -> Result<[U32Id<BRenderView>; 2]> {
    let (_, view) = scene.iter_views().next().ok_or(Error::NoView)?;
    let pose = view.pose;
    let [outer, inner, up, down] = EDGES.map(|degrees| degrees.to_radians().tan());
    let aside = pose.rotation * TyVector3F64::X * (EYES_APART / 2.0);

    let mut eye = |offset: TyVector3F64, left: f64, right: f64| {
        scene
            .retain_view(RenderView {
                pose: TyPoseF64::new(pose.position + offset, pose.rotation),
                projection: RenderProjection::Perspective {
                    left,
                    right,
                    up,
                    down,
                },
            })
            .map_err(Error::Render)
    };

    Ok([eye(-aside, outer, inner)?, eye(aside, -inner, -outer)?])
}

#[cfg(test)]
mod tests {
    use crate::{eye_view, retain_headset_views};
    use ty_math::TyVector3F64;
    use voxrender::{RenderProjection, RenderScene};

    #[test]
    fn the_eyes_sit_apart_and_mirror_their_edges() {
        let mut scene = RenderScene::default();

        // The view looks down -Z, so its +X is the world's.
        scene
            .retain_view(eye_view(TyVector3F64::ZERO, -TyVector3F64::Z))
            .unwrap();

        let [left_id, right_id] = retain_headset_views(&mut scene).unwrap();
        let left = scene.view(left_id).unwrap();
        let right = scene.view(right_id).unwrap();

        assert!((right.pose.position.x - left.pose.position.x - 0.064).abs() < 1e-12);
        assert_eq!(left.pose.rotation, right.pose.rotation);

        let edges = |projection| match projection {
            RenderProjection::Perspective {
                left,
                right,
                up,
                down,
            } => [left, right, up, down],

            RenderProjection::Orthographic { .. } => panic!("an eye's view is in perspective"),
        };

        let [l, r, u, d] = edges(left.projection);
        assert_eq!(edges(right.projection), [-r, -l, u, d]);

        // Each eye sees farther to its outer side and farther down.
        assert!(-l > r && -d > u);
    }
}
