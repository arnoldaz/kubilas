use std::time::Instant;

use cgmath::{Matrix4, Quaternion, Vector3};
use vulkanalia::vk::{self, DeviceV1_0, DeviceV1_3, Handle, HasBuilder};
use anyhow::{Result};
use crate::{depth::DepthResources, pipeline::PipelineData, registry::{MeshRegistry, TextureRegistry}, scene::GpuEntity, swapchain::SwapchainData, vulkan_context::VulkanContext};

pub struct CommandData {
    pub command_pool: vk::CommandPool,
    pub command_buffers: Vec<vk::CommandBuffer>,

    pub start: Instant,
}

impl CommandData {
    pub unsafe fn new(vulkan_context: &VulkanContext, swapchain_data: &SwapchainData) -> Result<Self> {
        let command_pool = Self::create_command_pool(vulkan_context)?;
        let command_buffers = Self::create_command_buffers(command_pool, vulkan_context, swapchain_data)?;

        Ok(Self { command_pool, command_buffers, start: Instant::now() })
    }

    pub unsafe fn destroy(self, vulkan_context: &VulkanContext) {
        vulkan_context.device.free_command_buffers(self.command_pool, &self.command_buffers);
        vulkan_context.device.destroy_command_pool(self.command_pool, None);
    }

    unsafe fn create_command_pool(vulkan_context: &VulkanContext) -> Result<vk::CommandPool> {
        let (graphics, _) = vulkan_context.queue_family_indices()?;

        let info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(graphics);

        let command_pool = vulkan_context.device.create_command_pool(&info, None)?;

        Ok(command_pool)
    }

    unsafe fn create_command_buffers(command_pool: vk::CommandPool, vulkan_context: &VulkanContext, swapchain_data: &SwapchainData) -> Result<Vec<vk::CommandBuffer>> {
        let allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(swapchain_data.swapchain_image_views.len() as u32);

        let command_buffers = vulkan_context.device.allocate_command_buffers(&allocate_info)?;

        Ok(command_buffers)
    }

    pub unsafe fn begin_single_time_commands(&self, vulkan_context: &VulkanContext) -> Result<vk::CommandBuffer> {
        let info = vk::CommandBufferAllocateInfo::builder()
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_pool(self.command_pool)
            .command_buffer_count(1);

        let command_buffer = vulkan_context.device.allocate_command_buffers(&info)?[0];

        let info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        vulkan_context.device.begin_command_buffer(command_buffer, &info)?;

        Ok(command_buffer)
    }

    pub unsafe fn end_single_time_commands(&self, command_buffer: vk::CommandBuffer, vulkan_context: &VulkanContext) -> Result<()> {
        vulkan_context.device.end_command_buffer(command_buffer)?;

        let command_buffers = &[command_buffer];
        let info = vk::SubmitInfo::builder()
            .command_buffers(command_buffers);

        vulkan_context.device.queue_submit(vulkan_context.graphics_queue, &[info], vk::Fence::null())?;
        vulkan_context.device.queue_wait_idle(vulkan_context.graphics_queue)?;

        vulkan_context.device.free_command_buffers(self.command_pool, &[command_buffer]);

        Ok(())
    }

    pub unsafe fn update_command_buffer(&mut self, image_index: usize, vulkan_context: &VulkanContext, swapchain_data: &SwapchainData, depth_resources: &DepthResources, pipeline_data: &PipelineData, entities: &Vec<GpuEntity>,
        mesh_registry: &MeshRegistry, texture_registry: &TextureRegistry) -> Result<()> {
        let command_buffer = self.command_buffers[image_index];

        vulkan_context.device.reset_command_buffer(
            command_buffer,
            vk::CommandBufferResetFlags::empty(),
        )?;

        let time = self.start.elapsed().as_secs_f32();


        let info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        vulkan_context.device.begin_command_buffer(command_buffer, &info)?;

        let render_area = vk::Rect2D::builder()
            .offset(vk::Offset2D::default())
            .extent(swapchain_data.swapchain_extent);

        let color_clear_value = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        };

        let depth_clear_value = vk::ClearValue {
            depth_stencil: vk::ClearDepthStencilValue {
                depth: 1.0,
                stencil: 0,
            },
        };

        let color_attachment = vk::RenderingAttachmentInfo::builder()
            .image_view(swapchain_data.swapchain_image_views[image_index])
            .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .clear_value(color_clear_value);
        let color_attachments = [color_attachment];

        let depth_attachment = vk::RenderingAttachmentInfo::builder()
            .image_view(depth_resources.depth_image_view)
            .image_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .clear_value(depth_clear_value);

        let rendering_info = vk::RenderingInfo::builder()
            .render_area(render_area)
            .layer_count(1)
            .color_attachments(&color_attachments)
            .depth_attachment(&depth_attachment);

