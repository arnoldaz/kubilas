use vulkanalia::{vk};

use crate::{buffer::BufferAllocation, command::CommandData, mesh::Mesh, vulkan_context::{VulkanContext}};
use anyhow::Result;


pub struct GpuMesh {
    pub vertex_buffer: BufferAllocation,
    pub index_buffer: BufferAllocation,
    pub index_count: u32,
}

impl GpuMesh {
    pub fn create_from_mesh(mesh: &Mesh, vulkan_context: &VulkanContext, command_data: &CommandData) -> Result<Self> {
        let vertex_buffer = unsafe { BufferAllocation::allocate_buffer(&mesh.vertices, vk::BufferUsageFlags::VERTEX_BUFFER, vulkan_context, command_data) }?;
        let index_buffer = unsafe { BufferAllocation::allocate_buffer(&mesh.indices, vk::BufferUsageFlags::INDEX_BUFFER, vulkan_context, command_data) }?;
    
        Ok( Self { vertex_buffer, index_buffer, index_count: mesh.indices.len() as u32 } )
    }
    
    pub unsafe fn destroy(self, vulkan_context: &VulkanContext) {
        self.vertex_buffer.destroy(vulkan_context);
        self.index_buffer.destroy(vulkan_context);
    }
}