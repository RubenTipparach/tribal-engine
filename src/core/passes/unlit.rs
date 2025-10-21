use ash::vk;
use anyhow::Result;
use glam::{Mat4, Vec3, Vec4};

use crate::core::{RenderPass, RenderContext};
use crate::mesh::Mesh;
use crate::game::Game;

/// Uniform buffer object for unlit shader (camera matrices + time for animation)
#[repr(C)]
#[derive(Copy, Clone)]
struct UniformBufferObject {
    view: Mat4,
    proj: Mat4,
    view_pos: Vec3,
    time: f32,
}

/// Push constants for hologram rendering (model matrix + hologram parameters)
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct UnlitPushConstants {
    pub model: Mat4,
    pub color: Vec4,
    pub fresnel_power: f32,
    pub scanline_speed: f32,
    pub _padding: [f32; 2],
}

pub struct UnlitPass {
    // Built-in meshes
    cube_mesh: Mesh,

    // Pipeline and descriptor references
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    descriptor_set_layout: vk::DescriptorSetLayout,
    descriptor_sets: Vec<vk::DescriptorSet>,
    descriptor_pool: vk::DescriptorPool,
    uniform_buffers: Vec<vk::Buffer>,
    uniform_buffers_memory: Vec<vk::DeviceMemory>,
}

impl UnlitPass {
    pub fn new() -> Self {
        Self {
            cube_mesh: Mesh::create_cube(),
            pipeline: vk::Pipeline::null(),
            pipeline_layout: vk::PipelineLayout::null(),
            descriptor_set_layout: vk::DescriptorSetLayout::null(),
            descriptor_sets: Vec::new(),
            descriptor_pool: vk::DescriptorPool::null(),
            uniform_buffers: Vec::new(),
            uniform_buffers_memory: Vec::new(),
        }
    }

    unsafe fn create_shader_module(device: &ash::Device, code: &[u8]) -> Result<vk::ShaderModule> {
        let code_aligned = ash::util::read_spv(&mut std::io::Cursor::new(code))?;
        let create_info = vk::ShaderModuleCreateInfo::default().code(&code_aligned);
        Ok(device.create_shader_module(&create_info, None)?)
    }

    unsafe fn create_buffer(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: &ash::Device,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        properties: vk::MemoryPropertyFlags,
    ) -> Result<(vk::Buffer, vk::DeviceMemory)> {
        let buffer_info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = device.create_buffer(&buffer_info, None)?;
        let mem_requirements = device.get_buffer_memory_requirements(buffer);

        let memory_type_index = Self::find_memory_type(
            instance,
            physical_device,
            mem_requirements.memory_type_bits,
            properties,
        )?;

        let alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(mem_requirements.size)
            .memory_type_index(memory_type_index);

        let buffer_memory = device.allocate_memory(&alloc_info, None)?;
        device.bind_buffer_memory(buffer, buffer_memory, 0)?;

        Ok((buffer, buffer_memory))
    }

    unsafe fn find_memory_type(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        type_filter: u32,
        properties: vk::MemoryPropertyFlags,
    ) -> Result<u32> {
        let mem_properties = instance.get_physical_device_memory_properties(physical_device);

        for i in 0..mem_properties.memory_type_count {
            if (type_filter & (1 << i)) != 0
                && (mem_properties.memory_types[i as usize].property_flags & properties) == properties
            {
                return Ok(i);
            }
        }

        anyhow::bail!("Failed to find suitable memory type")
    }
}

impl RenderPass for UnlitPass {
    fn name(&self) -> &str {
        "Unlit"
    }

    fn initialize(
        &mut self,
        ctx: &RenderContext,
        render_pass: vk::RenderPass,
        extent: vk::Extent2D,
    ) -> Result<()> {
        unsafe {
            // Create descriptor set layout
            let ubo_binding = vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::VERTEX);

            let bindings = [ubo_binding];
            let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);

            self.descriptor_set_layout = ctx.device.create_descriptor_set_layout(&layout_info, None)?;

