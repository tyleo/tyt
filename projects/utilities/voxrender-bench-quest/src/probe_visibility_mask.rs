use crate::{Error, Result, XrSystem};
use openxr::{
    Session, Vector2f, View, ViewConfigurationType, VisibilityMaskTypeKHR, Vulkan as XrVulkan,
};

/// Logs the share of each eye's image that XR_KHR_visibility_mask hides,
/// with the vertex counts of the other mask types.
///
/// # Arguments
///
/// 1. `eyes`: the stereo views of a tracked frame, whose fields of view
///    bound the images
pub fn probe_visibility_mask(
    xr: &XrSystem,
    session: &Session<XrVulkan>,
    eyes: &[View],
) -> Result<()> {
    if xr.instance.exts().khr_visibility_mask.is_none() {
        log::info!("b0 visibility mask: absent");

        return Ok(());
    }

    for (eye, view) in (0..).zip(eyes) {
        for mask_type in [
            VisibilityMaskTypeKHR::VISIBLE_TRIANGLE_MESH,
            VisibilityMaskTypeKHR::LINE_LOOP,
        ] {
            let mask = session
                .get_visibility_mask_khr(ViewConfigurationType::PRIMARY_STEREO, eye, mask_type)
                .map_err(Error::Xr)?;

            log::info!(
                "b0 visibility mask {mask_type:?}, view {eye}: {} vertices",
                mask.vertices.len()
            );
        }

        let mask = session
            .get_visibility_mask_khr(
                ViewConfigurationType::PRIMARY_STEREO,
                eye,
                VisibilityMaskTypeKHR::HIDDEN_TRIANGLE_MESH,
            )
            .map_err(Error::Xr)?;

        if mask.indices.is_empty() {
            log::info!("b0 visibility mask, view {eye}: empty");

            continue;
        }

        if mask.indices.len() % 3 != 0 {
            return Err(Error::VisibilityMask);
        }

        // The mask's vertices lie on the view's z = -1 plane, where the
        // image spans the tangents of the field of view.
        let mut hidden = 0.0;

        for triangle in mask.indices.chunks_exact(3) {
            let corner = |index: u32| {
                usize::try_from(index)
                    .ok()
                    .and_then(|index| mask.vertices.get(index))
                    .ok_or(Error::VisibilityMask)
            };

            let [a, b, c] = [
                corner(triangle[0])?,
                corner(triangle[1])?,
                corner(triangle[2])?,
            ];

            hidden += triangle_area(a, b, c);
        }

        let fov = view.fov;
        let width = f64::from(fov.angle_right).tan() - f64::from(fov.angle_left).tan();
        let height = f64::from(fov.angle_up).tan() - f64::from(fov.angle_down).tan();

        log::info!(
            "b0 visibility mask, view {eye}: {} triangles hide {:.4} of the image",
            mask.indices.len() / 3,
            hidden / (width * height)
        );
    }

    Ok(())
}

/// The area of the triangle `a`, `b`, `c`.
fn triangle_area(a: &Vector2f, b: &Vector2f, c: &Vector2f) -> f64 {
    let [ax, ay, bx, by, cx, cy] = [a.x, a.y, b.x, b.y, c.x, c.y].map(f64::from);

    ((bx - ax) * (cy - ay) - (cx - ax) * (by - ay)).abs() / 2.0
}
