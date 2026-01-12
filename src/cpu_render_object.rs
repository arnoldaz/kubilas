
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

use log::*;
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
use crate::vertex::Vertex;
type Vec2 = cgmath::Vector2<f32>;
type Vec3 = cgmath::Vector3<f32>;

use std::collections::HashMap;
use std::hash::{Hash, Hasher};


// cpu render object
// gpu render object would have actual vulkan pointers and buffers
#[derive(Clone, Debug)]
pub struct CpuRenderObject {
    pub indices: Vec<u32>,
    pub vertices: Vec<Vertex>,
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub translation: Vector3<f32>,
    pub rotation: Quaternion<f32>,
    pub scale: Vector3<f32>,
}

impl CpuRenderObject {
    pub fn new(model_name: &str, image_name: &str, translation: Vector3<f32>, rotation: Quaternion<f32>, scale: Vector3<f32>) -> Result<Self> {
        let (indices, vertices) = load_model(model_name)?;
        let (pixels, width, height) = load_image(image_name)?;

        Ok(Self { indices, vertices, pixels, width, height, translation, rotation, scale })
    }
}

fn load_model(model_name: &str) -> Result<(Vec<u32>, Vec<Vertex>)> {
    let mut reader = BufReader::new(File::open(model_name)?);
    let (models, _) = tobj::load_obj_buf(
        &mut reader,
        &tobj::LoadOptions { triangulate: true, single_index: true, ..Default::default() },
        |_| Ok(Default::default()),
    )?;

    let mut unique_vertices = HashMap::new();

    let mut indices = Vec::<u32>::new();
    let mut vertices = Vec::<Vertex>::new();

    for model in &models {
        for index in &model.mesh.indices {
            let pos_offset = (3 * index) as usize;
            let tex_coord_offset = (2 * index) as usize;

            let vertex = Vertex {
                pos: vec3(
                    model.mesh.positions[pos_offset],
                    model.mesh.positions[pos_offset + 1],
                    model.mesh.positions[pos_offset + 2],
                ),
                color: vec3(1.0, 1.0, 1.0),
                tex_coord: vec2(
                    model.mesh.texcoords[tex_coord_offset],
                    1.0 - model.mesh.texcoords[tex_coord_offset + 1],
                ),
            };

            if let Some(index) = unique_vertices.get(&vertex) {
                indices.push(*index as u32);
            } else {
                let index = vertices.len();
                unique_vertices.insert(vertex, index);
                vertices.push(vertex);
                indices.push(index as u32);
            }
        }
    }

    println!("Loaded model '{model_name}' with {} vertices and {} indices", vertices.len(), indices.len());

    Ok((indices, vertices))
}

fn load_image(image_name: &str) -> Result<(Vec<u8>, u32, u32)> {
    let image = File::open(image_name)?;
    let decoder = png::Decoder::new(image);
    let mut reader = decoder.read_info()?;

    let mut pixels = vec![0; reader.info().raw_bytes()];
    reader.next_frame(&mut pixels)?;
    
    let (width, height) = reader.info().size();
    let color_type = reader.info().color_type;
    if color_type != png::ColorType::Rgba {
        panic!("Invalid texture image '{image_name}'");
    }

    println!("Loaded image '{image_name}' with {width} width and {height} height");

    Ok((pixels, width, height))
}