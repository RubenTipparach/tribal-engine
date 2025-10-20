/// Skybox render pass plugin
///
/// Renders the starry background skybox

use ash::vk;
use anyhow::Result;
use crate::background::{SkyboxRenderer, SkyboxUniformBufferObject};
use crate::core::{RenderPass, RenderContext};
use crate::mesh::Mesh;
use std::ffi::CString;

const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub struct SkyboxPass {
    renderer: Option<SkyboxRenderer>,
}

impl SkyboxPass {
    pub fn new() -> Self {
        Self {
            renderer: None,
        }
    }

    unsafe fn create_descriptor_set_layout(device: &ash::Device) -> Result<vk::DescriptorSetLayout> {
        let ubo_binding = vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT);

        let bindings = [ubo_binding];
        let create_info = vk::DescriptorSetLayoutCreateInfo::default()
            .bindings(&bindings);

        Ok(device.create_descriptor_set_layout(&create_info, None)?)
    }

    unsafe fn create_pipeline(
        device: &ash::Device,
        extent: vk::Extent2D,
        render_pass: vk::RenderPass,
        descriptor_set_layout: vk::DescriptorSetLayout,
    ) -> Result<(vk::PipelineLayout, vk::Pipeline)> {
        let vert_shader_code = include_bytes!("../../../shaders/skybox.vert.spv");
        let frag_shader_code = include_bytes!("../../../shaders/skybox.frag.spv");

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

        let binding_desc = crate::mesh::Vertex::get_binding_description();
        let attribute_desc = crate::mesh::Vertex::get_attribute_descriptions();

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(std::slice::from_ref(&binding_desc))
            .vertex_attribute_descriptions(&attribute_desc);

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
            .cull_mode(vk::CullModeFlags::NONE)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .depth_bias_enable(false);

        let multisampling = vk::PipelineMultisampleStateCreateInfo::default()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default()
            .depth_test_enable(true)
            .depth_write_enable(false)
            .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL);

        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(false);

        let color_blending = vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .attachments(std::slice::from_ref(&color_blend_attachment));

        let set_layouts = [descriptor_set_layout];
        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(&set_layouts);

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

        let pipelines = device.create_graphics_pipelines(
            vk::PipelineCache::null(),
            std::slice::from_ref(&pipeline_info),
            None,
        ).map_err(|e| anyhow::anyhow!("Failed to create skybox pipeline: {:?}", e.1))?;

        device.destroy_shader_module(vert_shader_module, None);
        device.destroy_shader_module(frag_shader_module, None);

        Ok((pipeline_layout, pipelines[0]))
    }

    unsafe fn create_shader_module(device: &ash::Device, code: &[u8]) -> Result<vk::ShaderModule> {
        let code_aligned = ash::util::read_spv(&mut std::io::Cursor::new(code))?;
        let create_info = vk::ShaderModuleCreateInfo::default().code(&code_aligned);
        Ok(device.create_shader_module(&create_info, None)?)
    }

    unsafe fn create_uniform_buffers(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: &ash::Device,
    ) -> Result<(Vec<vk::Buffer>, Vec<vk::DeviceMemory>)> {
        let buffer_size = std::mem::size_of::<SkyboxUniformBufferObject>() as vk::DeviceSize;

        let mut uniform_buffers = Vec::new();
        let mut uniform_buffers_memory = Vec::new();

        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            let (buffer, memory) = Self::create_buffer(
                instance,
                physical_device,
                device,
                buffer_size,
                vk::BufferUsageFlags::UNIFORM_BUFFER,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )?;

            uniform_buffers.push(buffer);
            uniform_buffers_memory.push(memory);
        }

        Ok((uniform_buffers, uniform_buffers_memory))
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

        Err(anyhow::anyhow!("Failed to find suitable memory type"))
    }

    unsafe fn create_descriptor_pool(device: &ash::Device) -> Result<vk::DescriptorPool> {
        let pool_size = vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(MAX_FRAMES_IN_FLIGHT as u32);

        let pool_sizes = [pool_size];

        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets(MAX_FRAMES_IN_FLIGHT as u32);

        Ok(device.create_descriptor_pool(&pool_info, None)?)
    }

    unsafe fn create_descriptor_sets(
        device: &ash::Device,
        descriptor_pool: vk::DescriptorPool,
        descriptor_set_layout: vk::DescriptorSetLayout,
        uniform_buffers: &[vk::Buffer],
    ) -> Result<Vec<vk::DescriptorSet>> {
        let layouts = vec![descriptor_set_layout; MAX_FRAMES_IN_FLIGHT];

        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&layouts);

        let descriptor_sets = device.allocate_descriptor_sets(&alloc_info)?;

        for i in 0..MAX_FRAMES_IN_FLIGHT {
            let buffer_info = vk::DescriptorBufferInfo::default()
                .buffer(uniform_buffers[i])
                .offset(0)
                .range(std::mem::size_of::<SkyboxUniformBufferObject>() as vk::DeviceSize);

            let buffer_infos = [buffer_info];

            let descriptor_write = vk::WriteDescriptorSet::default()
                .dst_set(descriptor_sets[i])
                .dst_binding(0)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(&buffer_infos);

            let descriptor_writes = [descriptor_write];

            device.update_descriptor_sets(&descriptor_writes, &[]);
        }

        Ok(descriptor_sets)
    }
}

