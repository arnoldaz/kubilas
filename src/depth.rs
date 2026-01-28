use vulkanalia::vk::{self, DeviceV1_0, InstanceV1_0};
use anyhow::{anyhow, Result};
use crate::{command::CommandData, swapchain::SwapchainData, vulkan::{create_image, create_image_view, transition_image_layout}, vulkan_context::VulkanContext};

pub struct DepthResources {
    pub depth_format: vk::Format,
    pub depth_image: vk::Image,
    pub depth_image_memory: vk::DeviceMemory,
    pub depth_image_view: vk::ImageView,
}

impl DepthResources {
    pub unsafe fn new(vulkan_context: &VulkanContext, swapchain_data: &SwapchainData, command_data: &CommandData) -> Result<Self> {
        let depth_format = Self::get_depth_format(vulkan_context)?;
        let (depth_image, depth_image_memory, depth_image_view) = Self::create_depth_objects(depth_format, vulkan_context, swapchain_data, command_data)?;

        Ok(Self { depth_format, depth_image, depth_image_memory, depth_image_view })
    }

    pub unsafe fn destroy(self, vulkan_context: &VulkanContext) {
        vulkan_context.device.destroy_image_view(self.depth_image_view, None);
        vulkan_context.device.free_memory(self.depth_image_memory, None);
        vulkan_context.device.destroy_image(self.depth_image, None);
    }

    unsafe fn create_depth_objects(depth_format: vk::Format, vulkan_context: &VulkanContext, swapchain_data: &SwapchainData, command_data: &CommandData) -> Result<(vk::Image, vk::DeviceMemory, vk::ImageView)> {       
        let (depth_image, depth_image_memory) = create_image(
            vulkan_context,
            swapchain_data.swapchain_extent.width,
            swapchain_data.swapchain_extent.height,
            depth_format,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        let depth_image_view = create_image_view(vulkan_context, depth_image, depth_format, vk::ImageAspectFlags::DEPTH)?;

        transition_image_layout(
            vulkan_context,
            command_data,
            depth_image,
            depth_format,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        )?;

        Ok((depth_image, depth_image_memory, depth_image_view))
    }

    unsafe fn get_depth_format(vulkan_context: &VulkanContext) -> Result<vk::Format> {
        let candidates = &[
            vk::Format::D24_UNORM_S8_UINT,
            vk::Format::D32_SFLOAT_S8_UINT,
            vk::Format::D32_SFLOAT,
        ];

        Self::get_supported_format(
            vulkan_context,
            candidates,
            vk::ImageTiling::OPTIMAL,
            vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
        )
    }

    unsafe fn get_supported_format(vulkan_context: &VulkanContext, candidates: &[vk::Format], tiling: vk::ImageTiling, features: vk::FormatFeatureFlags) -> Result<vk::Format> {
        candidates
            .iter()
            .cloned()
            .find(|f| {
                let properties = vulkan_context.instance.get_physical_device_format_properties(vulkan_context.physical_device, *f);
                match tiling {
                    vk::ImageTiling::LINEAR => properties.linear_tiling_features.contains(features),
                    vk::ImageTiling::OPTIMAL => properties.optimal_tiling_features.contains(features),
                    _ => false,
                }
            })
            .ok_or_else(|| anyhow!("Failed to find supported format"))
    }
}