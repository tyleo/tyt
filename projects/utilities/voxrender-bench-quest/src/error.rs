use ash::{LoadingError, vk::Result as VkResult};
use openxr::{EntryError, Version, sys::Result as XrResult};
use std::{
    error::Error as StdError,
    fmt::{Display, Formatter, Result as FmtResult},
    io::Error as IOError,
};
use voxrender_bench::Error as BenchError;
use wgpu::{
    BufferAsyncError, MapRangeError, PollError, RequestDeviceError,
    hal::{DeviceError, InstanceError},
};

/// Why the app stopped.
#[derive(Debug)]
pub enum Error {
    /// The OpenXR loader would not load.
    Entry(EntryError),

    /// An OpenXR call failed.
    Xr(XrResult),

    /// The runtime lacks `XR_KHR_vulkan_enable2`.
    NoVulkanEnable2,

    /// The runtime takes no Vulkan version this app creates.
    VulkanVersion {
        /// The oldest version the runtime takes.
        min: Version,

        /// The newest version the runtime takes.
        max: Version,
    },

    /// The Vulkan library would not load.
    VulkanLoad(LoadingError),

    /// A Vulkan call failed.
    Vulkan(VkResult),

    /// The physical device has no graphics queue.
    NoGraphicsQueue,

    /// wgpu would not wrap the Vulkan instance.
    HalInstance(InstanceError),

    /// wgpu would not wrap the physical device.
    NoAdapter,

    /// The adapter lacks multiview.
    NoMultiview,

    /// wgpu would not wrap the Vulkan device.
    HalDevice(DeviceError),

    /// wgpu would not open the device.
    Device(RequestDeviceError),

    /// The system cannot show an opaque layer.
    NoOpaqueBlend,

    /// The stereo view configuration has other than two views.
    ViewCount(usize),

    /// The two eyes recommend different image sizes.
    UnequalViews,

    /// No swapchain format has a wgpu format this app maps.
    NoSwapchainFormat,

    /// An image size is past what OpenXR's rectangles hold.
    ImageSize(u32),

    /// A visibility mask's indices do not form triangles over its vertices.
    VisibilityMask,

    /// A performance counter changed its unit between samples.
    CounterUnit(&'static str),

    /// Waiting on the GPU failed.
    Poll(PollError),

    /// A readback buffer would not map.
    Map(BufferAsyncError),

    /// A mapped readback range was unavailable.
    MapRange(MapRangeError),

    /// `getprop` would not run.
    Getprop(IOError),

    /// A benchmark failed.
    Bench(BenchError),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Error::Entry(error) => write!(f, "the OpenXR loader failed: {error}"),
            Error::Xr(error) => write!(f, "an OpenXR call failed: {error}"),
            Error::NoVulkanEnable2 => write!(f, "the runtime lacks XR_KHR_vulkan_enable2"),
            Error::VulkanVersion { min, max } => {
                write!(f, "the runtime takes Vulkan {min} to {max}, not 1.1")
            }
            Error::VulkanLoad(error) => write!(f, "Vulkan would not load: {error}"),
            Error::Vulkan(error) => write!(f, "a Vulkan call failed: {error}"),
            Error::NoGraphicsQueue => write!(f, "the GPU has no graphics queue"),
            Error::HalInstance(error) => write!(f, "wgpu rejected the Vulkan instance: {error}"),
            Error::NoAdapter => write!(f, "wgpu rejected the physical device"),
            Error::NoMultiview => write!(f, "the adapter lacks multiview"),
            Error::HalDevice(error) => write!(f, "wgpu rejected the Vulkan device: {error}"),
            Error::Device(error) => write!(f, "wgpu would not open the device: {error}"),
            Error::NoOpaqueBlend => write!(f, "the system has no opaque blend mode"),
            Error::ViewCount(count) => write!(f, "the stereo configuration has {count} views"),
            Error::UnequalViews => write!(f, "the eyes recommend different image sizes"),
            Error::NoSwapchainFormat => write!(f, "no swapchain format maps to wgpu"),
            Error::ImageSize(size) => write!(f, "an image side of {size} is too large"),
            Error::VisibilityMask => write!(f, "a visibility mask's indices are malformed"),
            Error::CounterUnit(label) => write!(f, "the {label} counter changed its unit"),
            Error::Poll(error) => write!(f, "waiting on the GPU failed: {error}"),
            Error::Map(error) => write!(f, "a readback buffer would not map: {error}"),
            Error::MapRange(error) => write!(f, "a readback range was unavailable: {error}"),
            Error::Getprop(error) => write!(f, "getprop would not run: {error}"),
            Error::Bench(error) => write!(f, "a benchmark failed: {error}"),
        }
    }
}

impl StdError for Error {}
