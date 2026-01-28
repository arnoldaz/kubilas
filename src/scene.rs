use cgmath::{Euler, Rad, Vector3};
use crate::registry::{MeshId, MeshRegistry, TextureId, TextureRegistry};

// Maybe use it like that somehow
pub struct Scene {
    pub entities: Vec<GpuEntity>,
    pub texture_registry: TextureRegistry,
    pub mesh_registry: MeshRegistry,
}

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
