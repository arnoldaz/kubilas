use anyhow::Result;


// use log::*;
use vulkanalia::prelude::v1_0::*;

use std::mem::size_of;
use cgmath::{Euler, Rad, Vector3};

use crate::app::AppData;
use crate::cpu_render_object::CpuRenderObject;
use crate::image::{copy_buffer_to_image, create_image, create_image_view, transition_image_layout};
use crate::vertex::Vertex;
use crate::vulkan::{copy_buffer, create_buffer};
type Vec2 = cgmath::Vector2<f32>;
type Vec3 = cgmath::Vector3<f32>;


use std::ptr::copy_nonoverlapping as memcpy;
use vulkanalia_vma::{self as vma, Alloc, Allocator};



#[derive(Clone, Debug)]
pub struct GpuRenderObject {
    pub texture_image: vk::Image,
    pub texture_image_memory: vk::DeviceMemory,
    pub texture_image_view: vk::ImageView,
    pub indices_count: u32,
    pub vertex_buffer: vk::Buffer,
    pub vertex_allocation: vma::Allocation,
    pub index_buffer: vk::Buffer,
    pub index_allocation: vma::Allocation,
    pub translation: Vector3<f32>,
    pub rotation: Euler<Rad<f32>>,
    pub scale: Vector3<f32>,
    pub sampler_index: u32,
}

impl GpuRenderObject {
    pub unsafe fn new(vulkan_instance: &Instance, vulkan_device: &Device, app_data: &mut AppData, cpu_render_object: &CpuRenderObject) -> Result<Self> {
        let (texture_image, texture_image_memory, texture_image_view, sampler_index) = create_texture_image(vulkan_instance, vulkan_device, app_data, cpu_render_object)?;

        // TODO: check how can I pass rust vector readonly to function
        let (vertex_buffer, vertex_allocation) = create_vertex_buffer(vulkan_instance, vulkan_device, app_data, cpu_render_object)?;
        let (index_buffer, index_allocation) = create_index_buffer(vulkan_instance, vulkan_device, app_data, cpu_render_object)?;

        Ok(GpuRenderObject {
            texture_image,
            texture_image_memory,
            texture_image_view,
            indices_count: cpu_render_object.indices.len() as u32,
            vertex_buffer,
            vertex_allocation,
            index_buffer,
            index_allocation,
            translation: cpu_render_object.translation,
            rotation: cpu_render_object.rotation,
            scale: cpu_render_object.scale,
            sampler_index
        })    
    }

    pub unsafe fn destroy(&self, vulkan_device: &Device, allocator: &vma::Allocator) {
        vulkan_device.destroy_image_view(self.texture_image_view, None);
        vulkan_device.destroy_image(self.texture_image, None);
        vulkan_device.free_memory(self.texture_image_memory, None);

        allocator.destroy_buffer(self.index_buffer, self.index_allocation);
        allocator.destroy_buffer(self.vertex_buffer, self.vertex_allocation);
    }
}


