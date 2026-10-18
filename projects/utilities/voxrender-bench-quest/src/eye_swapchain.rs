use crate::{Error, Result, XrGpu, XrSystem};
use ash::vk::{Format as VkFormat, Handle, Image};
use openxr::{
    Extent2Di, Session, Swapchain, SwapchainCreateFlags, SwapchainCreateInfo, SwapchainUsageFlags,
    ViewConfigurationType, Vulkan as XrVulkan,
};
use wgpu::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureUses,
    TextureView, TextureViewDescriptor, TextureViewDimension,
    hal::{
        MemoryFlags, TextureDescriptor as HalTextureDescriptor, api::Vulkan, vulkan::TextureMemory,
    },
};

/// The two eyes' array layers.
const EYE_COUNT: u32 = 2;

/// The swapchain both eyes draw into, one array layer per eye.
pub struct EyeSwapchain {
    /// The OpenXR swapchain.
    pub handle: Swapchain<XrVulkan>,

    /// A two-layer view of each swapchain image, in the runtime's order.
    pub views: Vec<TextureView>,

    /// Each eye's image size.
    pub extent: Extent2Di,
}

impl EyeSwapchain {
    /// Creates the swapchain at the eyes' recommended size in the runtime's
    /// first format that wgpu takes, and wraps its images as wgpu textures.
    pub fn new(xr: &XrSystem, gpu: &XrGpu, session: &Session<XrVulkan>) -> Result<Self> {
        let eyes = xr
            .instance
            .enumerate_view_configuration_views(xr.system, ViewConfigurationType::PRIMARY_STEREO)
            .map_err(Error::Xr)?;

        let [left, right] = eyes.as_slice() else {
            return Err(Error::ViewCount(eyes.len()));
        };

        if left != right {
            return Err(Error::UnequalViews);
        }

        let width = left.recommended_image_rect_width;
        let height = left.recommended_image_rect_height;

        let extent = Extent2Di {
            width: i32::try_from(width).map_err(|_| Error::ImageSize(width))?,
            height: i32::try_from(height).map_err(|_| Error::ImageSize(height))?,
        };

        let (xr_format, format) = session
            .enumerate_swapchain_formats()
            .map_err(Error::Xr)?
            .into_iter()
            .find_map(|xr_format| to_texture_format(xr_format).map(|format| (xr_format, format)))
            .ok_or(Error::NoSwapchainFormat)?;

        let handle = session
            .create_swapchain(&SwapchainCreateInfo {
                create_flags: SwapchainCreateFlags::EMPTY,
                usage_flags: SwapchainUsageFlags::COLOR_ATTACHMENT,
                format: xr_format,
                sample_count: 1,
                width,
                height,
                face_count: 1,
                array_size: EYE_COUNT,
                mip_count: 1,
            })
            .map_err(Error::Xr)?;

        let size = Extent3d {
            width,
            height,
            depth_or_array_layers: EYE_COUNT,
        };

        let views = handle
            .enumerate_images()
            .map_err(Error::Xr)?
            .into_iter()
            .map(|image| {
                let hal_texture = {
                    // SAFETY: the device is wgpu's wrap of a Vulkan device.
                    let hal_device = unsafe { gpu.device.as_hal::<Vulkan>() }
                        .expect("XrGpu opens its device on Vulkan");

                    // SAFETY: the runtime created `image` to this descriptor,
                    // and the no-op drop leaves the image to the runtime.
                    unsafe {
                        hal_device.texture_from_raw(
                            Image::from_raw(image),
                            &HalTextureDescriptor {
                                label: Some("eye swapchain"),
                                size,
                                mip_level_count: 1,
                                sample_count: 1,
                                dimension: TextureDimension::D2,
                                format,
                                usage: TextureUses::COLOR_TARGET,
                                memory_flags: MemoryFlags::empty(),
                                view_formats: Vec::new(),
                            },
                            Some(Box::new(|| {})),
                            TextureMemory::External,
                        )
                    }
                };

                // SAFETY: OpenXR hands out color images in
                // `COLOR_ATTACHMENT_OPTIMAL` and takes them back in it.
                let texture = unsafe {
                    gpu.device.create_texture_from_hal::<Vulkan>(
                        hal_texture,
                        &TextureDescriptor {
                            label: Some("eye swapchain"),
                            size,
                            mip_level_count: 1,
                            sample_count: 1,
                            dimension: TextureDimension::D2,
                            format,
                            usage: TextureUsages::RENDER_ATTACHMENT,
                            view_formats: &[],
                        },
                        TextureUses::COLOR_TARGET,
                    )
                };

                texture.create_view(&TextureViewDescriptor {
                    label: Some("eye swapchain"),
                    dimension: Some(TextureViewDimension::D2Array),
                    array_layer_count: Some(EYE_COUNT),
                    ..TextureViewDescriptor::default()
                })
            })
            .collect();

        Ok(EyeSwapchain {
            handle,
            views,
            extent,
        })
    }
}

/// The wgpu format of an sRGB swapchain format, the formats a compositor
/// expects.
fn to_texture_format(xr_format: u32) -> Option<TextureFormat> {
    let format = VkFormat::from_raw(i32::try_from(xr_format).ok()?);

    match format {
        VkFormat::R8G8B8A8_SRGB => Some(TextureFormat::Rgba8UnormSrgb),
        VkFormat::B8G8R8A8_SRGB => Some(TextureFormat::Bgra8UnormSrgb),
        _ => None,
    }
}
