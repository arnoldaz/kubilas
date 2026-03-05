
use cgmath::{Vector2, Vector3, Vector4, vec2, vec3, vec4};
use vulkanalia::prelude::v1_0::*;

use std::mem::{offset_of, size_of};
type Vec2 = cgmath::Vector2<f32>;
type Vec3 = cgmath::Vector3<f32>;

use std::hash::{Hash, Hasher};

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vertex {
    pub position: Vector3<f32>,
    pub color: Vector3<f32>,
    pub texture_coordinates: Vector2<f32>,
}

impl Eq for Vertex {}

impl Hash for Vertex {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.position[0].to_bits().hash(state);
        self.position[1].to_bits().hash(state);
        self.position[2].to_bits().hash(state);
        self.color[0].to_bits().hash(state);
        self.color[1].to_bits().hash(state);
        self.color[2].to_bits().hash(state);
        self.texture_coordinates[0].to_bits().hash(state);
        self.texture_coordinates[1].to_bits().hash(state);
    }
}

impl From<egui::epaint::Vertex> for Vertex {
    fn from(value: egui::epaint::Vertex) -> Self {
        let color = value.color.to_normalized_gamma_f32();
        Self {
            position: vec3(value.pos.x, value.pos.y, 0.0),
            color: vec3(color[0], color[1], color[2]),
            texture_coordinates: vec2(value.uv.x, value.uv.y),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct UiVertex {
    pub position: Vector2<f32>,
    pub texture_coordinates: Vector2<f32>,
    pub color: Vector4<u8>,
}

impl From<egui::epaint::Vertex> for UiVertex {
    fn from(value: egui::epaint::Vertex) -> Self {
        Self {
            position: vec2(value.pos.x, value.pos.y),
            texture_coordinates: vec2(value.uv.x, value.uv.y),
            color: vec4(value.color[0], value.color[1], value.color[2], value.color[3]),
        }
    }
}

impl UiVertex {
    pub const fn _new(position: Vector2<f32>, texture_coordinates: Vector2<f32>, color: Vector4<u8>) -> Self {
        Self { position, texture_coordinates, color }
    }

    pub fn binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(size_of::<UiVertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build()
    }

    pub fn attribute_descriptions() -> [vk::VertexInputAttributeDescription; 3] {
        let pos = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(0)
            .format(vk::Format::R32G32_SFLOAT)
            .offset(offset_of!(UiVertex, position) as u32)
            .build();

        let color = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(1)
            .format(vk::Format::R32G32_SFLOAT)
            .offset(offset_of!(UiVertex, texture_coordinates) as u32)
            .build();

        let tex_coord = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(2)
            .format(vk::Format::R8G8B8A8_UNORM)
            .offset(offset_of!(UiVertex, color) as u32)
            .build();

        [pos, color, tex_coord]
    }
}

impl Vertex {
    pub const fn new(pos: Vec3, color: Vec3, tex_coord: Vec2) -> Self {
        Self { position: pos, color, texture_coordinates: tex_coord }
    }

    pub fn binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(size_of::<Vertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build()
    }

    pub fn attribute_descriptions() -> [vk::VertexInputAttributeDescription; 3] {
        let pos = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(0)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(offset_of!(Vertex, position) as u32)
            .build();

        let color = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(1)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(offset_of!(Vertex, color) as u32)
            .build();

        let tex_coord = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(2)
            .format(vk::Format::R32G32_SFLOAT)
            .offset(offset_of!(Vertex, texture_coordinates) as u32)
            .build();

        [pos, color, tex_coord]
    }
}