pub unsafe fn create_texture_image(vulkan_instance: &Instance, vulkan_device: &Device, app_data: &mut AppData, cpu_render_object: &CpuRenderObject) -> Result<(vk::Image, vk::DeviceMemory, vk::ImageView, u32)> {
    // TODO: consider using vkMemoryToImageCopy instead of buffer after upgrading to 1.4

    let size = cpu_render_object.pixels.len() as u64;

    let (staging_buffer, staging_buffer_memory) = create_buffer(
        vulkan_instance,
        vulkan_device,
        app_data,
        size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
    )?;

    let memory = vulkan_device.map_memory(
        staging_buffer_memory,
        0,
        size,
        vk::MemoryMapFlags::empty(),
    )?;

    memcpy(cpu_render_object.pixels.as_ptr(), memory.cast(), cpu_render_object.pixels.len());

    vulkan_device.unmap_memory(staging_buffer_memory);

    let (texture_image, texture_image_memory) = create_image(
        vulkan_instance,
        vulkan_device,
        app_data,
        cpu_render_object.width,
        cpu_render_object.height,
        vk::Format::R8G8B8A8_SRGB,
        vk::ImageTiling::OPTIMAL,
        vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    // data.texture_image = texture_image;
    // data.texture_image_memory = texture_image_memory;

    transition_image_layout(
        vulkan_device,
        app_data,
        texture_image,
        vk::Format::R8G8B8A8_SRGB,
        vk::ImageLayout::UNDEFINED,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
    )?;
    
    copy_buffer_to_image(
        vulkan_device,
        app_data,
        staging_buffer,
        texture_image,
        cpu_render_object.width,
        cpu_render_object.height,
    )?;

    transition_image_layout(
        vulkan_device,
        app_data,
        texture_image,
        vk::Format::R8G8B8A8_SRGB,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    )?;

    vulkan_device.destroy_buffer(staging_buffer, None);
    vulkan_device.free_memory(staging_buffer_memory, None);

    let texture_image_view = create_image_view(
        vulkan_device,
        texture_image,
        vk::Format::R8G8B8A8_SRGB,
        vk::ImageAspectFlags::COLOR,
    )?;

    app_data.sampler_index += 1;
    let image_info = vk::DescriptorImageInfo::builder()
        .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .image_view(texture_image_view)
        .sampler(app_data.texture_sampler);

    let image_infos = &[image_info];
    let sampler_write = vk::WriteDescriptorSet::builder()
        .dst_set(app_data.descriptor_set)
        .dst_binding(1)
        .dst_array_element(app_data.sampler_index as u32)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .image_info(image_infos);

    vulkan_device.update_descriptor_sets(&[sampler_write], &[] as &[vk::CopyDescriptorSet]);

    Ok((texture_image, texture_image_memory, texture_image_view, app_data.sampler_index as u32))
}


pub unsafe fn create_vertex_buffer(vulkan_instance: &Instance, vulkan_device: &Device, app_data: &AppData, cpu_render_object: &CpuRenderObject) -> Result<(vk::Buffer, vma::Allocation)> {
    let allocator = app_data.allocator.as_ref().unwrap();
    create_gpu_buffer(allocator, &cpu_render_object.vertices, vk::BufferUsageFlags::VERTEX_BUFFER)
}

pub unsafe fn create_index_buffer(vulkan_instance: &Instance, vulkan_device: &Device, app_data: &AppData, cpu_render_object: &CpuRenderObject) -> Result<(vk::Buffer, vma::Allocation)> {
    let allocator = app_data.allocator.as_ref().unwrap();
    create_gpu_buffer(allocator, &cpu_render_object.indices, vk::BufferUsageFlags::INDEX_BUFFER)
}

pub unsafe fn create_gpu_buffer<T: Copy, S: AsRef<[T]>>(allocator: &Allocator, data: S, usage_flag: vk::BufferUsageFlags) -> Result<(vk::Buffer, vma::Allocation)> {
    let data_slice = data.as_ref();
    let size = (size_of::<T>() * data_slice.len()) as u64;

    let buffer_create_info = vk::BufferCreateInfo::builder()
        .size(size)
        .usage(vk::BufferUsageFlags::TRANSFER_DST | usage_flag)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let allocation_options = vma::AllocationOptions {
        flags: vma::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE | vma::AllocationCreateFlags::HOST_ACCESS_ALLOW_TRANSFER_INSTEAD,
        usage: vma::MemoryUsage::Auto,
        ..Default::default()
    };

    let (buffer, allocation) = allocator.create_buffer(buffer_create_info, &allocation_options)?;

    let mapped = allocator.map_memory(allocation)? as *mut T;
    mapped.copy_from_nonoverlapping(data_slice.as_ptr(), data_slice.len());
    allocator.unmap_memory(allocation);

    Ok((buffer, allocation))
}