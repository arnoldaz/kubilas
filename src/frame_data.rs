use vulkanalia::vk::{self, DeviceV1_0};
use crate::{pipeline::PipelineData, swapchain::SwapchainData, vulkan::{create_sync_objects, create_uniform_buffers}, vulkan_context::VulkanContext};
use anyhow::{Result};


// Temp struct until I think of something better
pub struct FrameData {
    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub render_finished_semaphores: Vec<vk::Semaphore>,
    pub in_flight_fences: Vec<vk::Fence>,
    pub images_in_flight: Vec<vk::Fence>,

    pub uniform_buffers: Vec<vk::Buffer>,
    pub uniform_buffers_memory: Vec<vk::DeviceMemory>,
}

impl FrameData {
    pub unsafe fn new(vulkan_context: &VulkanContext, swapchain_data: &SwapchainData, pipeline_data: &PipelineData) -> Result<Self> {
        let (image_available_semaphores, render_finished_semaphores, in_flight_fences, images_in_flight) = create_sync_objects(vulkan_context, swapchain_data)?;
        let (uniform_buffers, uniform_buffers_memory) = create_uniform_buffers(vulkan_context, pipeline_data)?;

        Ok(Self { image_available_semaphores, render_finished_semaphores, in_flight_fences, images_in_flight, uniform_buffers, uniform_buffers_memory })
    }

    pub unsafe fn destroy(self, vulkan_context: &VulkanContext) {
        self.image_available_semaphores
            .iter()
            .for_each(|s| vulkan_context.device.destroy_semaphore(*s, None));
        self.render_finished_semaphores
            .iter()
            .for_each(|s| vulkan_context.device.destroy_semaphore(*s, None));
        self.in_flight_fences
            .iter()
            .for_each(|f| vulkan_context.device.destroy_fence(*f, None));
        // self.images_in_flight
        //     .iter()
        //     .for_each(|f| vulkan_context.device.destroy_fence(*f, None));

        self.uniform_buffers
            .iter()
            .for_each(|b| vulkan_context.device.destroy_buffer(*b, None));
        self.uniform_buffers_memory
            .iter()
            .for_each(|m| vulkan_context.device.free_memory(*m, None));
    }
}

