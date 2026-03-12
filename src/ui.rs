use vulkanalia::prelude::v1_0::*;
use std::collections::HashMap;
use std::sync::Arc;
use egui::Context;
use winit::{event::WindowEvent, window::Window};
use anyhow::{anyhow, Result};

use crate::bitmap::Bitmap;
use crate::buffer::BufferAllocation;
use crate::command::CommandData;
use crate::pipeline::PipelineData;
use crate::registry::{Destroy, TextureId, TextureRegistry};
use crate::texture::Texture;
use crate::vertex::UiVertex;
use crate::vulkan::insert_image_memory_barrier;
use crate::vulkan_context::{VulkanContext};

pub struct Ui {
    egui_context: egui::Context,
    egui_state: egui_winit::State,

    texture_id_map: HashMap<egui::TextureId, TextureId>,
    
    clipped_primitives: Vec<egui::ClippedPrimitive>,
    textures_delta: egui::TexturesDelta,

    window: Arc<Window>,
}

impl Ui {
    pub fn new(window: Arc<Window>) -> Self {
        let egui_context = egui::Context::default();
        let egui_state = egui_winit::State::new(egui_context.clone(), egui::ViewportId::ROOT, &window, None, None, None);

        Self {
            egui_context,
            egui_state,
            texture_id_map: HashMap::new(),
            clipped_primitives: Vec::new(),
            textures_delta: egui::TexturesDelta::default(),
            window
        }
    }

    pub fn run_frame(&mut self, ui_creation_callback: impl FnMut(&Context)) {
        let raw_input = self.egui_state.take_egui_input(&self.window);
        let full_output = self.egui_context.run(raw_input, ui_creation_callback);

        self.egui_state.handle_platform_output(&self.window, full_output.platform_output);
        let clipped_primitives = self.egui_context.tessellate(full_output.shapes, full_output.pixels_per_point);

        self.clipped_primitives = clipped_primitives;
        self.textures_delta = full_output.textures_delta;
    }

    pub fn is_consumed(&mut self, window_event: &WindowEvent) -> bool {
        self.egui_state.on_window_event(&self.window, window_event).consumed
    }

    pub unsafe fn update_textures(&mut self, vulkan_context: &VulkanContext, command_data: &CommandData, pipeline_data: &PipelineData, texture_registry: &mut TextureRegistry, ui_sampler: vk::Sampler) -> Result<()> {
        let textures = std::mem::take(&mut self.textures_delta.set);
        for (texture_id, image_delta) in textures {
            match &image_delta.image {
                egui::epaint::image::ImageData::Color(image) => {
                    let [width, height] = image.size.map(|x| x as u32);
                    let bytes = image.pixels
                        .clone()
                        .into_iter()
                        .flat_map(|c| [c.r(), c.g(), c.b(), c.a()])
                        .collect::<Vec<u8>>();

                    let bitmap = Bitmap::new(bytes, width, height);
                    let texture = Texture::create_from_bitmap(&bitmap, vulkan_context, command_data, ui_sampler)?;
                    
                    if let Some(pos) = image_delta.pos {
                        self.update_texture(&texture_id, texture, width, height, pos, vulkan_context, command_data, texture_registry)?;
                    } else {
                        self.upload_texture(texture_id, texture, vulkan_context, pipeline_data, texture_registry)?;
                    }
                }
            }
        }

        Ok(())
    }

