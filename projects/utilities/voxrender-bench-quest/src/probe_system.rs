use crate::{Error, Result, XrSystem, check_xr};
use openxr::{
    ViewConfigurationType,
    sys::{
        FrameSynthesisConfigViewEXT, SystemProperties, SystemSpaceWarpPropertiesFB,
        ViewConfigurationView,
    },
};
use std::{ffi::CStr, mem, ptr};

/// Logs B0's runtime and system results.
pub fn probe_system(xr: &XrSystem) -> Result<()> {
    let runtime = xr.instance.properties().map_err(Error::Xr)?;

    log::info!(
        "b0 runtime: {} {}",
        runtime.runtime_name,
        runtime.runtime_version
    );

    let system = xr
        .instance
        .system_properties(xr.system)
        .map_err(Error::Xr)?;

    log::info!(
        "b0 system: {}, vendor {:#x}, {:?}",
        system.system_name,
        system.vendor_id,
        system.graphics_properties
    );

    let mut names: Vec<_> = xr
        .available
        .names()
        .into_iter()
        .map(|name| {
            CStr::from_bytes_until_nul(name)
                .expect("openxr's extension names end in nul")
                .to_string_lossy()
        })
        .collect();

    names.sort();

    log::info!("b0 extensions: {}", names.join(" "));

    for view_type in xr
        .instance
        .enumerate_view_configurations(xr.system)
        .map_err(Error::Xr)?
    {
        let properties = xr
            .instance
            .view_configuration_properties(xr.system, view_type)
            .map_err(Error::Xr)?;

        let views = xr
            .instance
            .enumerate_view_configuration_views(xr.system, view_type)
            .map_err(Error::Xr)?;

        log::info!(
            "b0 view configuration {view_type:?}: fov mutable {}, {views:?}",
            properties.fov_mutable
        );
    }

    if xr.instance.exts().fb_space_warp.is_some() {
        let (width, height) = space_warp_size(xr)?;

        log::info!("b0 space warp motion vectors: {width}x{height}");
    } else {
        log::info!("b0 space warp: absent");
    }

    if xr.instance.exts().ext_frame_synthesis.is_some() {
        for (eye, (width, height)) in frame_synthesis_sizes(xr)?.into_iter().enumerate() {
            log::info!("b0 frame synthesis motion vectors, view {eye}: {width}x{height}");
        }
    } else {
        log::info!("b0 frame synthesis: absent");
    }

    Ok(())
}

/// XR_FB_space_warp's recommended motion-vector image size.
fn space_warp_size(xr: &XrSystem) -> Result<(u32, u32)> {
    let mut space_warp = SystemSpaceWarpPropertiesFB {
        ty: SystemSpaceWarpPropertiesFB::TYPE,
        next: ptr::null_mut(),
        recommended_motion_vector_image_rect_width: 0,
        recommended_motion_vector_image_rect_height: 0,
    };

    // SAFETY: every field of `SystemProperties` is valid as zeros.
    let mut properties = SystemProperties {
        ty: SystemProperties::TYPE,
        next: (&raw mut space_warp).cast(),
        ..unsafe { mem::zeroed() }
    };

    // SAFETY: `properties` and its chain are alive for the call.
    check_xr(unsafe {
        (xr.instance.fp().get_system_properties)(xr.instance.as_raw(), xr.system, &mut properties)
    })?;

    Ok((
        space_warp.recommended_motion_vector_image_rect_width,
        space_warp.recommended_motion_vector_image_rect_height,
    ))
}

/// XR_EXT_frame_synthesis's recommended motion-vector image size for each
/// stereo view.
fn frame_synthesis_sizes(xr: &XrSystem) -> Result<Vec<(u32, u32)>> {
    let enumerate = xr.instance.fp().enumerate_view_configuration_views;
    let mut count = 0;

    // SAFETY: a zero capacity writes only the count.
    check_xr(unsafe {
        enumerate(
            xr.instance.as_raw(),
            xr.system,
            ViewConfigurationType::PRIMARY_STEREO,
            0,
            &mut count,
            ptr::null_mut(),
        )
    })?;

    let length = usize::try_from(count).expect("view counts fit in usize");

    let mut synthesis: Vec<_> = (0..length)
        .map(|_| FrameSynthesisConfigViewEXT {
            ty: FrameSynthesisConfigViewEXT::TYPE,
            next: ptr::null_mut(),
            recommended_motion_vector_image_rect_width: 0,
            recommended_motion_vector_image_rect_height: 0,
        })
        .collect();

    let mut views: Vec<_> = synthesis
        .iter_mut()
        .map(|synthesis| ViewConfigurationView {
            ty: ViewConfigurationView::TYPE,
            next: (&raw mut *synthesis).cast(),
            recommended_image_rect_width: 0,
            max_image_rect_width: 0,
            recommended_image_rect_height: 0,
            max_image_rect_height: 0,
            recommended_swapchain_sample_count: 0,
            max_swapchain_sample_count: 0,
        })
        .collect();

    // SAFETY: `views` holds `count` entries whose chains are alive for the
    // call.
    check_xr(unsafe {
        enumerate(
            xr.instance.as_raw(),
            xr.system,
            ViewConfigurationType::PRIMARY_STEREO,
            count,
            &mut count,
            views.as_mut_ptr(),
        )
    })?;

    drop(views);

    let length = usize::try_from(count).expect("view counts fit in usize");

    Ok(synthesis[..length]
        .iter()
        .map(|synthesis| {
            (
                synthesis.recommended_motion_vector_image_rect_width,
                synthesis.recommended_motion_vector_image_rect_height,
            )
        })
        .collect())
}
