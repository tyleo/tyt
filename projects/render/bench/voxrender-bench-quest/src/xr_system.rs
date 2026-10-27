use crate::{Error, Result};
use android_activity::AndroidApp;
use openxr::{
    AndroidPlatformInfo, ApplicationInfo, Entry, ExtensionSet, FormFactor, Instance, SystemId,
    Version,
};

/// The headset's OpenXR instance and system.
pub struct XrSystem {
    /// The OpenXR instance.
    pub instance: Instance,

    /// The head-mounted display.
    pub system: SystemId,

    /// The runtime's extensions, enabled or not.
    pub available: ExtensionSet,
}

impl XrSystem {
    /// Loads the APK's OpenXR loader and opens the head-mounted display with
    /// Vulkan enabled.
    pub fn new(app: &AndroidApp) -> Result<Self> {
        // SAFETY: android-activity's VM and activity outlive the app.
        let platform = unsafe { AndroidPlatformInfo::new(app.vm_as_ptr(), app.activity_as_ptr()) };

        // SAFETY: the APK ships the Khronos loader.
        let entry = unsafe { Entry::load(&platform) }.map_err(Error::Entry)?;

        let available = entry.enumerate_extensions().map_err(Error::Xr)?;

        if !available.khr_vulkan_enable2 {
            return Err(Error::NoVulkanEnable2);
        }

        // B0 probes the optional extensions the runtime has.
        let mut extensions = ExtensionSet::default();
        extensions.khr_android_create_instance = true;
        extensions.khr_vulkan_enable2 = true;
        extensions.khr_visibility_mask = available.khr_visibility_mask;
        extensions.fb_display_refresh_rate = available.fb_display_refresh_rate;
        extensions.fb_space_warp = available.fb_space_warp;
        extensions.ext_frame_synthesis = available.ext_frame_synthesis;
        extensions.meta_vulkan_swapchain_create_info = available.meta_vulkan_swapchain_create_info;
        extensions.meta_performance_metrics = available.meta_performance_metrics;

        let instance = entry
            .create_instance(
                &ApplicationInfo {
                    application_name: "voxrender-bench-quest",
                    application_version: 0,
                    engine_name: "voxrender",
                    engine_version: 0,
                    // Quad views need OpenXR 1.1.
                    api_version: Version::new(1, 1, 0),
                },
                &extensions,
                &[],
                &platform,
            )
            .map_err(Error::Xr)?;

        let system = instance
            .system(FormFactor::HEAD_MOUNTED_DISPLAY)
            .map_err(Error::Xr)?;

        Ok(XrSystem {
            instance,
            system,
            available,
        })
    }
}
