use crate::{Error, Result, XrSystem};
use ash::{
    Device as VkDevice, Entry as VkEntry, Instance as VkInstance,
    vk::{
        self, ApplicationInfo, DeviceCreateInfo, DeviceQueueCreateInfo, Handle, InstanceCreateInfo,
        PFN_vkGetInstanceProcAddr, PhysicalDevice, QueueFlags, Result as VkResult,
    },
};
use openxr::{
    Version, Vulkan as XrVulkan, sys::platform::VkGetInstanceProcAddr, vulkan::SessionCreateInfo,
};
use std::mem;
use wgpu::{
    Adapter, Device, DeviceDescriptor, ExperimentalFeatures, Features, Instance, InstanceFlags,
    MemoryBudgetThresholds, MemoryHints, Queue,
    hal::{api::Vulkan, vulkan::Instance as HalInstance},
};

/// The Vulkan version the app creates.
const VULKAN_VERSION: Version = Version::new(1, 1, 0);

/// A wgpu device on the Vulkan instance and device the OpenXR runtime made.
pub struct XrGpu {
    /// The adapter the device runs on.
    pub adapter: Adapter,

    /// The device.
    pub device: Device,

    /// The device's queue.
    pub queue: Queue,

    /// The raw handles an OpenXR session binds to.
    pub session_info: SessionCreateInfo,
}

impl XrGpu {
    /// Has the runtime create a Vulkan instance and device that wgpu's
    /// extensions and features need, then wraps them in wgpu.
    ///
    /// # Arguments
    ///
    /// 1. `android_sdk_version`: the Android API level, which wgpu's Vulkan
    ///    workarounds check
    pub fn new(xr: &XrSystem, android_sdk_version: u32) -> Result<Self> {
        let requirements = xr
            .instance
            .graphics_requirements::<XrVulkan>(xr.system)
            .map_err(Error::Xr)?;

        let min = requirements.min_api_version_supported;
        let max = requirements.max_api_version_supported;

        if VULKAN_VERSION < min || VULKAN_VERSION.major() > max.major() {
            return Err(Error::VulkanVersion { min, max });
        }

        let api_version = vk::make_api_version(
            0,
            u32::from(VULKAN_VERSION.major()),
            u32::from(VULKAN_VERSION.minor()),
            VULKAN_VERSION.patch(),
        );

        // SAFETY: the system's Vulkan library follows the Vulkan spec.
        let entry = unsafe { VkEntry::load() }.map_err(Error::VulkanLoad)?;

        // SAFETY: OpenXR's and ash's `vkGetInstanceProcAddr` types share one
        // C signature.
        let get_instance_proc_addr = unsafe {
            mem::transmute::<PFN_vkGetInstanceProcAddr, VkGetInstanceProcAddr>(
                entry.static_fn().get_instance_proc_addr,
            )
        };

        let flags = InstanceFlags::from_build_config();

        let instance_extensions = HalInstance::desired_extensions(&entry, api_version, flags)
            .map_err(Error::HalInstance)?;

        let instance_extension_names: Vec<_> = instance_extensions
            .iter()
            .map(|name| name.as_ptr())
            .collect();

        let app_info = ApplicationInfo::default()
            .application_name(c"voxrender-bench-quest")
            .engine_name(c"voxrender")
            .api_version(api_version);

        let instance_info = InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&instance_extension_names);

        // SAFETY: `instance_info` and the names it points to are alive for
        // the call.
        let raw_instance = unsafe {
            xr.instance.create_vulkan_instance(
                xr.system,
                get_instance_proc_addr,
                (&raw const instance_info).cast(),
            )
        }
        .map_err(Error::Xr)?
        .map_err(|result| Error::Vulkan(VkResult::from_raw(result)))?;

        // SAFETY: the runtime created `raw_instance` from `entry`.
        let vk_instance = unsafe {
            VkInstance::load(
                entry.static_fn(),
                vk::Instance::from_raw(raw_instance as u64),
            )
        };

        // SAFETY: `raw_instance` is the runtime's live Vulkan instance.
        let raw_physical_device =
            unsafe { xr.instance.vulkan_graphics_device(xr.system, raw_instance) }
                .map_err(Error::Xr)?;

        let physical_device = PhysicalDevice::from_raw(raw_physical_device as u64);

