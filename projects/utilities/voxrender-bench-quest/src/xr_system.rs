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
}

impl XrSystem {
    /// Loads the APK's OpenXR loader and opens the head-mounted display with
    /// Vulkan enabled.
    pub fn new(app: &AndroidApp) -> Result<Self> {
        // SAFETY: android-activity's VM and activity outlive the app.
        let platform = unsafe { AndroidPlatformInfo::new(app.vm_as_ptr(), app.activity_as_ptr()) };

        // SAFETY: the APK ships the Khronos loader.
        let entry = unsafe { Entry::load(&platform) }.map_err(Error::Entry)?;

        if !entry
            .enumerate_extensions()
            .map_err(Error::Xr)?
            .khr_vulkan_enable2
        {
            return Err(Error::NoVulkanEnable2);
        }

        let mut extensions = ExtensionSet::default();
        extensions.khr_android_create_instance = true;
        extensions.khr_vulkan_enable2 = true;

        let instance = entry
            .create_instance(
                &ApplicationInfo {
                    application_name: "voxrender-bench-quest",
                    application_version: 0,
                    engine_name: "voxrender",
                    engine_version: 0,
                    api_version: Version::new(1, 0, 0),
                },
                &extensions,
                &[],
                &platform,
            )
            .map_err(Error::Xr)?;

        let system = instance
            .system(FormFactor::HEAD_MOUNTED_DISPLAY)
            .map_err(Error::Xr)?;

        Ok(XrSystem { instance, system })
    }
}