            // Create uniform buffers (one per frame in flight)
            let buffer_size = std::mem::size_of::<UniformBufferObject>() as vk::DeviceSize;
            for _ in 0..2 {
                let (uniform_buffer, uniform_memory) = Self::create_buffer(
                    ctx.instance,
                    ctx.physical_device,
                    ctx.device,
                    buffer_size,
                    vk::BufferUsageFlags::UNIFORM_BUFFER,
                    vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                )?;
                self.uniform_buffers.push(uniform_buffer);
                self.uniform_buffers_memory.push(uniform_memory);
            }

            // Create descriptor pool
            let pool_size = vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(2);

            let pool_info = vk::DescriptorPoolCreateInfo::default()
                .pool_sizes(std::slice::from_ref(&pool_size))
                .max_sets(2);

            self.descriptor_pool = ctx.device.create_descriptor_pool(&pool_info, None)?;

            // Create descriptor sets
            let layouts = vec![self.descriptor_set_layout; 2];
            let alloc_info = vk::DescriptorSetAllocateInfo::default()
                .descriptor_pool(self.descriptor_pool)
                .set_layouts(&layouts);

            self.descriptor_sets = ctx.device.allocate_descriptor_sets(&alloc_info)?;

            // Update descriptor sets
            for (i, &descriptor_set) in self.descriptor_sets.iter().enumerate() {
                let buffer_info = vk::DescriptorBufferInfo::default()
                    .buffer(self.uniform_buffers[i])
                    .offset(0)
                    .range(buffer_size);

                let descriptor_write = vk::WriteDescriptorSet::default()
                    .dst_set(descriptor_set)
                    .dst_binding(0)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .buffer_info(std::slice::from_ref(&buffer_info));

                ctx.device.update_descriptor_sets(&[descriptor_write], &[]);
            }

            // Create pipeline layout with push constants
            let push_constant_range = vk::PushConstantRange::default()
                .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
                .offset(0)
                .size(std::mem::size_of::<UnlitPushConstants>() as u32);

            let set_layouts = [self.descriptor_set_layout];
            let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default()
                .set_layouts(&set_layouts)
                .push_constant_ranges(std::slice::from_ref(&push_constant_range));

            self.pipeline_layout = ctx.device.create_pipeline_layout(&pipeline_layout_info, None)?;

            // Load shaders
            let vert_shader_code = std::fs::read("shaders/unlit.vert.spv")?;
            let frag_shader_code = std::fs::read("shaders/unlit.frag.spv")?;

            let vert_shader_module = Self::create_shader_module(ctx.device, &vert_shader_code)?;
            let frag_shader_module = Self::create_shader_module(ctx.device, &frag_shader_code)?;

            let entry_point = std::ffi::CStr::from_bytes_with_nul(b"main\0").unwrap();