        let color_range = vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        };

        let barrier = vk::ImageMemoryBarrier2::builder()
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .src_access_mask(vk::AccessFlags2::empty())
            .dst_access_mask(vk::AccessFlags2::COLOR_ATTACHMENT_READ | vk::AccessFlags2::COLOR_ATTACHMENT_WRITE)
            .image(swapchain_data.swapchain_images[image_index])
            .subresource_range(color_range)
            .src_stage_mask(vk::PipelineStageFlags2::TOP_OF_PIPE)
            .dst_stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT);

        let binding = [barrier];
        let dependency_info = vk::DependencyInfo::builder()
            .image_memory_barriers(&binding);

        vulkan_context.device.cmd_pipeline_barrier2(
            command_buffer,
            &dependency_info
        );

        vulkan_context.device.cmd_begin_rendering(command_buffer, &rendering_info);
        vulkan_context.device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline_data.pipeline);

        let viewport = vk::Viewport::builder()
            .x(0.0)
            .y(0.0)
            .width(swapchain_data.swapchain_extent.width as f32)
            .height(swapchain_data.swapchain_extent.height as f32)
            .min_depth(0.0)
            .max_depth(1.0);

        let scissor = vk::Rect2D::builder()
            .offset(vk::Offset2D { x: 0, y: 0 })
            .extent(swapchain_data.swapchain_extent);

        vulkan_context.device.cmd_set_viewport(command_buffer, 0, &[viewport]);
        vulkan_context.device.cmd_set_scissor(command_buffer, 0, &[scissor]);

        vulkan_context.device.cmd_bind_descriptor_sets(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            pipeline_data.pipeline_layout,
            0,
            &[pipeline_data.descriptor_set],
            &[],
        );

        for gpu_entity in entities {
            let mesh = mesh_registry.get(gpu_entity.mesh_id);
            let texture = texture_registry.get(gpu_entity.texture_id);

            // TODO: have only 1 buffer for both and use offset instead, nvidia dev guide says it's very bad now
            vulkan_context.device.cmd_bind_vertex_buffers(command_buffer, 0, &[mesh.vertex_buffer.buffer], &[0]);
            vulkan_context.device.cmd_bind_index_buffer(command_buffer, mesh.index_buffer.buffer, 0, vk::IndexType::UINT32);

            let mut new_rotation = gpu_entity.transform.rotation.clone();
            new_rotation.y *= time;
            let new_quaternion = Quaternion::from(new_rotation);
            let model = Self::trs_matrix(gpu_entity.transform.translation, new_quaternion, gpu_entity.transform.scale);
            let model_bytes = std::slice::from_raw_parts(
                &model as *const Matrix4<f32> as *const u8,
                size_of::<Matrix4<f32>>()
            );

            vulkan_context.device.cmd_push_constants(
                command_buffer,
                pipeline_data.pipeline_layout,
                vk::ShaderStageFlags::VERTEX,
                0,
                model_bytes,
            );

            let obj_index_bytes = std::slice::from_raw_parts(&(gpu_entity.texture_id.0 as u32) as *const u32 as *const u8, 4);
            vulkan_context.device.cmd_push_constants(
                command_buffer,
                pipeline_data.pipeline_layout,
                vk::ShaderStageFlags::FRAGMENT,
                64, // offset vertex push constants
                obj_index_bytes,
            );

            vulkan_context.device.cmd_draw_indexed(command_buffer, mesh.index_count, 1, 0, 0, 0);
        }

        vulkan_context.device.cmd_end_rendering(command_buffer);

        // TODO: abstract this bullshit
        let barrier = vk::ImageMemoryBarrier2::builder()
            .old_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .src_access_mask(vk::AccessFlags2::COLOR_ATTACHMENT_WRITE)
            .dst_access_mask(vk::AccessFlags2::empty())
            .image(swapchain_data.swapchain_images[image_index])
            .subresource_range(color_range)
            .src_stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)
            .dst_stage_mask(vk::PipelineStageFlags2::BOTTOM_OF_PIPE);

        let binding = [barrier];
        let dependency_info = vk::DependencyInfo::builder()
            .image_memory_barriers(&binding);

        vulkan_context.device.cmd_pipeline_barrier2(
            command_buffer,
            &dependency_info
        );

        vulkan_context.device.end_command_buffer(command_buffer)?;

        Ok(())
    }

    pub fn trs_matrix(
        translation: Vector3<f32>,
        rotation: Quaternion<f32>,
        scale: Vector3<f32>,
    ) -> Matrix4<f32> {
        let t = Matrix4::from_translation(translation);
        let r = Matrix4::from(rotation);
        let s = Matrix4::from_nonuniform_scale(scale.x, scale.y, scale.z);

        t * r * s
    }

}