impl RenderPass for SkyboxPass {
    fn initialize(
        &mut self,
        ctx: &RenderContext,
        render_pass: vk::RenderPass,
        extent: vk::Extent2D,
    ) -> Result<()> {
        unsafe {
            // Create skybox mesh
            let mesh = Mesh::create_inverted_sphere(50.0, 32, 16);

            // Create vertex buffer
            let (vertex_buffer, vertex_buffer_memory) = Self::create_buffer(
                ctx.instance,
                ctx.physical_device,
                ctx.device,
                (std::mem::size_of::<crate::mesh::Vertex>() * mesh.vertices.len()) as vk::DeviceSize,
                vk::BufferUsageFlags::VERTEX_BUFFER,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )?;

            // Copy vertex data
            let data = ctx.device.map_memory(
                vertex_buffer_memory,
                0,
                (std::mem::size_of::<crate::mesh::Vertex>() * mesh.vertices.len()) as vk::DeviceSize,
                vk::MemoryMapFlags::empty(),
            )?;
            std::ptr::copy_nonoverlapping(
                mesh.vertices.as_ptr(),
                data as *mut crate::mesh::Vertex,
                mesh.vertices.len(),
            );
            ctx.device.unmap_memory(vertex_buffer_memory);

            // Create index buffer
            let (index_buffer, index_buffer_memory) = Self::create_buffer(
                ctx.instance,
                ctx.physical_device,
                ctx.device,
                (std::mem::size_of::<u32>() * mesh.indices.len()) as vk::DeviceSize,
                vk::BufferUsageFlags::INDEX_BUFFER,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )?;

            // Copy index data
            let data = ctx.device.map_memory(
                index_buffer_memory,
                0,
                (std::mem::size_of::<u32>() * mesh.indices.len()) as vk::DeviceSize,
                vk::MemoryMapFlags::empty(),
            )?;
            std::ptr::copy_nonoverlapping(
                mesh.indices.as_ptr(),
                data as *mut u32,
                mesh.indices.len(),
            );
            ctx.device.unmap_memory(index_buffer_memory);

            // Create descriptor set layout
            let descriptor_set_layout = Self::create_descriptor_set_layout(ctx.device)?;

            // Create pipeline
            let (pipeline_layout, pipeline) = Self::create_pipeline(
                ctx.device,
                extent,
                render_pass,
                descriptor_set_layout,
            )?;

            // Create uniform buffers
            let (uniform_buffers, uniform_buffers_memory) = Self::create_uniform_buffers(
                ctx.instance,
                ctx.physical_device,
                ctx.device,
            )?;

            // Create descriptor pool and sets
            let descriptor_pool = Self::create_descriptor_pool(ctx.device)?;
            let descriptor_sets = Self::create_descriptor_sets(
                ctx.device,
                descriptor_pool,
                descriptor_set_layout,
                &uniform_buffers,
            )?;

            self.renderer = Some(SkyboxRenderer {
                mesh,
                vertex_buffer,
                vertex_buffer_memory,
                index_buffer,
                index_buffer_memory,
                descriptor_set_layout,
                pipeline_layout,
                pipeline,
                uniform_buffers,
                uniform_buffers_memory,
                descriptor_pool,
                descriptor_sets,
            });

            Ok(())
        }
    }