            let vert_stage_info = vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vert_shader_module)
                .name(entry_point);

            let frag_stage_info = vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(frag_shader_module)
                .name(entry_point);

            let shader_stages = [vert_stage_info, frag_stage_info];

            // Vertex input
            let binding_description = crate::mesh::Vertex::get_binding_description();
            let attribute_descriptions = crate::mesh::Vertex::get_attribute_descriptions();

            let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default()
                .vertex_binding_descriptions(std::slice::from_ref(&binding_description))
                .vertex_attribute_descriptions(&attribute_descriptions);

            let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
                .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
                .primitive_restart_enable(false);

            let viewport_state = vk::PipelineViewportStateCreateInfo::default()
                .viewport_count(1)
                .scissor_count(1);

            let rasterizer = vk::PipelineRasterizationStateCreateInfo::default()
                .depth_clamp_enable(false)
                .rasterizer_discard_enable(false)
                .polygon_mode(vk::PolygonMode::FILL)
                .line_width(1.0)
                .cull_mode(vk::CullModeFlags::BACK)
                .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
                .depth_bias_enable(false);

            let multisampling = vk::PipelineMultisampleStateCreateInfo::default()
                .sample_shading_enable(false)
                .rasterization_samples(vk::SampleCountFlags::TYPE_1);

            let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default()
                .depth_test_enable(true)
                .depth_write_enable(false) // Don't write to depth for transparent holograms
                .depth_compare_op(vk::CompareOp::LESS)
                .depth_bounds_test_enable(false)
                .stencil_test_enable(false);

            // Enable alpha blending for hologram transparency
            let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
                .color_write_mask(vk::ColorComponentFlags::RGBA)
                .blend_enable(true)
                .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
                .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
                .color_blend_op(vk::BlendOp::ADD)
                .src_alpha_blend_factor(vk::BlendFactor::ONE)
                .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
                .alpha_blend_op(vk::BlendOp::ADD);

            let color_blending = vk::PipelineColorBlendStateCreateInfo::default()
                .logic_op_enable(false)
                .attachments(std::slice::from_ref(&color_blend_attachment));

            let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
            let dynamic_state = vk::PipelineDynamicStateCreateInfo::default()
                .dynamic_states(&dynamic_states);

            let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
                .stages(&shader_stages)
                .vertex_input_state(&vertex_input_info)
                .input_assembly_state(&input_assembly)
                .viewport_state(&viewport_state)
                .rasterization_state(&rasterizer)
                .multisample_state(&multisampling)
                .depth_stencil_state(&depth_stencil)
                .color_blend_state(&color_blending)
                .dynamic_state(&dynamic_state)
                .layout(self.pipeline_layout)
                .render_pass(render_pass)
                .subpass(0);

            self.pipeline = ctx.device
                .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
                .map_err(|(_, e)| e)?[0];

            ctx.device.destroy_shader_module(vert_shader_module, None);
            ctx.device.destroy_shader_module(frag_shader_module, None);

            Ok(())
        }
    }

    fn update(&mut self, ctx: &RenderContext, frame_index: usize, game: &Game) -> Result<()> {
        unsafe {
            // Update uniform buffer with camera data and time
            let aspect_ratio = ctx.extent.width as f32 / ctx.extent.height as f32;
            let ubo = UniformBufferObject {
                view: game.camera.view_matrix(),
                proj: game.camera.projection_matrix(aspect_ratio),
                view_pos: game.camera.position(),
                time: game.time(),
            };

            let data = ctx.device.map_memory(
                self.uniform_buffers_memory[frame_index],
                0,
                std::mem::size_of::<UniformBufferObject>() as vk::DeviceSize,
                vk::MemoryMapFlags::empty(),
            )?;
            std::ptr::copy_nonoverlapping(&ubo, data as *mut UniformBufferObject, 1);
            ctx.device.unmap_memory(self.uniform_buffers_memory[frame_index]);

            Ok(())
        }
    }

    fn render(
        &mut self,
        ctx: &RenderContext,
        command_buffer: vk::CommandBuffer,
        frame_index: usize,
        game: &Game,
    ) -> Result<()> {
        unsafe {
            ctx.device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline);

            // Bind descriptor set
            ctx.device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_layout,
                0,
                &[self.descriptor_sets[frame_index]],
                &[],
            );

            // Render unlit objects from scene
            for obj in game.scene.objects_sorted() {
                if !obj.visible {
                    continue;
                }

                if let crate::scene::ObjectType::Unlit(mesh_path) = &obj.object_type {
                    // Get mesh from custom_meshes
                    if let Some(custom_meshes) = ctx.custom_meshes {
                        if let Some((_mesh, vertex_buffer, _vertex_memory, index_buffer, _index_memory)) = custom_meshes.get(mesh_path) {
                            let model_matrix = glam::Mat4::from_scale_rotation_translation(
                                obj.transform.scale,
                                obj.transform.rotation,
                                obj.transform.position,
                            );

                            // Cyan holographic color with transparency
                            let color = glam::Vec4::new(0.0, 0.8, 1.0, 0.6);
                            let fresnel_power = 3.0; // Strong edge glow
                            let scanline_speed = 2.0; // Medium animation speed

                            let push_constants = UnlitPushConstants {
                                model: model_matrix,
                                color,
                                fresnel_power,
                                scanline_speed,
                                _padding: [0.0, 0.0],
                            };

                            ctx.device.cmd_push_constants(
                                command_buffer,
                                self.pipeline_layout,
                                vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                                0,
                                bytemuck::bytes_of(&push_constants),
                            );

                            ctx.device.cmd_bind_vertex_buffers(
                                command_buffer,
                                0,
                                &[*vertex_buffer],
                                &[0],
                            );
                            ctx.device.cmd_bind_index_buffer(
                                command_buffer,
                                *index_buffer,
                                0,
                                vk::IndexType::UINT32,
                            );

                            // Get mesh from custom_meshes to know index count
                            if let Some((mesh, _, _, _, _)) = custom_meshes.get(mesh_path) {
                                ctx.device.cmd_draw_indexed(
                                    command_buffer,
                                    mesh.indices.len() as u32,
                                    1,
                                    0,
                                    0,
                                    0,
                                );
                            }
                        }
                    }
                }
            }

            // Render hologram ship for movement planning (in play mode)
            if game.game_manager.mode == crate::game_manager::GameMode::Play {
                if let Some(hologram_pos) = game.hologram_ship_position {
                    // Get the ship's mesh and rotation from ECS
                    if let Some(fed_entity) = game.fed_cruiser_entity {
                        if let Ok(mut query) = game.ecs_world.world.query_one::<(&crate::ecs::components::Rotation, &crate::ecs::components::Scale)>(fed_entity) {
                            if let Some((rotation, scale)) = query.get() {
                                if let Some(custom_meshes) = ctx.custom_meshes {
                                    let mesh_path = "content/models/Fed_cruiser_ship.obj";
                                    if let Some((_mesh, vertex_buffer, _vertex_memory, index_buffer, _index_memory)) = custom_meshes.get(mesh_path) {
                                        // Create model matrix for hologram at planned position
                                        let position = hologram_pos.as_vec3();
                                        let rotation_quat = glam::Quat::from_xyzw(
                                            rotation.0.x as f32,
                                            rotation.0.y as f32,
                                            rotation.0.z as f32,
                                            rotation.0.w as f32,
                                        );
                                        let scale_vec = glam::Vec3::new(scale.0.x as f32, scale.0.y as f32, scale.0.z as f32);

                                        let model_matrix = glam::Mat4::from_scale_rotation_translation(
                                            scale_vec,
                                            rotation_quat,
                                            position,
                                        );

                                        // Change color when hovering (yellow) vs not hovering (cyan)
                                        let color = if game.hovering_hologram {
                                            glam::Vec4::new(1.0, 1.0, 0.0, 0.7) // Yellow when hovering
                                        } else {
                                            glam::Vec4::new(0.0, 0.8, 1.0, 0.6) // Cyan normally
                                        };
                                        let fresnel_power = 3.0; // Strong edge glow
                                        let scanline_speed = 2.0; // Medium animation speed

                                        let push_constants = UnlitPushConstants {
                                            model: model_matrix,
                                            color,
                                            fresnel_power,
                                            scanline_speed,
                                            _padding: [0.0, 0.0],
                                        };

                                        ctx.device.cmd_push_constants(
                                            command_buffer,
                                            self.pipeline_layout,
                                            vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                                            0,
                                            bytemuck::bytes_of(&push_constants),
                                        );

                                        ctx.device.cmd_bind_vertex_buffers(
                                            command_buffer,
                                            0,
                                            &[*vertex_buffer],
                                            &[0],
                                        );
                                        ctx.device.cmd_bind_index_buffer(
                                            command_buffer,
                                            *index_buffer,
                                            0,
                                            vk::IndexType::UINT32,
                                        );

                                        // Get mesh from custom_meshes to know index count
                                        if let Some((mesh, _, _, _, _)) = custom_meshes.get(mesh_path) {
                                            ctx.device.cmd_draw_indexed(
                                                command_buffer,
                                                mesh.indices.len() as u32,
                                                1,
                                                0,
                                                0,
                                                0,
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            Ok(())
        }
    }

    fn recreate_swapchain(
        &mut self,
        _ctx: &RenderContext,
        _render_pass: vk::RenderPass,
        _extent: vk::Extent2D,
    ) -> Result<()> {
        Ok(())
    }

    fn cleanup(&mut self, device: &ash::Device) {
        unsafe {
            device.destroy_pipeline(self.pipeline, None);
            device.destroy_pipeline_layout(self.pipeline_layout, None);
            device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
            device.destroy_descriptor_pool(self.descriptor_pool, None);
            for buffer in &self.uniform_buffers {
                device.destroy_buffer(*buffer, None);
            }
            for memory in &self.uniform_buffers_memory {
                device.free_memory(*memory, None);
            }
        }
    }
}
