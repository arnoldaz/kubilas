use anyhow::{anyhow, Result};
use winit::window::Window;


use vulkanalia::loader::{LibloadingLoader, LIBRARY};
use vulkanalia::{window as vk_window};
use vulkanalia::prelude::v1_0::*;
use vulkanalia::vk::{DeviceV1_3, ExtDebugUtilsExtensionInstanceCommands, KhrSurfaceExtensionInstanceCommands, KhrSwapchainExtensionDeviceCommands};

use std::mem::size_of;
use cgmath::{Deg, Euler, Matrix4, Quaternion, Rad, Vector2, Vector3, Zero, point3};
use std::time::Instant;

use crate::bitmap::Bitmap;
use crate::camera::{Camera, CameraMovement, Projection};
use crate::gpu_mesh::GpuMesh;
use crate::image::{create_texture_sampler};
use crate::mesh::Mesh;
use crate::registry::{MeshRegistry, Registry, TextureId, TextureRegistry};
use crate::scene::{GpuEntity, Transform};
use crate::swapchain::{create_swapchain, create_swapchain_image_views};
use crate::texture::{Texture};
use crate::validations::{VALIDATION_ENABLED, create_instance, create_logical_device, pick_physical_device};
use crate::vulkan::{MAX_FRAMES_IN_FLIGHT, UniformBufferObject, create_command_buffers, create_command_pool, create_depth_objects, create_descriptor_pool, create_descriptor_set_layout, create_descriptor_sets, create_pipeline, create_sync_objects, create_uniform_buffers};

use std::ptr::copy_nonoverlapping as memcpy;
use vulkanalia_vma::{self as vma};

pub struct App {
    pub entry: Entry,
    pub instance: Instance,
    pub data: AppData,
    pub device: Device,
    pub frame: usize,
    pub resized: bool,
    pub start: Instant,
    pub gpu_entities: Vec<GpuEntity>,
    pub mesh_registry: MeshRegistry,
    pub texture_registry: TextureRegistry,

    pub camera: Camera,
    pub projection: Projection,
    pub camera_movement: CameraMovement,
}