    pub unsafe fn bind_and_draw(&mut self, screen_width: u32, screen_height: u32, vulkan_context: &VulkanContext, command_data: &CommandData, pipeline_data: &PipelineData, command_buffer: vk::CommandBuffer) -> Result<Vec<BufferAllocation>> {
        vulkan_context.device.cmd_bind_pipeline(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            pipeline_data.ui_pipeline,
        );

        vulkan_context.device.cmd_bind_descriptor_sets(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            pipeline_data.ui_pipeline_layout,
            0,
            &[pipeline_data.descriptor_set],
            &[],
        );

        let mut buffers = Vec::<BufferAllocation>::new();
        
        let clipped_primitives = std::mem::take(&mut self.clipped_primitives);
        for egui::ClippedPrimitive { clip_rect, primitive } in clipped_primitives {
            let mesh = match primitive {
                egui::epaint::Primitive::Mesh(mesh) => mesh,
                egui::epaint::Primitive::Callback(_) => {
                    return Err(anyhow!("`Primitive::Callback(_)` primitive not implemented"));
                }
            };
            let (vertices, indices) = (
                mesh.vertices
                    .into_iter()
                    .map(Into::into)
                    .collect::<Vec<UiVertex>>(),
                mesh.indices,
            );
            if vertices.is_empty() || indices.is_empty() {
                continue;
            }

            let texture_id = self.texture_id_map.get(&mesh.texture_id).unwrap();

            // TODO: have 1 permanent buffer and place it there to avoid allocating new buffers
            let vertex_buffer = BufferAllocation::allocate_buffer(&vertices, vk::BufferUsageFlags::VERTEX_BUFFER, vulkan_context, command_data)?;
            let index_buffer = BufferAllocation::allocate_buffer(&indices, vk::BufferUsageFlags::INDEX_BUFFER, vulkan_context, command_data)?;

            vulkan_context.device.cmd_bind_vertex_buffers(command_buffer, 0, &[vertex_buffer.buffer], &[0]);
            vulkan_context.device.cmd_bind_index_buffer(command_buffer, index_buffer.buffer, 0, vk::IndexType::UINT32);

            let id = texture_id.0 as u32;
            vulkan_context.device.cmd_push_constants(
                command_buffer,
                pipeline_data.ui_pipeline_layout,
                vk::ShaderStageFlags::FRAGMENT,
                64,
                &id.to_ne_bytes(),
            );

            let (width, height) = (screen_width, screen_height);

            let min = clip_rect.min;
            let min = egui::Pos2 {
                x: min.x * 1.0 as f32,
                y: min.y * 1.0 as f32,
            };
            let min = egui::Pos2 {
                x: f32::clamp(min.x, 0.0, width as f32),
                y: f32::clamp(min.y, 0.0, height as f32),
            };
            let max = clip_rect.max;
            let max = egui::Pos2 {
                x: max.x * 1.0 as f32,
                y: max.y * 1.0 as f32,
            };
            let max = egui::Pos2 {
                x: f32::clamp(max.x, min.x, width as f32),
                y: f32::clamp(max.y, min.y, height as f32),
            };

            let scissor = vk::Rect2D::builder()
                .offset(vk::Offset2D {
                    x: min.x.round() as i32,
                    y: min.y.round() as i32,
                })
                .extent(vk::Extent2D {
                    width: (max.x.round() - min.x) as u32,
                    height: (max.y.round() - min.y) as u32,
                });

            let viewport = vk::Viewport::builder()
                .x(0.0)
                .y(0.0)
                .width(width as f32)
                .height(height as f32)
                .min_depth(0.0)
                .max_depth(1.0);

            vulkan_context.device.cmd_set_scissor(command_buffer, 0, &[scissor]);
            vulkan_context.device.cmd_set_viewport(command_buffer, 0, &[viewport]);

            vulkan_context.device.cmd_draw_indexed(command_buffer, indices.len() as u32, 1, 0, 0, 0);

            buffers.push(vertex_buffer);
            buffers.push(index_buffer);
        }

        Ok(buffers)
    }