        // SAFETY: `physical_device` belongs to `vk_instance`.
        let queue_families =
            unsafe { vk_instance.get_physical_device_queue_family_properties(physical_device) };

        let queue_family_index = queue_families
            .iter()
            .position(|family| family.queue_flags.contains(QueueFlags::GRAPHICS))
            .ok_or(Error::NoGraphicsQueue)?;

        let queue_family_index =
            u32::try_from(queue_family_index).expect("Vulkan counts queue families in u32");

        // SAFETY: the runtime created `vk_instance` from `entry` with
        // `api_version` and `instance_extensions`, and wgpu destroys it.
        let hal_instance = unsafe {
            HalInstance::from_raw(
                entry,
                vk_instance.clone(),
                api_version,
                android_sdk_version,
                None,
                instance_extensions,
                flags,
                MemoryBudgetThresholds::default(),
                false,
                None,
            )
        }
        .map_err(Error::HalInstance)?;

        let exposed = hal_instance
            .expose_adapter(physical_device)
            .ok_or(Error::NoAdapter)?;

        let required =
            Features::MULTIVIEW | Features::TIMESTAMP_QUERY | Features::EXPERIMENTAL_RAY_QUERY;

        if !exposed.features.contains(required) {
            return Err(Error::MissingFeatures(required - exposed.features));
        }

        // B2 reports the subgroup sizes when the device has subgroups.
        let features = required | (exposed.features & Features::SUBGROUP);

        let limits = exposed.capabilities.limits.clone();
        let device_extensions = exposed.adapter.required_device_extensions(features);

        let mut physical_features = exposed
            .adapter
            .physical_device_features(&device_extensions, features);

        let device_extension_names: Vec<_> =
            device_extensions.iter().map(|name| name.as_ptr()).collect();

        let queue_priorities = [1.0];

        let queue_infos = [DeviceQueueCreateInfo::default()
            .queue_family_index(queue_family_index)
            .queue_priorities(&queue_priorities)];

        let device_info = physical_features
            .add_to_device_create(DeviceCreateInfo::default().queue_create_infos(&queue_infos))
            .enabled_extension_names(&device_extension_names);

        // SAFETY: `device_info` and everything it points to are alive for the
        // call.
        let raw_device = unsafe {
            xr.instance.create_vulkan_device(
                xr.system,
                get_instance_proc_addr,
                raw_physical_device,
                (&raw const device_info).cast(),
            )
        }
        .map_err(Error::Xr)?
        .map_err(|result| Error::Vulkan(VkResult::from_raw(result)))?;

        // SAFETY: the runtime created `raw_device` on `vk_instance`.
        let vk_device = unsafe {
            VkDevice::load(
                vk_instance.fp_v1_0(),
                vk::Device::from_raw(raw_device as u64),
            )
        };

        // SAFETY: `vk_device` was created with `device_extensions` and
        // `physical_features` on `queue_family_index`, and wgpu destroys it.
        let open_device = unsafe {
            exposed.adapter.device_from_raw(
                vk_device,
                None,
                &device_extensions,
                features,
                &limits,
                &MemoryHints::Performance,
                queue_family_index,
                0,
            )
        }
        .map_err(Error::HalDevice)?;

        // SAFETY: `hal_instance` is a live Vulkan instance.
        let instance = unsafe { Instance::from_hal::<Vulkan>(hal_instance) };

        // SAFETY: `exposed` came from `hal_instance`.
        let adapter = unsafe { instance.create_adapter_from_hal(exposed) };

        // SAFETY: the bench accepts ray queries' experimental status. Its ids
        // check their hits against the reference.
        let experimental_features = unsafe { ExperimentalFeatures::enabled() };

        // SAFETY: `open_device` came from `adapter` with `features`.
        let (device, queue) = unsafe {
            adapter.create_device_from_hal(
                open_device,
                &DeviceDescriptor {
                    label: Some("voxrender-bench-quest"),
                    required_features: features,
                    required_limits: limits,
                    experimental_features,
                    ..DeviceDescriptor::default()
                },
            )
        }
        .map_err(Error::Device)?;

        Ok(XrGpu {
            adapter,
            device,
            queue,
            session_info: SessionCreateInfo {
                instance: raw_instance,
                physical_device: raw_physical_device,
                device: raw_device,
                queue_family_index,
                queue_index: 0,
            },
        })
    }
}
