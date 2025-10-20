use ash::vk;
use anyhow::Result;
use glam::{Mat4, Vec4};

use crate::core::RenderPass;
use crate::game::Game;

/// Push constants for outline rendering
#[repr(C)]
#[derive(Copy, Clone)]
pub struct OutlinePushConstants {
    pub model: Mat4,
    pub outline_color: Vec4,  // RGB color + alpha
    pub outline_width: f32,   // How much to expand the mesh (typical: 0.01-0.05)
    pub _padding1: f32,
    pub _padding2: f32,
    pub _padding3: f32,
}

unsafe impl bytemuck::Pod for OutlinePushConstants {}
unsafe impl bytemuck::Zeroable for OutlinePushConstants {}

pub struct OutlinePass {
    descriptor_set_layout: vk::DescriptorSetLayout,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    descriptor_sets: Vec<vk::DescriptorSet>,
}

impl OutlinePass {
    pub fn new() -> Self {
        Self {
            descriptor_set_layout: vk::DescriptorSetLayout::null(),
            pipeline_layout: vk::PipelineLayout::null(),
            pipeline: vk::Pipeline::null(),
            descriptor_sets: Vec::new(),
        }
    }
}

impl RenderPass for OutlinePass {
    fn initialize(
        &mut self,
        ctx: &crate::core::RenderContext,
        render_pass: vk::RenderPass,
        extent: vk::Extent2D,
    ) -> Result<()> {
        unsafe {
            // Get shared descriptor sets from context
            if let (Some(mesh_descriptor_sets),) = (ctx.mesh_descriptor_sets,) {
                self.descriptor_sets = mesh_descriptor_sets.to_vec();
            }

            // Create descriptor set layout
            self.descriptor_set_layout = Self::create_descriptor_set_layout(ctx.device)?;

            // Create pipeline
            let (pipeline_layout, pipeline) = Self::create_pipeline(
                ctx.device,
                extent,
                render_pass,
                self.descriptor_set_layout,
            )?;
            self.pipeline_layout = pipeline_layout;
            self.pipeline = pipeline;

            Ok(())
        }
    }

    fn update(
        &mut self,
        _ctx: &crate::core::RenderContext,
        _frame_index: usize,
        _game: &Game,
    ) -> Result<()> {
        Ok(())
    }

    fn render(
        &mut self,
        ctx: &crate::core::RenderContext,
        command_buffer: vk::CommandBuffer,
        frame_index: usize,
        game: &Game,
    ) -> Result<()> {
        unsafe {
            if self.pipeline == vk::Pipeline::null() {
                return Ok(());
            }

            // Get outlined objects from game (selected or highlighted objects)
            let outlined_objects = game.get_outlined_objects();
            if outlined_objects.is_empty() {
                return Ok(());
            }

            // Bind outline pipeline
            ctx.device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline,
            );

            // Bind descriptor set
            ctx.device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_layout,
                0,
                &[self.descriptor_sets[frame_index]],
                &[],
            );

            // Render each outlined object
            if let Some(custom_meshes) = ctx.custom_meshes {
                for (mesh_path, model_matrix, outline_color, outline_width) in outlined_objects.iter() {
                    if let Some((mesh, vertex_buffer, _vertex_memory, index_buffer, _index_memory)) = custom_meshes.get(mesh_path) {
                        // Bind mesh buffers
                        let vertex_buffers = [*vertex_buffer];
                        let offsets = [0_u64];
                        ctx.device.cmd_bind_vertex_buffers(command_buffer, 0, &vertex_buffers, &offsets);
                        ctx.device.cmd_bind_index_buffer(command_buffer, *index_buffer, 0, vk::IndexType::UINT32);

                        // Push constants
                        let push_data = OutlinePushConstants {
                            model: *model_matrix,
                            outline_color: *outline_color,
                            outline_width: *outline_width,
                            _padding1: 0.0,
                            _padding2: 0.0,
                            _padding3: 0.0,
                        };
                        let push_constants = bytemuck::bytes_of(&push_data);
                        ctx.device.cmd_push_constants(
                            command_buffer,
                            self.pipeline_layout,
                            vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                            0,
                            push_constants,
                        );

                        ctx.device.cmd_draw_indexed(command_buffer, mesh.indices.len() as u32, 1, 0, 0, 0);
                    }
                }
            }