    unsafe fn update_texture(&mut self, egui_texture_id: &egui::TextureId, new_texture: Texture, texture_width: u32, texture_height: u32, subregion_position: [usize; 2],
        vulkan_context: &VulkanContext, command_data: &CommandData, texture_registry: &mut TextureRegistry) -> Result<()> {

        let existing_texture_id = self.texture_id_map.get(egui_texture_id).ok_or_else(|| anyhow::anyhow!("Texture not found"))?;
        let existing_texture = texture_registry.get(*existing_texture_id);

        let command_buffer = command_data.begin_single_time_commands(vulkan_context)?;

        insert_image_memory_barrier(
            &vulkan_context.device,
            command_buffer,
            existing_texture.image,
            vk::QUEUE_FAMILY_IGNORED,
            vk::QUEUE_FAMILY_IGNORED,
            vk::AccessFlags::SHADER_READ,
            vk::AccessFlags::TRANSFER_WRITE,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
            vk::PipelineStageFlags::TRANSFER,
            vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
        );

        insert_image_memory_barrier(
            &vulkan_context.device,
            command_buffer,
            new_texture.image,
            vk::QUEUE_FAMILY_IGNORED,
            vk::QUEUE_FAMILY_IGNORED,
            vk::AccessFlags::SHADER_READ,
            vk::AccessFlags::TRANSFER_READ,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
            vk::PipelineStageFlags::TRANSFER,
            vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
        );

        let top_left = vk::Offset3D::builder()
            .x(subregion_position[0] as i32)
            .y(subregion_position[1] as i32)
            .z(0)
            .build();

        let bottom_right = vk::Offset3D::builder()
            .x(subregion_position[0] as i32 + texture_width as i32)
            .y(subregion_position[1] as i32 + texture_height as i32)
            .z(1)
            .build();

        let src_subresource = vk::ImageSubresourceLayers::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .mip_level(0)
            .base_array_layer(0)
            .layer_count(1);

        let dst_subresource = vk::ImageSubresourceLayers::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .mip_level(0)
            .base_array_layer(0)
            .layer_count(1);

        let src_offsets = [
            vk::Offset3D::builder()
                .x(0)
                .y(0)
                .z(0)
                .build(),
            vk::Offset3D::builder()
                .x(texture_width as i32)
                .y(texture_height as i32)
                .z(1)
                .build(),
        ];

        let dst_offsets = [top_left, bottom_right];

        let region = vk::ImageBlit::builder()
            .src_subresource(src_subresource)
            .src_offsets(src_offsets)
            .dst_subresource(dst_subresource)
            .dst_offsets(dst_offsets)
            .build();

        vulkan_context.device.cmd_blit_image(
            command_buffer,
            new_texture.image,
            vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            existing_texture.image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            &[region],
            vk::Filter::NEAREST,
        );

        insert_image_memory_barrier(
            &vulkan_context.device,
            command_buffer,
            existing_texture.image,
            vk::QUEUE_FAMILY_IGNORED,
            vk::QUEUE_FAMILY_IGNORED,
            vk::AccessFlags::TRANSFER_WRITE,
            vk::AccessFlags::SHADER_READ,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
            vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
        );

        command_data.end_single_time_commands(command_buffer, vulkan_context)?;

        new_texture.destroy(vulkan_context);

        Ok(())
    }

    unsafe fn upload_texture(&mut self, egui_texture_id: egui::TextureId, new_texture: Texture,
         vulkan_context: &VulkanContext, pipeline_data: &PipelineData, texture_registry: &mut TextureRegistry) -> Result<()> {

        let new_texture_id = texture_registry.add(new_texture);
        self.texture_id_map.insert(egui_texture_id, new_texture_id);

        let new_texture_readonly = texture_registry.get(new_texture_id);
        let image_info = vk::DescriptorImageInfo::builder()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(new_texture_readonly.image_view)
            .sampler(new_texture_readonly.sampler);

        let image_infos = &[image_info];
        let sampler_write = vk::WriteDescriptorSet::builder()
            .dst_set(pipeline_data.descriptor_set)
            .dst_binding(1)
            .dst_array_element(new_texture_id.0 as u32)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(image_infos);

        vulkan_context.device.update_descriptor_sets(&[sampler_write], &[] as &[vk::CopyDescriptorSet]);

        Ok(())
    }
}
