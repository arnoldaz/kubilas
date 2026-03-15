use anyhow::{anyhow, Result};
use cgmath::Vector2;
use vulkanalia::prelude::v1_0::*;
use std::mem::size_of;
type Mat4 = cgmath::Matrix4<f32>;
use crate::command::CommandData;
use crate::pipeline::{PipelineData};
use crate::swapchain::SwapchainData;
use crate::vulkan_context::{VulkanContext};

pub const MAX_FRAMES_IN_FLIGHT: usize = 2;

// pub image_available_semaphores: Vec<vk::Semaphore>,
// pub render_finished_semaphores: Vec<vk::Semaphore>,
// pub in_flight_fences: Vec<vk::Fence>,
// pub images_in_flight: Vec<vk::Fence>,
pub unsafe fn create_sync_objects(vulkan_context: &VulkanContext, swapchain_data: &SwapchainData) -> Result<(Vec<vk::Semaphore>, Vec<vk::Semaphore>, Vec<vk::Fence>, Vec<vk::Fence>)> {
    let semaphore_info = vk::SemaphoreCreateInfo::builder();
    let fence_info = vk::FenceCreateInfo::builder()
        .flags(vk::FenceCreateFlags::SIGNALED);

    let mut image_available_semaphores = Vec::new();
    let mut in_flight_fences = Vec::new();
    for _ in 0..MAX_FRAMES_IN_FLIGHT {
        image_available_semaphores.push(vulkan_context.device.create_semaphore(&semaphore_info, None)?);
        in_flight_fences.push(vulkan_context.device.create_fence(&fence_info, None)?);
    }
    
    let image_count = swapchain_data.swapchain_images.len();
    
    let mut render_finished_semaphores = Vec::new();
    for _ in 0..image_count {
        render_finished_semaphores.push(vulkan_context.device.create_semaphore(&semaphore_info, None)?);
    }

    let images_in_flight: Vec<vk::Fence> = swapchain_data.swapchain_images
        .iter()
        .map(|_| vk::Fence::null())
        .collect();

    Ok((image_available_semaphores, render_finished_semaphores, in_flight_fences, images_in_flight))
}

pub unsafe fn create_buffer(vulkan_context: &VulkanContext, size: vk::DeviceSize, usage: vk::BufferUsageFlags, properties: vk::MemoryPropertyFlags) -> Result<(vk::Buffer, vk::DeviceMemory)> {
    let buffer_info = vk::BufferCreateInfo::builder()
        .size(size)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let buffer = vulkan_context.device.create_buffer(&buffer_info, None)?;

    let requirements = vulkan_context.device.get_buffer_memory_requirements(buffer);

    let memory_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(requirements.size)
        .memory_type_index(get_memory_type_index(vulkan_context, properties, requirements)?);

    let buffer_memory = vulkan_context.device.allocate_memory(&memory_info, None)?;

    vulkan_context.device.bind_buffer_memory(buffer, buffer_memory, 0)?;

    Ok((buffer, buffer_memory))
}

pub unsafe fn copy_buffer(vulkan_context: &VulkanContext, command_data: &CommandData, source: vk::Buffer, destination: vk::Buffer, size: vk::DeviceSize) -> Result<()> {
    let command_buffer = command_data.begin_single_time_commands(vulkan_context)?;

    let regions = vk::BufferCopy::builder().size(size);
    vulkan_context.device.cmd_copy_buffer(command_buffer, source, destination, &[regions]);

    command_data.end_single_time_commands(command_buffer, vulkan_context)?;

    Ok(())
}

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct UniformBufferObject {
    pub view: Mat4,
    pub proj: Mat4,
    pub screen_size: Vector2<u32>,
}

pub unsafe fn create_uniform_buffers(vulkan_context: &VulkanContext, pipeline_data: &PipelineData) -> Result<(Vec<vk::Buffer>, Vec<vk::DeviceMemory>)> {
    let mut uniform_buffers = Vec::<vk::Buffer>::new();
    let mut uniform_buffers_memory = Vec::<vk::DeviceMemory>::new();

    for i in 0..MAX_FRAMES_IN_FLIGHT {

        let (uniform_buffer, uniform_buffer_memory) = create_buffer(
            vulkan_context,
            size_of::<UniformBufferObject>() as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
        )?;

        uniform_buffers.push(uniform_buffer);
        uniform_buffers_memory.push(uniform_buffer_memory);

        let info = vk::DescriptorBufferInfo::builder()
            .buffer(uniform_buffers[i])
            .offset(0)
            .range(size_of::<UniformBufferObject>() as u64);

        // TODO: I don't even know how to fix this, it seems I have MAX_FRAMES_IN_FLIGHT amount of ubo objects but only use one in shader, need to somehow only bind buffer just to first one
        let buffer_info = &[info];
        let ubo_write = vk::WriteDescriptorSet::builder()
            .dst_set(pipeline_data.descriptor_set)
            .dst_binding(0)
            .dst_array_element(i as u32)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(buffer_info);

        vulkan_context.device.update_descriptor_sets(&[ubo_write], &[] as &[vk::CopyDescriptorSet]);
    }

    Ok((uniform_buffers, uniform_buffers_memory))
}

