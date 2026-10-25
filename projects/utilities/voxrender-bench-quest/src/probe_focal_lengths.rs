use crate::{Error, Result, XrSystem};
use openxr::{View, ViewConfigurationType};

/// Logs each eye's field of view and its focal lengths in pixels at the
/// recommended image size.
///
/// # Arguments
///
/// 1. `eyes`: the stereo views of a tracked frame
pub fn probe_focal_lengths(xr: &XrSystem, eyes: &[View]) -> Result<()> {
    let views = xr
        .instance
        .enumerate_view_configuration_views(xr.system, ViewConfigurationType::PRIMARY_STEREO)
        .map_err(Error::Xr)?;

    for (eye, (view, size)) in eyes.iter().zip(&views).enumerate() {
        let fov = view.fov;
        let [left, right, up, down] = [
            fov.angle_left,
            fov.angle_right,
            fov.angle_up,
            fov.angle_down,
        ]
        .map(f64::from);

        // The image spans the tangents of the field of view.
        let x = f64::from(size.recommended_image_rect_width) / (right.tan() - left.tan());
        let y = f64::from(size.recommended_image_rect_height) / (up.tan() - down.tan());

        log::info!(
            "b0 focal lengths, view {eye}: left {:.3}, right {:.3}, up {:.3}, down {:.3} \
             degrees; f {x:.1} by {y:.1} px",
            left.to_degrees(),
            right.to_degrees(),
            up.to_degrees(),
            down.to_degrees()
        );
    }

    Ok(())
}