impl App {
    pub unsafe fn create(window: &Window) -> Result<Self> {
        let loader = LibloadingLoader::new(LIBRARY)?;
        let entry = Entry::new(loader).map_err(|b| anyhow!("{}", b))?;

        let mut data = AppData::default();

        let instance = create_instance(window, &entry, &mut data)?;
        data.surface = vk_window::create_surface(&instance, &window, &window)?;
        pick_physical_device(&instance, &mut data)?;
        let device = create_logical_device(&entry, &instance, &mut data)?;
        create_swapchain(window, &instance, &device, &mut data)?;
        create_swapchain_image_views(&device, &mut data)?;
        create_descriptor_set_layout(&device, &mut data)?;
        create_pipeline(&instance, &device, &mut data)?;
        create_command_pool(&instance, &device, &mut data)?;
        create_depth_objects(&instance, &device, &mut data)?;

        let allocator_options = vma::AllocatorOptions::new(&instance, &device, data.physical_device);
        // allocator_options.version = Version::V1_4_0;
        let allocator = vma::Allocator::new(&allocator_options)?;
        data.allocator = Some(allocator);

        create_texture_sampler(&device, &mut data)?;

        create_descriptor_pool(&device, &mut data)?;
        create_descriptor_sets(&device, &mut data)?;

        let allocator = data.allocator.as_ref().unwrap();

        // Scene general
        let camera = Camera::new(
            point3(0.0, 0.0, 10.0),
            Deg(0.0),
            Deg(-90.0),
        );

        let projection = Projection::new(
            data.swapchain_extent.width,
            data.swapchain_extent.height,
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

        let rotation = Euler { x: Rad(0.0), y: Rad(std::f32::consts::FRAC_PI_2), z: Rad(0.0) };
        let scale = Vector3 { x: 1.0, y: 1.0, z: 1.0 };

        let translation1 = Vector3 { x: 3.0, y: 0.0, z: 0.0 };
        let translation2 = Vector3 { x: 0.0, y: 0.0, z: 0.0 };
        let translation3 = Vector3 { x: -7.0, y: 0.0, z: 0.0 };
        let translation4 = Vector3 { x: -10.0, y: 0.0, z: 0.0 };
        let translation5 = Vector3 { x: -15.0, y: 0.0, z: 10.0 };

        let transform1 = Transform::new(translation1, rotation, scale);
        let transform2 = Transform::new(translation2, rotation, scale);
        let transform3 = Transform::new(translation3, rotation, scale);
        let transform4 = Transform::new(translation4, rotation, scale);
        let transform5 = Transform::new(translation5, rotation, scale);

        // GPU side
        let cube_gpu_mesh = GpuMesh::create_from_mesh(&cube_mesh, allocator, &device, &data)?;
        let teapot_gpu_mesh = GpuMesh::create_from_mesh(&teapot_mesh, allocator, &device, &data)?;
        let sphere_gpu_mesh = GpuMesh::create_from_mesh(&Mesh::default_sphere(), allocator, &device, &data)?;
        let generated_cube_gpu_mesh = GpuMesh::create_from_mesh(&Mesh::default_cube(), allocator, &device, &data)?;
        let tetrahedron_gpu_mesh = GpuMesh::create_from_mesh(&Mesh::default_tetrahedron(), allocator, &device, &data)?;

        let mut mesh_registry = MeshRegistry::new();
        let cube_mesh_id = mesh_registry.add(cube_gpu_mesh);
        let teapot_mesh_id = mesh_registry.add(teapot_gpu_mesh);
        let sphere_mesh_id = mesh_registry.add(sphere_gpu_mesh);
        let generated_cube_mesh_id = mesh_registry.add(generated_cube_gpu_mesh);
        let tetrahedron_mesh_id = mesh_registry.add(tetrahedron_gpu_mesh);

        let cube_texture = Texture::create_from_bitmap(&cube_bitmap, &instance, &device, &mut data)?;
        let teapot_texture = Texture::create_from_bitmap(&teapot_bitmap, &instance, &device, &mut data)?;
        let white_texture = Texture::create_from_bitmap(&white_bitmap, &instance, &device, &mut data)?;

        let mut texture_registry = TextureRegistry::new();
        let cube_texture_id = texture_registry.add(cube_texture);
        let teapot_texture_id = texture_registry.add(teapot_texture);
        let white_texture_id = texture_registry.add(white_texture);

        // Entities
        let entity1 = GpuEntity::new(cube_mesh_id, cube_texture_id, transform1);
        let entity2 = GpuEntity::new(teapot_mesh_id, teapot_texture_id, transform2);
        let entity3 = GpuEntity::new(sphere_mesh_id, white_texture_id, transform3);
        let entity4 = GpuEntity::new(generated_cube_mesh_id, white_texture_id, transform4);
        let entity5 = GpuEntity::new(tetrahedron_mesh_id, white_texture_id, transform5);

        let entities = vec![entity1, entity2, entity3, entity4, entity5];

        for id in 0..texture_registry.size() {
            let texture = texture_registry.get(TextureId(id));
            // data.sampler_index += 1;
            let image_info = vk::DescriptorImageInfo::builder()
                .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .image_view(texture.image_view)
                .sampler(data.texture_sampler);

            let image_infos = &[image_info];
            let sampler_write = vk::WriteDescriptorSet::builder()
                .dst_set(data.descriptor_set)
                .dst_binding(1)
                .dst_array_element(id as u32)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .image_info(image_infos);

            device.update_descriptor_sets(&[sampler_write], &[] as &[vk::CopyDescriptorSet]);
        }


        create_uniform_buffers(&instance, &device, &mut data)?;
        create_command_buffers(&device, &mut data)?;
        create_sync_objects(&device, &mut data)?;

        Ok(Self { entry, instance, data, device, frame: 0, resized: false, start: Instant::now(), gpu_entities: entities, mesh_registry, texture_registry, camera, projection, camera_movement })
    }

    pub unsafe fn render(&mut self, window: &Window) -> Result<()> {
        self.device.wait_for_fences(
            &[self.data.in_flight_fences[self.frame]],
            true,
            u64::MAX,
        )?;
    
        let result = self.device.acquire_next_image_khr(
            self.data.swapchain,
            u64::MAX,
            self.data.image_available_semaphores[self.frame],
            vk::Fence::null(),
        );

        let image_index = match result {
            Ok((image_index, _)) => image_index as usize,
            Err(vk::ErrorCode::OUT_OF_DATE_KHR) => return self.recreate_swapchain(window),
            Err(e) => return Err(anyhow!(e)),
        };

        // println!("{} img index. {} len self.data.render_finished_semaphores. {} frame", image_index, self.data.render_finished_semaphores.len(), self.frame);

        if !self.data.images_in_flight[image_index as usize].is_null() {
            self.device.wait_for_fences(
                &[self.data.images_in_flight[image_index as usize]],
                true,
                u64::MAX,
            )?;
        }
    
        self.data.images_in_flight[image_index as usize] = self.data.in_flight_fences[self.frame];

        self.update_command_buffer(image_index)?;
        self.update_uniform_buffer(image_index)?;

        let wait_semaphores = &[self.data.image_available_semaphores[self.frame]];
        let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = &[self.data.command_buffers[image_index as usize]];
        let signal_semaphores = &[self.data.render_finished_semaphores[image_index as usize]];
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(command_buffers)
            .signal_semaphores(signal_semaphores);


        self.device.reset_fences(&[self.data.in_flight_fences[self.frame]])?;

        self.device.queue_submit(self.data.graphics_queue, &[submit_info], self.data.in_flight_fences[self.frame])?;

        let swapchains = &[self.data.swapchain];
        let image_indices = &[image_index as u32];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(signal_semaphores)
            .swapchains(swapchains)
            .image_indices(image_indices);

        let result = self.device.queue_present_khr(self.data.present_queue, &present_info);
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

    pub unsafe fn destroy(&mut self) {
        // old destroy swapchain
        self.device.destroy_image_view(self.data.depth_image_view, None);
        self.device.free_memory(self.data.depth_image_memory, None);
        self.device.destroy_image(self.data.depth_image, None);
        self.device.destroy_descriptor_pool(self.data.descriptor_pool, None);
        self.data.uniform_buffers
            .iter()
            .for_each(|b| self.device.destroy_buffer(*b, None));
        self.data.uniform_buffers_memory
            .iter()
            .for_each(|m| self.device.free_memory(*m, None));
        self.device.free_command_buffers(self.data.command_pool, &self.data.command_buffers);
        self.device.destroy_pipeline(self.data.pipeline, None);
        self.device.destroy_pipeline_layout(self.data.pipeline_layout, None);
        self.data.swapchain_image_views
            .iter()
            .for_each(|v| self.device.destroy_image_view(*v, None));
        self.device.destroy_swapchain_khr(self.data.swapchain, None);
        // end of old destroy swapchain

        self.device.destroy_sampler(self.data.texture_sampler, None);
        self.device.destroy_descriptor_set_layout(self.data.descriptor_set_layout, None);

        // self.gpu_render_objects
        //     .iter()
        //     .for_each(|o| o.destroy(&self.device, self.data.allocator.as_ref().unwrap()));

        for texture in self.texture_registry.into_items() {
            self.device.destroy_image_view(texture.image_view, None);
            self.device.destroy_image(texture.image, None);
            self.device.free_memory(texture.image_memory, None);
        }

        let allocator = self.data.allocator.take().expect("AAAA");
        for mesh in self.mesh_registry.into_items() {
            allocator.destroy_buffer(mesh.index_buffer.buffer, mesh.index_buffer.allocation);
            allocator.destroy_buffer(mesh.vertex_buffer.buffer, mesh.vertex_buffer.allocation);
        }


        self.data.in_flight_fences
            .iter()
            .for_each(|f| self.device.destroy_fence(*f, None));
        self.data.render_finished_semaphores
            .iter()
            .for_each(|s| self.device.destroy_semaphore(*s, None));
        self.data.image_available_semaphores
            .iter()
            .for_each(|s| self.device.destroy_semaphore(*s, None));
        self.device.destroy_command_pool(self.data.command_pool, None);
        drop(allocator);
        self.device.destroy_device(None);
        self.instance.destroy_surface_khr(self.data.surface, None);
    
        if VALIDATION_ENABLED {
            self.instance.destroy_debug_utils_messenger_ext(self.data.messenger, None);
        }
    
        self.instance.destroy_instance(None);
    }

    pub unsafe fn recreate_swapchain(&mut self, window: &Window) -> Result<()> {
        self.device.device_wait_idle()?;

        self.data.swapchain_image_views
            .iter()
            .for_each(|v| self.device.destroy_image_view(*v, None));
        self.device.destroy_image_view(self.data.depth_image_view, None);
        self.device.free_memory(self.data.depth_image_memory, None);
        self.device.destroy_image(self.data.depth_image, None);
        self.device.destroy_swapchain_khr(self.data.swapchain, None);

        create_swapchain(window, &self.instance, &self.device, &mut self.data)?;
        create_swapchain_image_views(&self.device, &mut self.data)?;
        create_depth_objects(&self.instance, &self.device, &mut self.data)?;

        self.projection.resize(self.data.swapchain_extent.width, self.data.swapchain_extent.height);

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

    unsafe fn update_command_buffer(&mut self, image_index: usize) -> Result<()> {
        let command_buffer = self.data.command_buffers[image_index];

        self.device.reset_command_buffer(
            command_buffer,
            vk::CommandBufferResetFlags::empty(),
        )?;

        let time = self.start.elapsed().as_secs_f32();


        let info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        self.device.begin_command_buffer(command_buffer, &info)?;

        let render_area = vk::Rect2D::builder()
            .offset(vk::Offset2D::default())
            .extent(self.data.swapchain_extent);

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
            .image_view(self.data.swapchain_image_views[image_index])
            .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .clear_value(color_clear_value);
        let color_attachments = [color_attachment];

        let depth_attachment = vk::RenderingAttachmentInfo::builder()
            .image_view(self.data.depth_image_view)
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
            .image(self.data.swapchain_images[image_index])
            .subresource_range(color_range)
            .src_stage_mask(vk::PipelineStageFlags2::TOP_OF_PIPE)
            .dst_stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT);

        let binding = [barrier];
        let dependency_info = vk::DependencyInfo::builder()
            .image_memory_barriers(&binding);

        self.device.cmd_pipeline_barrier2(
            command_buffer,
            &dependency_info
        );

        self.device.cmd_begin_rendering(command_buffer, &rendering_info);
        self.device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, self.data.pipeline);

        let viewport = vk::Viewport::builder()
            .x(0.0)
            .y(0.0)
            .width(self.data.swapchain_extent.width as f32)
            .height(self.data.swapchain_extent.height as f32)
            .min_depth(0.0)
            .max_depth(1.0);

        let scissor = vk::Rect2D::builder()
            .offset(vk::Offset2D { x: 0, y: 0 })
            .extent(self.data.swapchain_extent);

        self.device.cmd_set_viewport(command_buffer, 0, &[viewport]);
        self.device.cmd_set_scissor(command_buffer, 0, &[scissor]);

        self.device.cmd_bind_descriptor_sets(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.data.pipeline_layout,
            0,
            &[self.data.descriptor_set],
            &[],
        );

        for gpu_entity in &self.gpu_entities {
            let mesh = self.mesh_registry.get(gpu_entity.mesh_id);
            let texture = self.texture_registry.get(gpu_entity.texture_id);

            // TODO: have only 1 buffer for both and use offset instead, nvidia dev guide says it's very bad now
            self.device.cmd_bind_vertex_buffers(command_buffer, 0, &[mesh.vertex_buffer.buffer], &[0]);
            self.device.cmd_bind_index_buffer(command_buffer, mesh.index_buffer.buffer, 0, vk::IndexType::UINT32);

            let mut new_rotation = gpu_entity.transform.rotation.clone();
            new_rotation.y *= time;
            let new_quaternion = Quaternion::from(new_rotation);
            let model = Self::trs_matrix(gpu_entity.transform.translation, new_quaternion, gpu_entity.transform.scale);
            let model_bytes = std::slice::from_raw_parts(
                &model as *const Matrix4<f32> as *const u8,
                size_of::<Matrix4<f32>>()
            );

            self.device.cmd_push_constants(
                command_buffer,
                self.data.pipeline_layout,
                vk::ShaderStageFlags::VERTEX,
                0,
                model_bytes,
            );

            let obj_index_bytes = std::slice::from_raw_parts(&(gpu_entity.texture_id.0 as u32) as *const u32 as *const u8, 4);
            self.device.cmd_push_constants(
                command_buffer,
                self.data.pipeline_layout,
                vk::ShaderStageFlags::FRAGMENT,
                64, // offset vertex push constants
                obj_index_bytes,
            );

            self.device.cmd_draw_indexed(command_buffer, mesh.index_count, 1, 0, 0, 0);
        }

        self.device.cmd_end_rendering(command_buffer);

        // TODO: abstract this bullshit
        let barrier = vk::ImageMemoryBarrier2::builder()
            .old_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .src_access_mask(vk::AccessFlags2::COLOR_ATTACHMENT_WRITE)
            .dst_access_mask(vk::AccessFlags2::empty())
            .image(self.data.swapchain_images[image_index])
            .subresource_range(color_range)
            .src_stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)
            .dst_stage_mask(vk::PipelineStageFlags2::BOTTOM_OF_PIPE);

        let binding = [barrier];
        let dependency_info = vk::DependencyInfo::builder()
            .image_memory_barriers(&binding);

        self.device.cmd_pipeline_barrier2(
            command_buffer,
            &dependency_info
        );

        self.device.end_command_buffer(command_buffer)?;

        Ok(())
    }

