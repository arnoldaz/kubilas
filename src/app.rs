use anyhow::{anyhow, Result};
use egui::Context;
use winit::event::WindowEvent;
use winit::window::Window;

use vulkanalia::prelude::v1_0::*;
use vulkanalia::vk::{KhrSwapchainExtensionDeviceCommands};

use std::mem::{self, size_of};
use std::sync::Arc;
use cgmath::{Deg, Euler, Rad, Vector3, point3, vec2};

use crate::bitmap::Bitmap;
use crate::camera::{Camera, CameraMovement, Projection};
use crate::command::CommandData;
use crate::depth::DepthResources;
use crate::frame_data::{FrameData};
use crate::gltf::Gltf;
use crate::gpu_mesh::GpuMesh;
use crate::mesh::Mesh;
use crate::pipeline::PipelineData;
use crate::registry::{Destroy, MeshRegistry, TextureId, TextureRegistry};
use crate::scene::{GpuEntity, Transform};
use crate::swapchain::{SwapchainData};
use crate::texture::{Texture};
use crate::ui::Ui;
use crate::vulkan_context::VulkanContext;
use crate::vulkan::{MAX_FRAMES_IN_FLIGHT, UniformBufferObject, create_texture_sampler, create_texture_sampler_ui};

use std::ptr::copy_nonoverlapping as memcpy;

pub struct App {
    pub vulkan_context: VulkanContext,
    pub swapchain_data: SwapchainData,
    pub command_data: CommandData,
    pub depth_resources: DepthResources,
    pub pipeline_data: PipelineData,
    pub frame_data: FrameData,
    pub ui: Ui,

    // TODO: move samplers to some new struct
    pub texture_sampler: vk::Sampler,
    pub texture_sampler_ui: vk::Sampler,
    
    pub gpu_entities: Vec<GpuEntity>,
    pub mesh_registry: MeshRegistry,
    pub texture_registry: TextureRegistry,

    pub frame: usize,
    pub resized: bool,

    pub camera: Camera,
    pub projection: Projection,
    pub camera_movement: CameraMovement,
}

