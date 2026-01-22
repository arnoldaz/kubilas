use vulkanalia::{Device, vk};
use vulkanalia_vma::Allocator;

use crate::{app::AppData, buffer::BufferAllocation, mesh::Mesh};
use anyhow::Result;


pub struct GpuMesh {
    pub vertex_buffer: BufferAllocation,
    pub index_buffer: BufferAllocation,
    pub index_count: u32,
}

impl GpuMesh {
    pub fn create_from_mesh(mesh: &Mesh, allocator: &Allocator, device: &Device, data: &AppData) -> Result<Self> {
        let vertex_buffer = unsafe { BufferAllocation::allocate_buffer(allocator, &mesh.vertices, vk::BufferUsageFlags::VERTEX_BUFFER, device, data) }?;
        let index_buffer = unsafe { BufferAllocation::allocate_buffer(allocator, &mesh.indices, vk::BufferUsageFlags::INDEX_BUFFER, device, data) }?;
    
        Ok( Self { vertex_buffer, index_buffer, index_count: mesh.indices.len() as u32 } )
    }
}