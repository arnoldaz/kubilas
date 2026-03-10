use anyhow::Result;
use vulkanalia::prelude::v1_0::*;

use crate::bitmap::Bitmap;
use crate::command::CommandData;
use crate::registry::Destroy;
use crate::vulkan::{copy_buffer_to_image, create_buffer, create_image, create_image_ui, create_image_view, transition_image_layout};
use crate::vulkan_context::{VulkanContext};

use std::ptr::copy_nonoverlapping as memcpy;

#[derive(Default)]
pub struct Texture {
    pub image: vk::Image,
    pub image_memory: vk::DeviceMemory,
    pub image_view: vk::ImageView,
    pub _sampler: vk::Sampler,
}

impl Texture {
    pub fn _new(image: vk::Image, image_memory: vk::DeviceMemory, image_view: vk::ImageView, sampler: vk::Sampler) -> Self {
        Self { image, image_memory, image_view, _sampler: sampler }
    }

    pub unsafe fn create_from_bitmap(bitmap: &Bitmap, vulkan_context: &VulkanContext, command_data: &CommandData, sampler: vk::Sampler) -> Result<Self> {
        // TODO: consider using vkMemoryToImageCopy instead of buffer after upgrading to 1.4
        let size = bitmap.pixels.len() as u64;

        let (staging_buffer, staging_buffer_memory) = create_buffer(
            vulkan_context,
            size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
        )?;

        let memory = vulkan_context.device.map_memory(
            staging_buffer_memory,
            0,
            size,
            vk::MemoryMapFlags::empty(),
        )?;

        memcpy(bitmap.pixels.as_ptr(), memory.cast(), bitmap.pixels.len());

        vulkan_context.device.unmap_memory(staging_buffer_memory);

        let (texture_image, texture_image_memory) = create_image(
            vulkan_context,
            bitmap.width,
            bitmap.height,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        // data.texture_image = texture_image;
        // data.texture_image_memory = texture_image_memory;

        transition_image_layout(
            vulkan_context,
            command_data,
            texture_image,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        )?;
        
        copy_buffer_to_image(
            vulkan_context,
            command_data,
            staging_buffer,
            texture_image,
            bitmap.width,
            bitmap.height,
        )?;

        transition_image_layout(
            vulkan_context,
            command_data,
            texture_image,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        )?;

        vulkan_context.device.destroy_buffer(staging_buffer, None);
        vulkan_context.device.free_memory(staging_buffer_memory, None);

        let texture_image_view = create_image_view(
            vulkan_context,
            texture_image,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageAspectFlags::COLOR,
        )?;

        Ok( Self { image: texture_image, image_memory: texture_image_memory, image_view: texture_image_view, _sampler: sampler } )
    }

    pub unsafe fn create_from_bitmap_ui(bitmap: &Bitmap, vulkan_context: &VulkanContext, command_data: &CommandData, sampler: vk::Sampler, extent: vk::Extent3D) -> Result<Self> {
        // TODO: consider using vkMemoryToImageCopy instead of buffer after upgrading to 1.4
        let size = bitmap.pixels.len() as u64;

        let (staging_buffer, staging_buffer_memory) = create_buffer(
            vulkan_context,
            size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
        )?;

        let memory = vulkan_context.device.map_memory(
            staging_buffer_memory,
            0,
            size,
            vk::MemoryMapFlags::empty(),
        )?;

        memcpy(bitmap.pixels.as_ptr(), memory.cast(), bitmap.pixels.len());

        vulkan_context.device.unmap_memory(staging_buffer_memory);

        let (texture_image, texture_image_memory) = create_image_ui(
            vulkan_context,
            bitmap.width,
            bitmap.height,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            extent
        )?;

        // data.texture_image = texture_image;
        // data.texture_image_memory = texture_image_memory;

        transition_image_layout(
            vulkan_context,
            command_data,
            texture_image,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        )?;
        
        copy_buffer_to_image(
            vulkan_context,
            command_data,
            staging_buffer,
            texture_image,
            bitmap.width,
            bitmap.height,
        )?;

        transition_image_layout(
            vulkan_context,
            command_data,
            texture_image,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        )?;

        vulkan_context.device.destroy_buffer(staging_buffer, None);
        vulkan_context.device.free_memory(staging_buffer_memory, None);

        let texture_image_view = create_image_view(
            vulkan_context,
            texture_image,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageAspectFlags::COLOR,
        )?;

        Ok( Self { image: texture_image, image_memory: texture_image_memory, image_view: texture_image_view, _sampler: sampler } )
    }
}

impl Destroy for Texture {
    unsafe fn destroy(self, vulkan_context: &VulkanContext) {
        vulkan_context.device.destroy_image_view(self.image_view, None);
        vulkan_context.device.destroy_image(self.image, None);
        vulkan_context.device.free_memory(self.image_memory, None);
    }
}