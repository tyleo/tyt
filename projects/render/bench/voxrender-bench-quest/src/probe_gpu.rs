use crate::{Error, Result, XrGpu};
use ash::{
    khr::{ray_query, shader_atomic_int64, shader_float16_int8},
    vk::{
        self, PhysicalDeviceFeatures2, PhysicalDeviceRayQueryFeaturesKHR,
        PhysicalDeviceShaderAtomicInt64Features, PhysicalDeviceShaderFloat16Int8Features,
    },
};
use std::ffi::CStr;
use wgpu::hal::api::Vulkan;

/// Logs B0's GPU results, including the Vulkan feature bits wgpu does not
/// report.
pub fn probe_gpu(gpu: &XrGpu) -> Result<()> {
    log::info!("b0 adapter: {:?}", gpu.adapter.get_info());
    log::info!("b0 wgpu features: {:?}", gpu.adapter.features());
    log::info!("b0 wgpu limits: {:#?}", gpu.adapter.limits());
    log::info!(
        "b0 wgpu downlevel: {:?}",
        gpu.adapter.get_downlevel_capabilities()
    );

    // SAFETY: the probe makes no Vulkan call that changes wgpu's state.
    let hal_device =
        unsafe { gpu.device.as_hal::<Vulkan>() }.expect("XrGpu opens its device on Vulkan");

    let instance = hal_device.shared_instance().raw_instance();
    let physical_device = hal_device.raw_physical_device();

    // SAFETY: `physical_device` belongs to `instance`.
    let properties = unsafe { instance.get_physical_device_properties(physical_device) };

    log::info!(
        "b0 vulkan api: {}.{}.{}",
        vk::api_version_major(properties.api_version),
        vk::api_version_minor(properties.api_version),
        vk::api_version_patch(properties.api_version)
    );

    // SAFETY: `physical_device` belongs to `instance`.
    let extensions = unsafe { instance.enumerate_device_extension_properties(physical_device) }
        .map_err(Error::Vulkan)?;

    let mut names: Vec<_> = extensions
        .iter()
        .map(|extension| {
            extension
                .extension_name_as_c_str()
                .expect("Vulkan extension names end in nul")
        })
        .collect();

    names.sort();

    log::info!(
        "b0 vulkan extensions: {}",
        names
            .iter()
            .map(|name| name.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" ")
    );

    let has = |name: &CStr| names.contains(&name);
    let has_atomic_int64 = has(shader_atomic_int64::NAME);
    let has_float16 = has(shader_float16_int8::NAME);
    let has_ray_query = has(ray_query::NAME);

    let mut atomic_int64 = PhysicalDeviceShaderAtomicInt64Features::default();
    let mut float16 = PhysicalDeviceShaderFloat16Int8Features::default();
    let mut ray = PhysicalDeviceRayQueryFeaturesKHR::default();
    let mut features = PhysicalDeviceFeatures2::default();

    // A feature struct joins the chain only when its extension is present.
    if has_atomic_int64 {
        features = features.push_next(&mut atomic_int64);
    }

    if has_float16 {
        features = features.push_next(&mut float16);
    }

    if has_ray_query {
        features = features.push_next(&mut ray);
    }

    // SAFETY: `physical_device` belongs to `instance`, and the chain holds
    // only structs of present extensions.
    unsafe { instance.get_physical_device_features2(physical_device, &mut features) };

    let shader_int64 = features.features.shader_int64;

    log::info!("b0 vulkan shaderInt64: {}", shader_int64 == vk::TRUE);

    log::info!(
        "b0 vulkan shaderFloat16: {}",
        has_float16 && float16.shader_float16 == vk::TRUE
    );

    log::info!(
        "b0 vulkan shaderSharedInt64Atomics: {}",
        has_atomic_int64 && atomic_int64.shader_shared_int64_atomics == vk::TRUE
    );

    log::info!(
        "b0 vulkan rayQuery: {}",
        has_ray_query && ray.ray_query == vk::TRUE
    );

    Ok(())
}