pub unsafe fn create_image_view(vulkan_context: &VulkanContext, image: vk::Image, format: vk::Format, aspects: vk::ImageAspectFlags) -> Result<vk::ImageView> {
    let subresource_range = vk::ImageSubresourceRange::builder()
        .aspect_mask(aspects)
        .base_mip_level(0)
        .level_count(1)
        .base_array_layer(0)
        .layer_count(1);

    let info = vk::ImageViewCreateInfo::builder()
        .image(image)
        .view_type(vk::ImageViewType::_2D)
        .format(format)
        .subresource_range(subresource_range);

    Ok(vulkan_context.device.create_image_view(&info, None)?)
}

pub unsafe fn create_image(vulkan_context: &VulkanContext, width: u32, height: u32, format: vk::Format, tiling: vk::ImageTiling, usage: vk::ImageUsageFlags, properties: vk::MemoryPropertyFlags) -> Result<(vk::Image, vk::DeviceMemory)> {
    let info = vk::ImageCreateInfo::builder()
        .image_type(vk::ImageType::_2D)
        .extent(vk::Extent3D {
            width,
            height,
            depth: 1,
        })
        .mip_levels(1)
        .array_layers(1)
        .format(format)
        .tiling(tiling)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .usage(usage)
        .samples(vk::SampleCountFlags::_1)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let image = vulkan_context.device.create_image(&info, None)?;

    let requirements = vulkan_context.device.get_image_memory_requirements(image);
    let info = vk::MemoryAllocateInfo::builder()
        .allocation_size(requirements.size)
        .memory_type_index(get_memory_type_index(vulkan_context, properties, requirements)?);

    let image_memory = vulkan_context.device.allocate_memory(&info, None)?;

    vulkan_context.device.bind_image_memory(image, image_memory, 0)?;

    Ok((image, image_memory))
}

pub unsafe fn get_memory_type_index(vulkan_context: &VulkanContext, properties: vk::MemoryPropertyFlags, requirements: vk::MemoryRequirements) -> Result<u32> {
    let memory = vulkan_context.instance.get_physical_device_memory_properties(vulkan_context.physical_device);

    (0..memory.memory_type_count)
        .find(|i| {
            let suitable = (requirements.memory_type_bits & (1 << i)) != 0;
            let memory_type = memory.memory_types[*i as usize];
            suitable && memory_type.property_flags.contains(properties)
        })
        .ok_or_else(|| anyhow!("Failed to find suitable memory type."))
}

pub unsafe fn insert_image_memory_barrier(
    device: &vulkanalia::Device,
    cmd: vk::CommandBuffer,
    image: vk::Image,
    src_q_family_index: u32,
    dst_q_family_index: u32,
    src_access_mask: vk::AccessFlags,
    dst_access_mask: vk::AccessFlags,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
    src_stage_mask: vk::PipelineStageFlags,
    dst_stage_mask: vk::PipelineStageFlags,
    subresource_range: vk::ImageSubresourceRange,
) {
    let barrier = vk::ImageMemoryBarrier::builder()
        .src_queue_family_index(src_q_family_index)
        .dst_queue_family_index(dst_q_family_index)
        .src_access_mask(src_access_mask)
        .dst_access_mask(dst_access_mask)
        .old_layout(old_layout)
        .new_layout(new_layout)
        .image(image)
        .subresource_range(subresource_range)
        .build();

    device.cmd_pipeline_barrier(
        cmd,
        src_stage_mask,
        dst_stage_mask,
        vk::DependencyFlags::BY_REGION,
        &[] as &[vk::MemoryBarrier],
        &[] as &[vk::BufferMemoryBarrier],
        &[barrier],
    );
}

