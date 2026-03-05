use anyhow::{Result};
use vulkanalia::{bytecode::Bytecode, vk::{self, DeviceV1_0, Handle, HasBuilder}};
use crate::{depth::DepthResources, swapchain::SwapchainData, vertex::{UiVertex, Vertex}, vulkan_context::VulkanContext};

pub struct PipelineData {
    pub descriptor_set_layout: vk::DescriptorSetLayout,

    pub pipeline: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,

    pub ui_pipeline: vk::Pipeline,
    pub ui_pipeline_layout: vk::PipelineLayout,

    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_set: vk::DescriptorSet,
}

impl PipelineData {
    pub unsafe fn new(vulkan_context: &VulkanContext, swapchain_data: &SwapchainData, depth_resources: &DepthResources) -> Result<Self> {
        let descriptor_set_layout = Self::create_descriptor_set_layout(vulkan_context)?;
        let (pipeline, pipeline_layout) = Self::create_pipeline(descriptor_set_layout, vulkan_context, swapchain_data, depth_resources)?;
        let (ui_pipeline, ui_pipeline_layout) = Self::create_pipeline_ui(descriptor_set_layout, vulkan_context, swapchain_data, depth_resources)?;
        let descriptor_pool = Self::create_descriptor_pool(vulkan_context)?;
        let descriptor_set = Self::create_descriptor_sets(descriptor_set_layout, descriptor_pool, vulkan_context)?;

        Ok(Self { descriptor_set_layout, pipeline, pipeline_layout, ui_pipeline, ui_pipeline_layout, descriptor_pool, descriptor_set })
    }

    pub unsafe fn destroy(self, vulkan_context: &VulkanContext) {
        vulkan_context.device.destroy_descriptor_pool(self.descriptor_pool, None);
        vulkan_context.device.destroy_pipeline(self.pipeline, None);
        vulkan_context.device.destroy_pipeline_layout(self.pipeline_layout, None);
        vulkan_context.device.destroy_pipeline(self.ui_pipeline, None);
        vulkan_context.device.destroy_pipeline_layout(self.ui_pipeline_layout, None);
        vulkan_context.device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
    }

    unsafe fn create_descriptor_set_layout(vulkan_context: &VulkanContext) -> Result<vk::DescriptorSetLayout> {
        let ubo_binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(65536)
            .stage_flags(vk::ShaderStageFlags::VERTEX);

        let sampler_binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(1)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(65536)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT);

        let bindings = &[ubo_binding, sampler_binding];

        let binding_flags = &[
            vk::DescriptorBindingFlags::PARTIALLY_BOUND | vk::DescriptorBindingFlags::UPDATE_AFTER_BIND,
            vk::DescriptorBindingFlags::PARTIALLY_BOUND | vk::DescriptorBindingFlags::UPDATE_AFTER_BIND
        ];
        let mut binding_flags_info = vk::DescriptorSetLayoutBindingFlagsCreateInfo::builder()
            .binding_flags(binding_flags);