    fn update(&mut self, ctx: &RenderContext, frame_index: usize, game: &crate::game::Game) -> Result<()> {
        unsafe {
            if let Some(renderer) = &self.renderer {
                let view = game.get_view_matrix();
                let aspect = ctx.extent.width as f32 / ctx.extent.height as f32;
                let proj = game.camera.projection_matrix(aspect);
                let view_pos = game.camera.position();

                let ubo = SkyboxRenderer::create_ubo(view, proj, view_pos, &game.skybox_config);

                let data = ctx.device.map_memory(
                    renderer.uniform_buffers_memory[frame_index],
                    0,
                    std::mem::size_of::<SkyboxUniformBufferObject>() as vk::DeviceSize,
                    vk::MemoryMapFlags::empty(),
                )?;

                std::ptr::copy_nonoverlapping(&ubo, data as *mut SkyboxUniformBufferObject, 1);
                ctx.device.unmap_memory(renderer.uniform_buffers_memory[frame_index]);
            }

            Ok(())
        }
    }

    fn render(
        &mut self,
        ctx: &RenderContext,
        command_buffer: vk::CommandBuffer,
        frame_index: usize,
        _game: &crate::game::Game,
    ) -> Result<()> {
        unsafe {
            if let Some(renderer) = &self.renderer {
                ctx.device.cmd_bind_pipeline(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    renderer.pipeline,
                );

                let vertex_buffers = [renderer.vertex_buffer];
                let offsets = [0];
                ctx.device.cmd_bind_vertex_buffers(command_buffer, 0, &vertex_buffers, &offsets);
                ctx.device.cmd_bind_index_buffer(command_buffer, renderer.index_buffer, 0, vk::IndexType::UINT32);

                ctx.device.cmd_bind_descriptor_sets(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    renderer.pipeline_layout,
                    0,
                    &[renderer.descriptor_sets[frame_index]],
                    &[],
                );

                ctx.device.cmd_draw_indexed(
                    command_buffer,
                    renderer.mesh.indices.len() as u32,
                    1,
                    0,
                    0,
                    0,
                );
            }

            Ok(())
        }
    }

    fn recreate_swapchain(
        &mut self,
        ctx: &RenderContext,
        render_pass: vk::RenderPass,
        extent: vk::Extent2D,
    ) -> Result<()> {
        unsafe {
            if let Some(renderer) = &mut self.renderer {
                // Destroy old pipeline
                ctx.device.destroy_pipeline(renderer.pipeline, None);
                ctx.device.destroy_pipeline_layout(renderer.pipeline_layout, None);

                // Recreate pipeline with new extent
                let (pipeline_layout, pipeline) = Self::create_pipeline(
                    ctx.device,
                    extent,
                    render_pass,
                    renderer.descriptor_set_layout,
                )?;

                renderer.pipeline_layout = pipeline_layout;
                renderer.pipeline = pipeline;
            }

            Ok(())
        }
    }

    fn cleanup(&mut self, device: &ash::Device) {
        unsafe {
            if let Some(renderer) = &self.renderer {
                renderer.cleanup(device);
            }
        }
    }

    fn name(&self) -> &str {
        "Skybox"
    }

    fn should_render(&self, _game: &crate::game::Game) -> bool {
        // Always render skybox
        true
    }
}
