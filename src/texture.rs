use anyhow::Result;
use vulkanalia::prelude::v1_0::*;

use crate::app::AppData;
use crate::bitmap::Bitmap;
use crate::image::{copy_buffer_to_image, create_image, create_image_view, transition_image_layout};
use crate::vulkan::{create_buffer};
type Vec2 = cgmath::Vector2<f32>;
type Vec3 = cgmath::Vector3<f32>;

use std::ptr::copy_nonoverlapping as memcpy;

#[derive(Default)]
pub struct Texture {
    pub image: vk::Image,
    pub image_memory: vk::DeviceMemory,
    pub image_view: vk::ImageView,
    pub sampler: vk::Sampler,
}

impl Texture {
    pub fn new(image: vk::Image, image_memory: vk::DeviceMemory, image_view: vk::ImageView, sampler: vk::Sampler) -> Self {
        Self { image, image_memory, image_view, sampler }
    }

    pub unsafe fn create_from_bitmap(bitmap: &Bitmap, instance: &Instance, device: &Device, data: &mut AppData) -> Result<Self> {
        // TODO: consider using vkMemoryToImageCopy instead of buffer after upgrading to 1.4
        let size = bitmap.pixels.len() as u64;

        let (staging_buffer, staging_buffer_memory) = create_buffer(
            instance,
            device,
            data,
            size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
        )?;

        let memory = device.map_memory(
            staging_buffer_memory,
            0,
            size,
            vk::MemoryMapFlags::empty(),
        )?;

        memcpy(bitmap.pixels.as_ptr(), memory.cast(), bitmap.pixels.len());

        device.unmap_memory(staging_buffer_memory);

        let (texture_image, texture_image_memory) = create_image(
            instance,
            device,
            data,
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
            device,
            data,
            texture_image,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        )?;
        
        copy_buffer_to_image(
            device,
            data,
            staging_buffer,
            texture_image,
            bitmap.width,
            bitmap.height,
        )?;

        transition_image_layout(
            device,
            data,
            texture_image,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        )?;

        device.destroy_buffer(staging_buffer, None);
        device.free_memory(staging_buffer_memory, None);

        let texture_image_view = create_image_view(
            device,
            texture_image,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageAspectFlags::COLOR,
        )?;

        Ok( Self { image: texture_image, image_memory: texture_image_memory, image_view: texture_image_view, sampler: data.texture_sampler } )
    }

}