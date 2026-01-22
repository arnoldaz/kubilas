
// // IDK yet
// pub struct ScneeSomething {
//     pub camera: Camera,
//     pub projection: Projection,
//     pub camera_movement: CameraMovement,
// }

// // Shared
// pub struct Vertex {
//     pub position: Vector3<f32>,
//     pub color: Vector3<f32>,
//     pub texture_coordinates: Vector2<f32>,
// }

// // CPU
// pub struct Bitmap {
//     pub pixels: Vec<u8>,
//     pub width: u32,
//     pub height: u32,
// }
// pub struct Mesh {
//     pub vertices: Vec<Vertex>,
//     pub indices: Vec<u32>,
// }


// // GPU
// pub struct Texture {
//     image: vk::Image,
//     image_memory: vk::DeviceMemory,
//     image_view: vk::ImageView,
//     sampler: vk::Sampler,
// }
// pub struct BufferAllocation {
//     buffer: vk::Buffer,
//     allocation: vma::Allocation,
// }
// pub struct GpuMesh {
//     vertex_buffer: BufferAllocation,
//     index_buffer: BufferAllocation,
//     index_count: u32,
// }
// pub struct TextureRegistry {
//     textures: Vec<Texture>,
// }
// pub struct TextureId(pub u32);
// pub struct MeshRegistry {
//     meshes: Vec<GpuMesh>,
// }
// pub struct MeshId(pub u32);




// pub struct GpuRender {
//     texture_id: u32,

//     mesh_buffer: BufferAllocation,
// }

// // OLD
// pub struct GpuRenderObject {
//     pub texture_image: vk::Image,
//     pub texture_image_memory: vk::DeviceMemory,
//     pub texture_image_view: vk::ImageView,

//     pub indices_count: u32,

//     pub vertex_buffer: vk::Buffer,
//     pub vertex_allocation: vma::Allocation,
//     pub index_buffer: vk::Buffer,
//     pub index_allocation: vma::Allocation,

//     pub translation: Vector3<f32>,
//     pub rotation: Euler<Rad<f32>>,
//     pub scale: Vector3<f32>,

//     pub sampler_index: u32,
// }

// pub struct CpuRenderObject {
//     pub indices: Vec<u32>,
//     pub vertices: Vec<GpuVertex>,

//     pub pixels: Vec<u8>,
//     pub width: u32,
//     pub height: u32,

//     pub translation: Vector3<f32>,
//     pub rotation: Euler<Rad<f32>>,
//     pub scale: Vector3<f32>,
// }

use cgmath::{Euler, Rad, Vector3};

use crate::registry::{MeshId, TextureId};


pub struct Transform {
    pub translation: Vector3<f32>,
    pub rotation: Euler<Rad<f32>>,
    pub scale: Vector3<f32>,
}

pub struct GpuEntity {
    pub mesh_id: MeshId,
    pub texture_id: TextureId,
    
    pub transform: Transform
}

impl Transform {
    pub fn new(translation: Vector3<f32>, rotation: Euler<Rad<f32>>, scale: Vector3<f32>) -> Self {
        Self { translation, rotation, scale }
    }
}

impl GpuEntity {
    pub fn new(mesh_id: MeshId, texture_id: TextureId, transform: Transform) -> Self {
        Self { mesh_id, texture_id, transform }
    }
}