use anyhow::{Result};
use vulkanalia::vk::{self, DeviceV1_0, Handle, HasBuilder, KhrSurfaceExtensionInstanceCommands, KhrSwapchainExtensionDeviceCommands};
use winit::window::Window;
use crate::{vulkan::create_image_view, vulkan_context::VulkanContext};

pub struct SwapchainData {
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_images: Vec<vk::Image>,
    pub swapchain_format: vk::Format,
    pub swapchain_extent: vk::Extent2D,
    pub swapchain_image_views: Vec<vk::ImageView>,
}

impl SwapchainData {
    pub unsafe fn new(window: &Window, vulkan_context: &VulkanContext) -> Result<Self> {
        let (swapchain, swapchain_images, swapchain_format, swapchain_extent) = Self::create_swapchain(window, vulkan_context)?;
        let swapchain_image_views = Self::create_swapchain_image_views(vulkan_context, &swapchain_images, swapchain_format)?;
        
        Ok(Self { swapchain, swapchain_images, swapchain_format, swapchain_extent, swapchain_image_views })
    }

    pub unsafe fn destroy(self, vulkan_context: &VulkanContext) {
        self.swapchain_image_views
            .iter()
            .for_each(|v| vulkan_context.device.destroy_image_view(*v, None));

        vulkan_context.device.destroy_swapchain_khr(self.swapchain, None);
    }

    unsafe fn create_swapchain(window: &Window, vulkan_context: &VulkanContext) -> Result<(vk::SwapchainKHR, Vec<vk::Image>, vk::Format, vk::Extent2D)> {
        let (graphics, present) = vulkan_context.queue_family_indices()?;
        let capabilities = vulkan_context.instance.get_physical_device_surface_capabilities_khr(vulkan_context.physical_device, vulkan_context.surface)?;
        let formats = vulkan_context.instance.get_physical_device_surface_formats_khr(vulkan_context.physical_device, vulkan_context.surface)?;
        let present_modes = vulkan_context.instance.get_physical_device_surface_present_modes_khr(vulkan_context.physical_device, vulkan_context.surface)?;

        let surface_format = Self::get_swapchain_surface_format(&formats);
        let present_mode = Self::get_swapchain_present_mode(&present_modes);
        let extent = Self::get_swapchain_extent(window, capabilities);

        let mut image_count = capabilities.min_image_count + 1;
        if capabilities.max_image_count != 0 && image_count > capabilities.max_image_count {
            image_count = capabilities.max_image_count;
        }

        let mut queue_family_indices = vec![];
        let image_sharing_mode = if graphics != present {
            queue_family_indices.push(graphics);
            queue_family_indices.push(present);
            vk::SharingMode::CONCURRENT
        } else {
            vk::SharingMode::EXCLUSIVE
        };

        let info = vk::SwapchainCreateInfoKHR::builder()
            .surface(vulkan_context.surface)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(image_sharing_mode)
            .queue_family_indices(&queue_family_indices)
            .pre_transform(capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .old_swapchain(vk::SwapchainKHR::null());

        let swapchain = vulkan_context.device.create_swapchain_khr(&info, None)?;
        let swapchain_images = vulkan_context.device.get_swapchain_images_khr(swapchain)?;
        let swapchain_format = surface_format.format;
        let swapchain_extent = extent;

        Ok((swapchain, swapchain_images, swapchain_format, swapchain_extent))
    }

    unsafe fn create_swapchain_image_views(vulkan_context: &VulkanContext, swapchain_images: &Vec<vk::Image>, swapchain_format: vk::Format) -> Result<Vec<vk::ImageView>> {
        swapchain_images
            .iter()
            .map(|i| create_image_view(vulkan_context, *i, swapchain_format, vk::ImageAspectFlags::COLOR))
            .collect::<Result<Vec<_>, _>>()
    }

    fn get_swapchain_surface_format(formats: &[vk::SurfaceFormatKHR]) -> vk::SurfaceFormatKHR {
        formats
            .iter()
            .cloned()
            .find(|f| {
                f.format == vk::Format::B8G8R8A8_SRGB && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            })
            .unwrap_or_else(|| formats[0])
    }

    fn get_swapchain_present_mode(present_modes: &[vk::PresentModeKHR]) -> vk::PresentModeKHR {
        present_modes
            .iter()
            .cloned()
            .find(|m| *m == vk::PresentModeKHR::MAILBOX)
            .unwrap_or(vk::PresentModeKHR::FIFO)
    }

    fn get_swapchain_extent(window: &Window, capabilities: vk::SurfaceCapabilitiesKHR) -> vk::Extent2D {
        if capabilities.current_extent.width != u32::MAX {
            capabilities.current_extent
        } else {
            vk::Extent2D::builder()
                .width(window.inner_size().width.clamp(capabilities.min_image_extent.width, capabilities.max_image_extent.width))
                .height(window.inner_size().height.clamp(capabilities.min_image_extent.height, capabilities.max_image_extent.height))
                .build()
        }
    }
}