    unsafe fn update_uniform_buffer(&self, image_index: usize) -> Result<()> {
        let view = self.camera.get_view_matrix();
        let proj = self.projection.get_perspective_projection_matrix();
        let ubo = UniformBufferObject { view, proj };

        let memory = self.device.map_memory(
            self.data.uniform_buffers_memory[self.frame],
            0,
            size_of::<UniformBufferObject>() as u64,
            vk::MemoryMapFlags::empty(),
        )?;
        
        memcpy(&ubo, memory.cast(), 1);

        self.device.unmap_memory(self.data.uniform_buffers_memory[self.frame]);

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct AppData {
    pub surface: vk::SurfaceKHR,
    pub messenger: vk::DebugUtilsMessengerEXT,
    pub physical_device: vk::PhysicalDevice,
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,
    pub swapchain_format: vk::Format,
    pub swapchain_extent: vk::Extent2D,
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_images: Vec<vk::Image>,
    pub swapchain_image_views: Vec<vk::ImageView>,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub pipeline_layout: vk::PipelineLayout,
    pub pipeline: vk::Pipeline,
    pub command_pool: vk::CommandPool,
    pub command_buffers: Vec<vk::CommandBuffer>,
    pub image_available_semaphores: Vec<vk::Semaphore>,
    pub render_finished_semaphores: Vec<vk::Semaphore>,
    pub in_flight_fences: Vec<vk::Fence>,
    pub images_in_flight: Vec<vk::Fence>,
    // pub vertices: Vec<Vertex>,
    // pub indices: Vec<u32>,
    // pub vertex_buffer: vk::Buffer,
    // pub vertex_buffer_memory: vk::DeviceMemory,
    // pub index_buffer: vk::Buffer,
    // pub index_buffer_memory: vk::DeviceMemory,
    pub uniform_buffers: Vec<vk::Buffer>,
    pub uniform_buffers_memory: Vec<vk::DeviceMemory>,
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_set: vk::DescriptorSet,
    pub ubo_index: i32,
    // pub sampler_index: i32,
    // pub texture_image: vk::Image,
    // pub texture_image_memory: vk::DeviceMemory,
    // pub texture_image_view: vk::ImageView,
    pub texture_sampler: vk::Sampler,
    pub depth_image: vk::Image,
    pub depth_image_memory: vk::DeviceMemory,
    pub depth_image_view: vk::ImageView,

    pub allocator: Option<vma::Allocator>,
}