impl App {
    pub unsafe fn create(window: Arc<Window>) -> Result<Self> {
        let vulkan_context = VulkanContext::new(&window)?;
        let swapchain_data = SwapchainData::new(&window, &vulkan_context)?;
        let command_data = CommandData::new(&vulkan_context, &swapchain_data)?;
        let depth_resources = DepthResources::new(&vulkan_context, &swapchain_data, &command_data)?;
        let pipeline_data = PipelineData::new(&vulkan_context, &swapchain_data, &depth_resources)?;
        let frame_data = FrameData::new(&vulkan_context, &swapchain_data, &pipeline_data)?;
        let ui = Ui::new(window);

        let texture_sampler = create_texture_sampler(&vulkan_context)?;
        let texture_sampler_ui = create_texture_sampler_ui(&vulkan_context)?;

        // Scene general
        let camera = Camera::new(
            point3(0.0, 0.0, 10.0),
            Deg(0.0),
            Deg(-90.0),
        );

        let projection = Projection::new(
            swapchain_data.swapchain_extent.width,
            swapchain_data.swapchain_extent.height,
            Deg(90.0),
            0.1,
            10000.0,
        );

        let camera_movement = CameraMovement::new(5.0, 0.0025);

        // CPU side
        let cube_mesh = Mesh::create_from_model("assets/cube.obj")?;
        let teapot_mesh = Mesh::create_from_model("assets/teapot.obj")?;

        let cube_bitmap = Bitmap::create_from_file("assets/cube.png")?;
        let teapot_bitmap = Bitmap::create_from_file("assets/viking_room.png")?;
        let white_bitmap = Bitmap::white();

        let (gltf_mesh, gltf_bitmap) = Gltf::load()?;

        let rotation = Euler { x: Rad(0.0), y: Rad(std::f32::consts::FRAC_PI_2), z: Rad(0.0) };
        let scale = Vector3 { x: 1.0, y: 1.0, z: 1.0 };

        let translation1 = Vector3 { x: 3.0, y: 0.0, z: 0.0 };
        let translation2 = Vector3 { x: 0.0, y: 0.0, z: 0.0 };
        let translation3 = Vector3 { x: -7.0, y: 0.0, z: 0.0 };
        let translation4 = Vector3 { x: -10.0, y: 0.0, z: 0.0 };
        let translation5 = Vector3 { x: 15.0, y: 0.0, z: 10.0 };
        let translation6 = Vector3 { x: 3.0, y: 10.0, z: 0.0 };

        let transform1 = Transform::new(translation1, rotation, scale);
        let transform2 = Transform::new(translation2, rotation, scale);
        let transform3 = Transform::new(translation3, rotation, scale);
        let transform4 = Transform::new(translation4, rotation, scale);
        let transform5 = Transform::new(translation5, rotation, scale);
        let transform6 = Transform::new(translation6, rotation, scale);

        // GPU side
        let cube_gpu_mesh = GpuMesh::create_from_mesh(&cube_mesh, &vulkan_context, &command_data)?;
        let teapot_gpu_mesh = GpuMesh::create_from_mesh(&teapot_mesh, &vulkan_context, &command_data)?;
        let sphere_gpu_mesh = GpuMesh::create_from_mesh(&Mesh::default_sphere(), &vulkan_context, &command_data)?;
        let generated_cube_gpu_mesh = GpuMesh::create_from_mesh(&Mesh::default_cube(), &vulkan_context, &command_data)?;
        let tetrahedron_gpu_mesh = GpuMesh::create_from_mesh(&Mesh::default_tetrahedron(), &vulkan_context, &command_data)?;
        let gltf_gpu_mesh = GpuMesh::create_from_mesh(&gltf_mesh, &vulkan_context, &command_data)?;

        let mut mesh_registry = MeshRegistry::new();
        let cube_mesh_id = mesh_registry.add(cube_gpu_mesh);
        let teapot_mesh_id = mesh_registry.add(teapot_gpu_mesh);
        let sphere_mesh_id = mesh_registry.add(sphere_gpu_mesh);
        let generated_cube_mesh_id = mesh_registry.add(generated_cube_gpu_mesh);
        let tetrahedron_mesh_id = mesh_registry.add(tetrahedron_gpu_mesh);
        let gltf_mesh_id = mesh_registry.add(gltf_gpu_mesh);

        let cube_texture = Texture::create_from_bitmap(&cube_bitmap, &vulkan_context, &command_data, texture_sampler)?;
        let teapot_texture = Texture::create_from_bitmap(&teapot_bitmap, &vulkan_context, &command_data, texture_sampler)?;
        let white_texture = Texture::create_from_bitmap(&white_bitmap, &vulkan_context, &command_data, texture_sampler)?;
        let gltf_texture = Texture::create_from_bitmap(&gltf_bitmap, &vulkan_context, &command_data, texture_sampler)?;

        let mut texture_registry = TextureRegistry::new();
        let cube_texture_id = texture_registry.add(cube_texture);
        let teapot_texture_id = texture_registry.add(teapot_texture);
        let white_texture_id = texture_registry.add(white_texture);
        let gltf_texture_id = texture_registry.add(gltf_texture);

        // Entities
        let entity1 = GpuEntity::new(cube_mesh_id, cube_texture_id, transform1);
        let entity2 = GpuEntity::new(teapot_mesh_id, teapot_texture_id, transform2);
        let entity3 = GpuEntity::new(sphere_mesh_id, white_texture_id, transform3);
        let entity4 = GpuEntity::new(generated_cube_mesh_id, white_texture_id, transform4);
        let entity5 = GpuEntity::new(tetrahedron_mesh_id, white_texture_id, transform5);
        let entity6 = GpuEntity::new(gltf_mesh_id, gltf_texture_id, transform6);

        let entities = vec![entity1, entity2, entity3, entity4, entity5, entity6];

        // let mut rng = rand::rng();
        // let mut transforms = Vec::<Transform>::new();
        // for _ in 0..10000 {
        //     let random_x: f32 = rand::Rng::random_range(&mut rng, -100.0..100.0);
        //     let random_y: f32 = rand::Rng::random_range(&mut rng, -100.0..100.0);
        //     let random_z: f32 = rand::Rng::random_range(&mut rng, -100.0..100.0);

        //     let translation = Vector3 { x: random_x, y: random_y, z: random_z };
        //     transforms.push(Transform::new(translation, rotation, scale));
        // }

        // for transform in transforms {
        //     entities.push(GpuEntity { mesh_id: tetrahedron_mesh_id, texture_id: white_texture_id, transform: transform });
        // }

        for id in 0..texture_registry.size() {
            let texture = texture_registry.get(TextureId(id));

            let image_info = vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(texture.image_view)
                .sampler(texture_sampler);

            let image_infos = &[image_info];
            let sampler_write = vk::WriteDescriptorSet::builder()
                .dst_set(pipeline_data.descriptor_set)
                .dst_binding(1)
                .dst_array_element(id as u32)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(image_infos);

            vulkan_context.device.update_descriptor_sets(&[sampler_write], &[] as &[vk::CopyDescriptorSet]);
        }

        Ok(Self { vulkan_context,
            swapchain_data,
            command_data,
            depth_resources,
            pipeline_data,
            frame_data,
            ui,
            texture_sampler,
            texture_sampler_ui,
            frame: 0,
            resized: false,
            gpu_entities: entities,
            mesh_registry,
            texture_registry,
            camera,
            projection,
            camera_movement,
        })
    }