            Ok(())
        }
    }

    fn recreate_swapchain(
        &mut self,
        ctx: &crate::core::RenderContext,
        render_pass: vk::RenderPass,
        extent: vk::Extent2D,
    ) -> Result<()> {
        unsafe {
            // Destroy old pipeline
            if self.pipeline != vk::Pipeline::null() {
                ctx.device.destroy_pipeline(self.pipeline, None);
                ctx.device.destroy_pipeline_layout(self.pipeline_layout, None);
            }

            // Create new pipeline
            let (pipeline_layout, pipeline) = Self::create_pipeline(
                ctx.device,
                extent,
                render_pass,
                self.descriptor_set_layout,
            )?;
            self.pipeline_layout = pipeline_layout;
            self.pipeline = pipeline;

            Ok(())
        }
    }

    fn cleanup(&mut self, device: &ash::Device) {
        unsafe {
            if self.pipeline != vk::Pipeline::null() {
                device.destroy_pipeline(self.pipeline, None);
            }
            if self.pipeline_layout != vk::PipelineLayout::null() {
                device.destroy_pipeline_layout(self.pipeline_layout, None);
            }
            if self.descriptor_set_layout != vk::DescriptorSetLayout::null() {
                device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
            }
        }
    }

    fn name(&self) -> &str {
        "Outline"
    }
}

impl OutlinePass {
    unsafe fn create_descriptor_set_layout(device: &ash::Device) -> Result<vk::DescriptorSetLayout> {
        let ubo_binding = vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT);

        let bindings = [ubo_binding];
        let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);

        Ok(device.create_descriptor_set_layout(&layout_info, None)?)
    }

    unsafe fn create_pipeline(
        device: &ash::Device,
        extent: vk::Extent2D,
        render_pass: vk::RenderPass,
        descriptor_set_layout: vk::DescriptorSetLayout,
    ) -> Result<(vk::PipelineLayout, vk::Pipeline)> {
        use std::ffi::CString;

        let vert_shader_code = include_bytes!("../../../shaders/outline.vert.spv");
        let frag_shader_code = include_bytes!("../../../shaders/outline.frag.spv");

        let vert_shader_module = Self::create_shader_module(device, vert_shader_code)?;
        let frag_shader_module = Self::create_shader_module(device, frag_shader_code)?;

        let entry_point = CString::new("main")?;

        let vert_stage_info = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vert_shader_module)
            .name(&entry_point);

        let frag_stage_info = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(frag_shader_module)
            .name(&entry_point);

        let shader_stages = [vert_stage_info, frag_stage_info];

        // Vertex input
        let binding_description = vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(std::mem::size_of::<crate::mesh::Vertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX);

        let attribute_descriptions = [
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(0),
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(1)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(12),
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(2)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(24),
        ];

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(std::slice::from_ref(&binding_description))
            .vertex_attribute_descriptions(&attribute_descriptions);

        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: extent.width as f32,
            height: extent.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };

        let scissor = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent,
        };

        let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .viewports(std::slice::from_ref(&viewport))
            .scissors(std::slice::from_ref(&scissor));

        let rasterizer = vk::PipelineRasterizationStateCreateInfo::default()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::FRONT) // Cull front faces to show only outline
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .depth_bias_enable(false);

        let multisampling = vk::PipelineMultisampleStateCreateInfo::default()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        // Depth testing to avoid z-fighting
        let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default()
            .depth_test_enable(true)
            .depth_write_enable(false) // Don't write depth for outline
            .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL); // Allow outline to show behind object

        // No blending needed for solid outline
        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(false);

        let color_blending = vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .attachments(std::slice::from_ref(&color_blend_attachment));

        // Push constants
        let push_constant_range = vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
            .offset(0)
            .size(std::mem::size_of::<OutlinePushConstants>() as u32);

        let set_layouts = [descriptor_set_layout];
        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(&set_layouts)
            .push_constant_ranges(std::slice::from_ref(&push_constant_range));

        let pipeline_layout = device.create_pipeline_layout(&pipeline_layout_info, None)?;

        let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterizer)
            .multisample_state(&multisampling)
            .depth_stencil_state(&depth_stencil)
            .color_blend_state(&color_blending)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0);

        let pipelines = device
            .create_graphics_pipelines(vk::PipelineCache::null(), std::slice::from_ref(&pipeline_info), None)
            .map_err(|e| anyhow::anyhow!("Failed to create outline pipeline: {:?}", e.1))?;

        device.destroy_shader_module(vert_shader_module, None);
        device.destroy_shader_module(frag_shader_module, None);

        Ok((pipeline_layout, pipelines[0]))
    }

    unsafe fn create_shader_module(device: &ash::Device, code: &[u8]) -> Result<vk::ShaderModule> {
        let shader_module_create_info = vk::ShaderModuleCreateInfo {
            code_size: code.len(),
            p_code: code.as_ptr() as *const u32,
            ..Default::default()
        };

        Ok(device.create_shader_module(&shader_module_create_info, None)?)
    }
}