pub unsafe fn transition_image_layout(vulkan_context: &VulkanContext, command_data: &CommandData, image: vk::Image, format: vk::Format, old_layout: vk::ImageLayout, new_layout: vk::ImageLayout) -> Result<()> {
    let (
        src_access_mask,
        dst_access_mask,
        src_stage_mask,
        dst_stage_mask,
    ) = match (old_layout, new_layout) {
        (vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL) => (
            vk::AccessFlags::empty(),
            vk::AccessFlags::TRANSFER_WRITE,
            vk::PipelineStageFlags::TOP_OF_PIPE,
            vk::PipelineStageFlags::TRANSFER,
        ),
        (vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL) => (
            vk::AccessFlags::TRANSFER_WRITE,
            vk::AccessFlags::SHADER_READ,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
        ),
        (vk::ImageLayout::UNDEFINED, vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL) => (
            vk::AccessFlags::empty(),
            vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            vk::PipelineStageFlags::TOP_OF_PIPE,
            vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
        ),
        (vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL, vk::ImageLayout::TRANSFER_SRC_OPTIMAL) => (
            vk::AccessFlags::SHADER_READ,
            vk::AccessFlags::TRANSFER_READ,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
            vk::PipelineStageFlags::TRANSFER,
        ),
        (vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL, vk::ImageLayout::TRANSFER_DST_OPTIMAL) => (
            vk::AccessFlags::SHADER_READ,
            vk::AccessFlags::TRANSFER_WRITE,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
            vk::PipelineStageFlags::TRANSFER,
        ),
        _ => return Err(anyhow!("Unsupported image layout transition!")),
    };


    let command_buffer = command_data.begin_single_time_commands(vulkan_context)?;

    let aspect_mask = if new_layout == vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL {
        match format {
            vk::Format::D32_SFLOAT_S8_UINT | vk::Format::D24_UNORM_S8_UINT =>
                vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL,
            _ => vk::ImageAspectFlags::DEPTH
        }
    } else {
        vk::ImageAspectFlags::COLOR
    };

    let subresource = vk::ImageSubresourceRange::builder()
        .aspect_mask(aspect_mask)
        .base_mip_level(0)
        .level_count(1)
        .base_array_layer(0)
        .layer_count(1);

    let barrier = vk::ImageMemoryBarrier::builder()
        .old_layout(old_layout)
        .new_layout(new_layout)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .image(image)
        .subresource_range(subresource)
        .src_access_mask(src_access_mask)
        .dst_access_mask(dst_access_mask);
    
    vulkan_context.device.cmd_pipeline_barrier(
        command_buffer,
        src_stage_mask,
        dst_stage_mask,
        vk::DependencyFlags::empty(),
        &[] as &[vk::MemoryBarrier],
        &[] as &[vk::BufferMemoryBarrier],
        &[barrier],
    );
    

    command_data.end_single_time_commands(command_buffer, vulkan_context)?;

    Ok(())
}

pub unsafe fn copy_buffer_to_image(vulkan_context: &VulkanContext, command_data: &CommandData, buffer: vk::Buffer, image: vk::Image, width: u32, height: u32) -> Result<()> {
    let command_buffer = command_data.begin_single_time_commands(vulkan_context)?;

    let subresource = vk::ImageSubresourceLayers::builder()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .mip_level(0)
        .base_array_layer(0)
        .layer_count(1);

    let region = vk::BufferImageCopy::builder()
        .buffer_offset(0)
        .buffer_row_length(0)
        .buffer_image_height(0)
        .image_subresource(subresource)
        .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
        .image_extent(vk::Extent3D { width, height, depth: 1 });
    
    vulkan_context.device.cmd_copy_buffer_to_image(
        command_buffer,
        buffer,
        image,
        vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        &[region],
    );
    
    command_data.end_single_time_commands(command_buffer, vulkan_context)?;

    Ok(())
}

pub unsafe fn create_texture_sampler(vulkan_context: &VulkanContext) -> Result<vk::Sampler> {
    let info = vk::SamplerCreateInfo::builder()
        .mag_filter(vk::Filter::LINEAR)
        .min_filter(vk::Filter::LINEAR)
        .address_mode_u(vk::SamplerAddressMode::REPEAT)
        .address_mode_v(vk::SamplerAddressMode::REPEAT)
        .address_mode_w(vk::SamplerAddressMode::REPEAT)
        .anisotropy_enable(true)
        .max_anisotropy(16.0)
        .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
        .unnormalized_coordinates(false)
        .compare_enable(false)
        .compare_op(vk::CompareOp::ALWAYS)
        .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
        .mip_lod_bias(0.0)
        .min_lod(0.0)
        .max_lod(0.0);
        
    let texture_sampler = vulkan_context.device.create_sampler(&info, None)?;

    Ok(texture_sampler)
}

pub unsafe fn create_texture_sampler_ui(vulkan_context: &VulkanContext) -> Result<vk::Sampler> {
    let info = vk::SamplerCreateInfo::builder()
        .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        .anisotropy_enable(false)
        .min_filter(vk::Filter::LINEAR)
        .mag_filter(vk::Filter::LINEAR)
        .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
        .min_lod(0.0)
        .max_lod(vk::LOD_CLAMP_NONE);
        
    let texture_sampler = vulkan_context.device.create_sampler(&info, None)?;

    Ok(texture_sampler)
}