        let info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(bindings)
            .flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL)
            .push_next(&mut binding_flags_info);
        
        let descriptor_set_layout = vulkan_context.device.create_descriptor_set_layout(&info, None)?;

        Ok(descriptor_set_layout)
    }

    unsafe fn create_pipeline(descriptor_set_layout: vk::DescriptorSetLayout, vulkan_context: &VulkanContext, swapchain_data: &SwapchainData, depth_resources: &DepthResources) -> Result<(vk::Pipeline, vk::PipelineLayout)> {
        let vert = include_bytes!("../target/shaders/main/vert.spv");
        let frag = include_bytes!("../target/shaders/main/frag.spv");

        let vert_shader_module = Self::create_shader_module(vulkan_context, &vert[..])?;
        let frag_shader_module = Self::create_shader_module(vulkan_context, &frag[..])?;

        let vert_stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vert_shader_module)
            .name(b"main\0");

        let frag_stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(frag_shader_module)
            .name(b"main\0");

        let binding_descriptions = &[Vertex::binding_description()];
        let attribute_descriptions = Vertex::attribute_descriptions();
        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(binding_descriptions)
            .vertex_attribute_descriptions(&attribute_descriptions);

        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewport_count(1)
            .scissor_count(1);

        let dynamic_state_info = vk::PipelineDynamicStateCreateInfo::builder()
            .dynamic_states(&[vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR]);

        let rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .depth_bias_enable(false);

        let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::_1);

        let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS)
            .depth_bounds_test_enable(false)
            .min_depth_bounds(0.0)
            .max_depth_bounds(1.0)
            .stencil_test_enable(false);

        let attachment = vk::PipelineColorBlendAttachmentState::builder()
            .blend_enable(true)
            .color_write_mask(vk::ColorComponentFlags::all())
            .src_color_blend_factor(vk::BlendFactor::ONE)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .alpha_blend_op(vk::BlendOp::ADD);

        let attachments = &[attachment];
        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(attachments)
            .blend_constants([0.0, 0.0, 0.0, 0.0]);

        let vert_push_constant_range = vk::PushConstantRange::builder()
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .offset(0)
            .size(64 /* 16 × 4 byte floats */);

        let frag_range = vk::PushConstantRange::builder()
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .offset(64) // offset after vertex constants
            .size(4);

        let set_layouts = &[descriptor_set_layout];
        let push_constant_ranges = &[vert_push_constant_range, frag_range];
        let layout_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(set_layouts)
            .push_constant_ranges(push_constant_ranges);

        let pipeline_layout = vulkan_context.device.create_pipeline_layout(&layout_info, None)?;

        let stages = &[vert_stage, frag_stage];

        let color_formats = &[swapchain_data.swapchain_format];
        let mut pipeline_rendering_info = vk::PipelineRenderingCreateInfo::builder()
            .color_attachment_formats(color_formats)
            .depth_attachment_format(depth_resources.depth_format)
            .stencil_attachment_format(depth_resources.depth_format);

        let info = vk::GraphicsPipelineCreateInfo::builder()
            .push_next(&mut pipeline_rendering_info)
            .stages(stages)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly_state)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization_state)
            .multisample_state(&multisample_state)
            .depth_stencil_state(&depth_stencil_state)
            .color_blend_state(&color_blend_state)
            .layout(pipeline_layout)
            .render_pass(vk::RenderPass::null())
            .dynamic_state(&dynamic_state_info)
            .base_pipeline_handle(vk::Pipeline::null())
            .base_pipeline_index(-1);

        let pipeline = vulkan_context.device.create_graphics_pipelines(vk::PipelineCache::null(), &[info], None)?.0[0];

        vulkan_context.device.destroy_shader_module(vert_shader_module, None);
        vulkan_context.device.destroy_shader_module(frag_shader_module, None);

        Ok((pipeline, pipeline_layout))
    }

    unsafe fn create_pipeline_ui(descriptor_set_layout: vk::DescriptorSetLayout, vulkan_context: &VulkanContext, swapchain_data: &SwapchainData, depth_resources: &DepthResources) -> Result<(vk::Pipeline, vk::PipelineLayout)> {
        let vert = include_bytes!("../target/shaders/ui/vert.spv");
        let frag = include_bytes!("../target/shaders/ui/frag.spv");

        let vert_shader_module = Self::create_shader_module(vulkan_context, &vert[..])?;
        let frag_shader_module = Self::create_shader_module(vulkan_context, &frag[..])?;

        let vert_stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vert_shader_module)
            .name(b"main\0");

        let frag_stage = vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(frag_shader_module)
            .name(b"main\0");

        let binding_descriptions = &[UiVertex::binding_description()];
        let attribute_descriptions = UiVertex::attribute_descriptions();
        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(binding_descriptions)
            .vertex_attribute_descriptions(&attribute_descriptions);

        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewport_count(1)
            .scissor_count(1);

        let dynamic_state_info = vk::PipelineDynamicStateCreateInfo::builder()
            .dynamic_states(&[vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR]);

        let rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::NONE)
            .front_face(vk::FrontFace::CLOCKWISE)
            .depth_bias_enable(false);

        let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::_1);

        let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(false)
            .depth_write_enable(false)
            .depth_compare_op(vk::CompareOp::ALWAYS);
            // .depth_bounds_test_enable(false)
            // .stencil_test_enable(false);

        let attachment = vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(vk::ColorComponentFlags::all())
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD);

        let attachments = &[attachment];
        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(attachments)
            .blend_constants([0.0, 0.0, 0.0, 0.0]);

        let vert_push_constant_range = vk::PushConstantRange::builder()
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .offset(0)
            .size(64 /* 16 × 4 byte floats */);

        let frag_range = vk::PushConstantRange::builder()
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .offset(64) // offset after vertex constants
            .size(4);

        let set_layouts = &[descriptor_set_layout];
        let push_constant_ranges = &[vert_push_constant_range, frag_range];
        let layout_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(set_layouts)
            .push_constant_ranges(push_constant_ranges);

        let pipeline_layout = vulkan_context.device.create_pipeline_layout(&layout_info, None)?;

        let stages = &[vert_stage, frag_stage];

        let color_formats = &[swapchain_data.swapchain_format];
        let mut pipeline_rendering_info = vk::PipelineRenderingCreateInfo::builder()
            .color_attachment_formats(color_formats)
            .depth_attachment_format(depth_resources.depth_format)
            .stencil_attachment_format(depth_resources.depth_format);

        let info = vk::GraphicsPipelineCreateInfo::builder()
            .push_next(&mut pipeline_rendering_info)
            .stages(stages)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly_state)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization_state)
            .multisample_state(&multisample_state)
            .depth_stencil_state(&depth_stencil_state)
            .color_blend_state(&color_blend_state)
            .layout(pipeline_layout)
            .render_pass(vk::RenderPass::null())
            .dynamic_state(&dynamic_state_info)
            .base_pipeline_handle(vk::Pipeline::null())
            .base_pipeline_index(-1);

        let pipeline = vulkan_context.device.create_graphics_pipelines(vk::PipelineCache::null(), &[info], None)?.0[0];

        vulkan_context.device.destroy_shader_module(vert_shader_module, None);
        vulkan_context.device.destroy_shader_module(frag_shader_module, None);

        Ok((pipeline, pipeline_layout))
    }

    unsafe fn create_shader_module(vulkan_context: &VulkanContext, bytecode: &[u8]) -> Result<vk::ShaderModule> {
        let bytecode = Bytecode::new(bytecode).unwrap();

        let info = vk::ShaderModuleCreateInfo::builder()
            .code_size(bytecode.code_size())
            .code(bytecode.code());

        let shader_module = vulkan_context.device.create_shader_module(&info, None)?;

        Ok(shader_module)
    }

    unsafe fn create_descriptor_pool(vulkan_context: &VulkanContext) -> Result<vk::DescriptorPool> {
        let ubo_size = vk::DescriptorPoolSize::builder()
            .type_(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(65536);
        
        let sampler_size = vk::DescriptorPoolSize::builder()
            .type_(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(65536);

        let pool_sizes = &[ubo_size, sampler_size];
        let info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(pool_sizes)
            .max_sets(1)
            .flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND);

        let descriptor_pool = vulkan_context.device.create_descriptor_pool(&info, None)?;

        Ok(descriptor_pool)
    }

    unsafe fn create_descriptor_sets(descriptor_set_layout: vk::DescriptorSetLayout, descriptor_pool: vk::DescriptorPool, vulkan_context: &VulkanContext) -> Result<vk::DescriptorSet> {
        let layouts = &[descriptor_set_layout];
        let info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool)
            .set_layouts(layouts);

        let descriptor_set = vulkan_context.device.allocate_descriptor_sets(&info)?[0];

        Ok(descriptor_set)
    }
}