use crate::{Error, Result, XrSystem, check_xr};
use ash::vk::{Format as VkFormat, ImageCreateFlags, ImageUsageFlags};
use openxr::{
    Session, SwapchainCreateFlags, SwapchainUsageFlags, Vulkan as XrVulkan,
    sys::{Swapchain as SysSwapchain, SwapchainCreateInfo, VulkanSwapchainCreateInfoMETA},
};
use std::ptr;

/// The side of each probe swapchain.
const SIDE: u32 = 64;

/// The formats B0 tries: color, depth, and motion vectors.
const FORMATS: [(&str, VkFormat, SwapchainUsageFlags); 3] = [
    (
        "color",
        VkFormat::R8G8B8A8_SRGB,
        SwapchainUsageFlags::COLOR_ATTACHMENT,
    ),
    (
        "depth",
        VkFormat::D32_SFLOAT,
        SwapchainUsageFlags::DEPTH_STENCIL_ATTACHMENT,
    ),
    (
        "motion",
        VkFormat::R16G16B16A16_SFLOAT,
        SwapchainUsageFlags::COLOR_ATTACHMENT,
    ),
];

/// Logs B0's swapchain and refresh-rate results.
pub fn probe_session(xr: &XrSystem, session: &Session<XrVulkan>) -> Result<()> {
    let formats: Vec<_> = session
        .enumerate_swapchain_formats()
        .map_err(Error::Xr)?
        .into_iter()
        .map(|format| VkFormat::from_raw(i32::try_from(format).expect("VkFormat values fit i32")))
        .collect();

    log::info!("b0 swapchain formats: {formats:?}");

    if xr.instance.exts().fb_display_refresh_rate.is_some() {
        log::info!(
            "b0 refresh rates: {:?}",
            session
                .enumerate_display_refresh_rates()
                .map_err(Error::Xr)?
        );
    } else {
        log::info!("b0 refresh rates: absent");
    }

    let has_meta = xr
        .instance
        .exts()
        .meta_vulkan_swapchain_create_info
        .is_some();

    for (label, format, usage) in FORMATS {
        log::info!(
            "b0 swapchain {label}: {}",
            create_result(xr, session, format, usage, None)?
        );

        log::info!(
            "b0 swapchain {label} storage mutable: {}",
            create_result(
                xr,
                session,
                format,
                usage | SwapchainUsageFlags::UNORDERED_ACCESS | SwapchainUsageFlags::MUTABLE_FORMAT,
                None,
            )?
        );

        let meta = if has_meta {
            create_result(
                xr,
                session,
                format,
                usage,
                Some(VulkanSwapchainCreateInfoMETA {
                    ty: VulkanSwapchainCreateInfoMETA::TYPE,
                    next: ptr::null(),
                    additional_create_flags: u64::from(
                        (ImageCreateFlags::MUTABLE_FORMAT | ImageCreateFlags::EXTENDED_USAGE)
                            .as_raw(),
                    ),
                    additional_usage_flags: u64::from(ImageUsageFlags::STORAGE.as_raw()),
                }),
            )?
        } else {
            "absent".to_string()
        };

        log::info!("b0 swapchain {label} meta storage mutable: {meta}");
    }

    Ok(())
}

/// "ok" when the runtime creates a stereo swapchain of `format` and
/// `usage` with `meta` chained, else the runtime's error.
fn create_result(
    xr: &XrSystem,
    session: &Session<XrVulkan>,
    format: VkFormat,
    usage: SwapchainUsageFlags,
    meta: Option<VulkanSwapchainCreateInfoMETA>,
) -> Result<String> {
    let next = meta
        .as_ref()
        .map_or(ptr::null(), |meta| (&raw const *meta).cast());

    let info = SwapchainCreateInfo {
        ty: SwapchainCreateInfo::TYPE,
        next,
        create_flags: SwapchainCreateFlags::EMPTY,
        usage_flags: usage,
        format: i64::from(format.as_raw()),
        sample_count: 1,
        width: SIDE,
        height: SIDE,
        face_count: 1,
        array_size: 2,
        mip_count: 1,
    };

    let mut handle = SysSwapchain::default();

    // SAFETY: `info` and its chain are alive for the call.
    let result =
        unsafe { (xr.instance.fp().create_swapchain)(session.as_raw(), &info, &mut handle) };

    if result.into_raw() < 0 {
        return Ok(format!("{result}"));
    }

    // SAFETY: the runtime just created `handle`, and nothing else holds it.
    check_xr(unsafe { (xr.instance.fp().destroy_swapchain)(handle) })?;

    Ok("ok".to_string())
}
