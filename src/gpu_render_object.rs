use thiserror::Error;
use anyhow::{anyhow, Result};
use winit::dpi::LogicalSize;
use winit::event::{ElementState, Event, KeyEvent, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowBuilder};

use std::collections::HashSet;
use std::ffi::CStr;
use std::fs::File;
use std::io::BufReader;
use std::os::raw::c_void;

// use log::*;
use vulkanalia::loader::{LibloadingLoader, LIBRARY};
use vulkanalia::window as vk_window;
use vulkanalia::prelude::v1_0::*;
use vulkanalia::vk::ExtDebugUtilsExtension;
use vulkanalia::vk::KhrSurfaceExtension;
use vulkanalia::vk::KhrSwapchainExtension;
use vulkanalia::bytecode::Bytecode;

use std::mem::size_of;
use cgmath::{Quaternion, Vector3, vec2, vec3};

use crate::app::AppData;
use crate::cpu_render_object::{self, CpuRenderObject};
use crate::image::{copy_buffer_to_image, create_image, create_image_view, transition_image_layout};
use crate::vertex::Vertex;
use crate::vulkan::{copy_buffer, create_buffer};
type Vec2 = cgmath::Vector2<f32>;
type Vec3 = cgmath::Vector3<f32>;

use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use std::ptr::copy_nonoverlapping as memcpy;


#[derive(Clone, Debug)]
pub struct GpuRenderObject {
    pub texture_image: vk::Image,
    pub texture_image_memory: vk::DeviceMemory,
    pub texture_image_view: vk::ImageView,
    pub indices_count: u32,
    pub vertex_buffer: vk::Buffer,
    pub vertex_buffer_memory: vk::DeviceMemory,
    pub index_buffer: vk::Buffer,
    pub index_buffer_memory: vk::DeviceMemory,
    pub translation: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>,
}

impl GpuRenderObject {
    pub unsafe fn new(vulkan_instance: &Instance, vulkan_device: &Device, app_data: &AppData, cpu_render_object: &CpuRenderObject) -> Result<Self> {
        let (texture_image, texture_image_memory) = create_texture_image(vulkan_instance, vulkan_device, app_data, cpu_render_object)?;
        let texture_image_view = create_texture_image_view(vulkan_device, texture_image)?;

        // TODO: check how can I pass rust vector readonly to function
        let (vertex_buffer, vertex_buffer_memory) = create_vertex_buffer(vulkan_instance, vulkan_device, app_data, cpu_render_object)?;
        let (index_buffer, index_buffer_memory) = create_index_buffer(vulkan_instance, vulkan_device, app_data, cpu_render_object)?;

        Ok(GpuRenderObject {
            texture_image,
            texture_image_memory,
            texture_image_view,
            indices_count: cpu_render_object.indices.len() as u32,
            vertex_buffer,
            vertex_buffer_memory,
            index_buffer,
            index_buffer_memory,
            translation: cpu_render_object.translation,
            rotation: cpu_render_object.rotation,
            scale: cpu_render_object.scale,
        })    
    }

    pub unsafe fn destroy(&self, vulkan_device: &Device) {
        vulkan_device.destroy_image_view(self.texture_image_view, None);
        vulkan_device.destroy_image(self.texture_image, None);
        vulkan_device.free_memory(self.texture_image_memory, None);

        vulkan_device.destroy_buffer(self.index_buffer, None);
        vulkan_device.free_memory(self.index_buffer_memory, None);
        vulkan_device.destroy_buffer(self.vertex_buffer, None);
        vulkan_device.free_memory(self.vertex_buffer_memory, None);
    }
}


pub unsafe fn create_texture_image(vulkan_instance: &Instance, vulkan_device: &Device, app_data: &AppData, cpu_render_object: &CpuRenderObject) -> Result<(vk::Image, vk::DeviceMemory)> {
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

    Ok((texture_image, texture_image_memory))
}

pub unsafe fn create_texture_image_view(vulkan_device: &Device, texture_image: vk::Image) -> Result<vk::ImageView> {
    let texture_image_view = create_image_view(
        vulkan_device,
        texture_image,
        vk::Format::R8G8B8A8_SRGB,
        vk::ImageAspectFlags::COLOR,
    )?;

    Ok(texture_image_view)
}

pub unsafe fn create_vertex_buffer(vulkan_instance: &Instance, vulkan_device: &Device, app_data: &AppData, cpu_render_object: &CpuRenderObject) -> Result<(vk::Buffer, vk::DeviceMemory)> {
    let size = (size_of::<Vertex>() * cpu_render_object.vertices.len()) as u64;

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

    memcpy(cpu_render_object.vertices.as_ptr(), memory.cast(), cpu_render_object.vertices.len());

    vulkan_device.unmap_memory(staging_buffer_memory);

    let (vertex_buffer, vertex_buffer_memory) = create_buffer(
        vulkan_instance,
        vulkan_device,
        app_data,
        size,
        vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    // data.vertex_buffer = vertex_buffer;
    // data.vertex_buffer_memory = vertex_buffer_memory;

    copy_buffer(vulkan_device, app_data, staging_buffer, vertex_buffer, size)?;

    vulkan_device.destroy_buffer(staging_buffer, None);
    vulkan_device.free_memory(staging_buffer_memory, None);

    Ok((vertex_buffer, vertex_buffer_memory))
}

pub unsafe fn create_index_buffer(vulkan_instance: &Instance, vulkan_device: &Device, app_data: &AppData, cpu_render_object: &CpuRenderObject) -> Result<(vk::Buffer, vk::DeviceMemory)> {
    let size = (size_of::<u32>() * cpu_render_object.indices.len()) as u64;

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

    memcpy(cpu_render_object.indices.as_ptr(), memory.cast(), cpu_render_object.indices.len());

    vulkan_device.unmap_memory(staging_buffer_memory);

    let (index_buffer, index_buffer_memory) = create_buffer(
        vulkan_instance,
        vulkan_device,
        app_data,
        size,
        vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    // data.index_buffer = index_buffer;
    // data.index_buffer_memory = index_buffer_memory;

    copy_buffer(vulkan_device, app_data, staging_buffer, index_buffer, size)?;

    vulkan_device.destroy_buffer(staging_buffer, None);
    vulkan_device.free_memory(staging_buffer_memory, None);

    Ok((index_buffer, index_buffer_memory))
}