    pub unsafe fn render(&mut self, window: &Window) -> Result<()> {
        self.vulkan_context.device.wait_for_fences(&[self.frame_data.in_flight_fences[self.frame]], true, u64::MAX)?;

        for buffer in self.frame_data.garbage_buffers[self.frame].drain(..) {
            buffer.destroy(&self.vulkan_context);
        }
    
        let result = self.vulkan_context.device.acquire_next_image_khr(
            self.swapchain_data.swapchain,
            u64::MAX,
            self.frame_data.image_available_semaphores[self.frame],
            vk::Fence::null(),
        );

        let image_index = match result {
            Ok((image_index, _)) => image_index as usize,
            Err(vk::ErrorCode::OUT_OF_DATE_KHR) => return self.recreate_swapchain(window),
            Err(e) => return Err(anyhow!(e)),
        };

        if !self.frame_data.images_in_flight[image_index].is_null() {
            self.vulkan_context.device.wait_for_fences(&[self.frame_data.images_in_flight[image_index]], true, u64::MAX)?;
        }
    
        self.frame_data.images_in_flight[image_index] = self.frame_data.in_flight_fences[self.frame];

        self.ui.update_textures(&self.vulkan_context, &self.command_data, &self.pipeline_data, &mut self.texture_registry, self.texture_sampler_ui)?;

        let garbage_buffers = self.command_data.update_command_buffer(
            image_index,
            &self.vulkan_context,
            &self.swapchain_data,
            &self.depth_resources,
            &self.pipeline_data,
            &self.gpu_entities,
            &self.mesh_registry,
            &mut self.ui,
        )?;
        self.frame_data.garbage_buffers[self.frame].extend(garbage_buffers);

        self.update_uniform_buffer()?;

        let wait_semaphores = &[self.frame_data.image_available_semaphores[self.frame]];
        let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = &[self.command_data.command_buffers[image_index]];
        let signal_semaphores = &[self.frame_data.render_finished_semaphores[image_index]];
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(command_buffers)
            .signal_semaphores(signal_semaphores);


        self.vulkan_context.device.reset_fences(&[self.frame_data.in_flight_fences[self.frame]])?;

        self.vulkan_context.device.queue_submit(self.vulkan_context.graphics_queue, &[submit_info], self.frame_data.in_flight_fences[self.frame])?;

        let swapchains = &[self.swapchain_data.swapchain];
        let image_indices = &[image_index as u32];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(signal_semaphores)
            .swapchains(swapchains)
            .image_indices(image_indices);

        let result = self.vulkan_context.device.queue_present_khr(self.vulkan_context.present_queue, &present_info);
        let changed = result == Ok(vk::SuccessCode::SUBOPTIMAL_KHR) || result == Err(vk::ErrorCode::OUT_OF_DATE_KHR);
        if self.resized || changed {
            self.resized = false;
            self.recreate_swapchain(window)?;
        } else if let Err(e) = result {
            return Err(anyhow!(e));
        }

        self.frame = (self.frame + 1) % MAX_FRAMES_IN_FLIGHT;
        Ok(())
    }

