use crate::{Error, Result};
use voxrender::{RenderProjection, RenderScene, RenderView};

/// Sets the left and right edges of `scene`'s first view, about their
/// center, to the shape of a `width` by `height` image of square pixels.
/// The view keeps its pose and its top and bottom edges. An orthographic
/// view stays as it is. Errors unless the scene has a view and the image
/// has two positive sides.
pub fn shape_view(scene: &mut RenderScene, width: u32, height: u32) -> Result<()> {
    let (view_id, view) = scene.iter_views().next().ok_or(Error::NoView)?;
    let pose = view.pose;

    let RenderProjection::Perspective {
        left,
        right,
        up,
        down,
    } = view.projection
    else {
        return Ok(());
    };

    let center = (right + left) / 2.0;
    let across = (up - down) / 2.0 * (f64::from(width) / f64::from(height));

    scene
        .set_view(
            view_id,
            RenderView {
                pose,
                projection: RenderProjection::Perspective {
                    left: center - across,
                    right: center + across,
                    up,
                    down,
                },
            },
        )
        .map_err(Error::Render)
}

#[cfg(test)]
mod tests {
    use crate::{eye_view, shape_view};
    use ty_math::TyVector3F64;
    use voxrender::{RenderProjection, RenderScene};

    #[test]
    fn a_view_takes_an_images_shape_about_its_center() {
        let mut scene = RenderScene::default();
        let view = eye_view(TyVector3F64::ZERO, TyVector3F64::X);
        let view_id = scene.retain_view(view).unwrap();

        shape_view(&mut scene, 4, 2).unwrap();

        // A 90-degree view's top edge has a slope of one.
        let shaped = scene.view(view_id).unwrap();
        let wide = RenderProjection::from_fov(90f64.to_radians(), 4, 2).unwrap();
        assert_eq!(shaped.projection, wide);
        assert_eq!(shaped.pose, view.pose);

        assert!(shape_view(&mut scene, 4, 0).is_err());
    }
}