    pub unsafe fn destroy(mut self) {
        for garbage_buffer in &mut self.frame_data.garbage_buffers {
            for buffer in garbage_buffer.drain(..) {
                buffer.destroy(&self.vulkan_context);
            }
        }

        for mesh in self.mesh_registry.into_items() {
            mesh.destroy(&self.vulkan_context);
        }

        for texture in self.texture_registry.into_items() {
            texture.destroy(&self.vulkan_context);
        }

        self.vulkan_context.device.destroy_sampler(self.texture_sampler, None);
        self.vulkan_context.device.destroy_sampler(self.texture_sampler_ui, None);

        self.frame_data.destroy(&self.vulkan_context);
        self.pipeline_data.destroy(&self.vulkan_context);
        self.depth_resources.destroy(&self.vulkan_context);
        self.command_data.destroy(&self.vulkan_context);
        self.swapchain_data.destroy(&self.vulkan_context);
        self.vulkan_context.destroy();
    }

    pub fn is_ui_consumed(&mut self, window_event: &WindowEvent) -> bool {
        self.ui.is_consumed(window_event)
    }

    pub fn run_ui_frame(&mut self, ui_creation_callback: impl FnMut(&Context)) {
        self.ui.run_frame(ui_creation_callback);
    }

    unsafe fn recreate_swapchain(&mut self, window: &Window) -> Result<()> {
        self.vulkan_context.device.device_wait_idle()?;

        let old_swapchain = mem::take(&mut self.swapchain_data);
        old_swapchain.destroy(&self.vulkan_context);

        let old_depth = mem::take(&mut self.depth_resources);
        old_depth.destroy(&self.vulkan_context);

        self.swapchain_data = SwapchainData::new(window, &self.vulkan_context)?;
        self.depth_resources = DepthResources::new(&self.vulkan_context, &self.swapchain_data, &self.command_data)?;

        self.projection.resize(self.swapchain_data.swapchain_extent.width, self.swapchain_data.swapchain_extent.height);

        Ok(())
    }

    unsafe fn update_uniform_buffer(&self) -> Result<()> {
        let view = self.camera.get_view_matrix();
        let proj = self.projection.get_perspective_projection_matrix();
        let ubo = UniformBufferObject { view, proj, screen_size: vec2(self.swapchain_data.swapchain_extent.width, self.swapchain_data.swapchain_extent.height) };

        let memory = self.vulkan_context.device.map_memory(
            self.frame_data.uniform_buffers_memory[self.frame],
            0,
            size_of::<UniformBufferObject>() as u64,
            vk::MemoryMapFlags::empty(),
        )?;
        
        memcpy(&ubo, memory.cast(), 1);

        self.vulkan_context.device.unmap_memory(self.frame_data.uniform_buffers_memory[self.frame]);

        Ok(())
    }
}
