use ash::vk;
use anyhow::Result;
use glam::{Mat4, Vec3};

use crate::core::RenderPass;
use crate::mesh::Mesh;
use crate::game::Game;

/// Star shader uniform buffer object
#[repr(C)]
#[derive(Copy, Clone)]
struct StarUniformBufferObject {
    model: Mat4,
    view: Mat4,
    proj: Mat4,
    view_pos: Vec3,
    time: f32,
    star_color: Vec3,
    gamma: f32,
    scale: f32,
    exposure: f32,
    speed_hi: f32,
    speed_low: f32,
    zoom: f32,
    _padding: f32,
}

pub struct StarPass {
    // Sphere mesh for rendering stars
    sphere_mesh: Mesh,
    sphere_vertex_buffer: vk::Buffer,
    sphere_vertex_buffer_memory: vk::DeviceMemory,
    sphere_index_buffer: vk::Buffer,
    sphere_index_buffer_memory: vk::DeviceMemory,

    // Pipeline and resources
    descriptor_set_layout: vk::DescriptorSetLayout,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    uniform_buffers: Vec<vk::Buffer>,
    uniform_buffers_memory: Vec<vk::DeviceMemory>,
    descriptor_pool: vk::DescriptorPool,
    descriptor_sets: Vec<vk::DescriptorSet>,

    max_frames_in_flight: usize,
}

impl StarPass {
    pub fn new(max_frames_in_flight: usize) -> Self {
        Self {
            sphere_mesh: Mesh::create_sphere(1.0, 64, 32),
            sphere_vertex_buffer: vk::Buffer::null(),
            sphere_vertex_buffer_memory: vk::DeviceMemory::null(),
            sphere_index_buffer: vk::Buffer::null(),
            sphere_index_buffer_memory: vk::DeviceMemory::null(),
            descriptor_set_layout: vk::DescriptorSetLayout::null(),
            pipeline_layout: vk::PipelineLayout::null(),
            pipeline: vk::Pipeline::null(),
            uniform_buffers: Vec::new(),
            uniform_buffers_memory: Vec::new(),
            descriptor_pool: vk::DescriptorPool::null(),
            descriptor_sets: Vec::new(),
            max_frames_in_flight,
        }
    }

    unsafe fn create_vertex_buffer(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: &ash::Device,
        command_pool: vk::CommandPool,
        graphics_queue: vk::Queue,
        vertices: &[crate::mesh::Vertex],
    ) -> Result<(vk::Buffer, vk::DeviceMemory)> {
        let buffer_size = (std::mem::size_of::<crate::mesh::Vertex>() * vertices.len()) as vk::DeviceSize;

        // Create staging buffer
        let (staging_buffer, staging_buffer_memory) = Self::create_buffer(
            instance,
            physical_device,
            device,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;

        // Copy data to staging buffer
        let data = device.map_memory(staging_buffer_memory, 0, buffer_size, vk::MemoryMapFlags::empty())?;
        std::ptr::copy_nonoverlapping(vertices.as_ptr(), data as *mut crate::mesh::Vertex, vertices.len());
        device.unmap_memory(staging_buffer_memory);

        // Create vertex buffer
        let (vertex_buffer, vertex_buffer_memory) = Self::create_buffer(
            instance,
            physical_device,
            device,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        // Copy from staging to vertex buffer
        Self::copy_buffer(device, command_pool, graphics_queue, staging_buffer, vertex_buffer, buffer_size)?;

        device.destroy_buffer(staging_buffer, None);
        device.free_memory(staging_buffer_memory, None);

        Ok((vertex_buffer, vertex_buffer_memory))
    }

    unsafe fn create_index_buffer(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: &ash::Device,
        command_pool: vk::CommandPool,
        graphics_queue: vk::Queue,
        indices: &[u32],
    ) -> Result<(vk::Buffer, vk::DeviceMemory)> {
        let buffer_size = (std::mem::size_of::<u32>() * indices.len()) as vk::DeviceSize;

        let (staging_buffer, staging_buffer_memory) = Self::create_buffer(
            instance,
            physical_device,
            device,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;

        let data = device.map_memory(staging_buffer_memory, 0, buffer_size, vk::MemoryMapFlags::empty())?;
        std::ptr::copy_nonoverlapping(indices.as_ptr(), data as *mut u32, indices.len());
        device.unmap_memory(staging_buffer_memory);

        let (index_buffer, index_buffer_memory) = Self::create_buffer(
            instance,
            physical_device,
            device,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        Self::copy_buffer(device, command_pool, graphics_queue, staging_buffer, index_buffer, buffer_size)?;

        device.destroy_buffer(staging_buffer, None);
        device.free_memory(staging_buffer_memory, None);

        Ok((index_buffer, index_buffer_memory))
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

    unsafe fn copy_buffer(
        device: &ash::Device,
        command_pool: vk::CommandPool,
        graphics_queue: vk::Queue,
        src_buffer: vk::Buffer,
        dst_buffer: vk::Buffer,
        size: vk::DeviceSize,
    ) -> Result<()> {
        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_pool(command_pool)
            .command_buffer_count(1);

        let command_buffers = device.allocate_command_buffers(&alloc_info)?;
        let command_buffer = command_buffers[0];

        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        device.begin_command_buffer(command_buffer, &begin_info)?;

        let copy_region = vk::BufferCopy::default().size(size);
        device.cmd_copy_buffer(command_buffer, src_buffer, dst_buffer, &[copy_region]);

        device.end_command_buffer(command_buffer)?;

        let submit_info = vk::SubmitInfo::default().command_buffers(&command_buffers);

        device.queue_submit(graphics_queue, &[submit_info], vk::Fence::null())?;
        device.queue_wait_idle(graphics_queue)?;

        device.free_command_buffers(command_pool, &command_buffers);

        Ok(())
    }

    unsafe fn update_uniform_buffer(
        &self,
        device: &ash::Device,
        frame_index: usize,
        game: &Game,
        model: Mat4,
    ) -> Result<()> {
        let time = game.get_time();
        let view = game.get_view_matrix();
        let view_pos = game.get_camera_position();

        let aspect = 1920.0 / 1080.0; // Will be provided via context in future
        let proj = game.camera.projection_matrix(aspect);

        let star_color = game.star_config.color;
        let gamma = game.star_config.gamma;
        let scale = 50.0;
        let exposure = game.star_config.exposure;
        let speed_hi = game.star_config.speed_hi;
        let speed_low = game.star_config.speed_low;
        let zoom = game.star_config.zoom;

        let ubo = StarUniformBufferObject {
            model,
            view,
            proj,
            view_pos,
            time,
            star_color,
            gamma,
            scale,
            exposure,
            speed_hi,
            speed_low,
            zoom,
            _padding: 0.0,
        };

        let data = device.map_memory(
            self.uniform_buffers_memory[frame_index],
            0,
            std::mem::size_of::<StarUniformBufferObject>() as vk::DeviceSize,
            vk::MemoryMapFlags::empty(),
        )?;
        std::ptr::copy_nonoverlapping(&ubo, data as *mut StarUniformBufferObject, 1);
        device.unmap_memory(self.uniform_buffers_memory[frame_index]);

        Ok(())
    }
}

impl RenderPass for StarPass {
    fn initialize(
        &mut self,
        ctx: &crate::core::RenderContext,
        render_pass: vk::RenderPass,
        extent: vk::Extent2D,
    ) -> Result<()> {
        unsafe {
            // Create sphere buffers
            let (sphere_vb, sphere_vb_mem) = Self::create_vertex_buffer(
                ctx.instance,
                ctx.physical_device,
                ctx.device,
                ctx.command_pool,
                ctx.graphics_queue,
                &self.sphere_mesh.vertices,
            )?;
            self.sphere_vertex_buffer = sphere_vb;
            self.sphere_vertex_buffer_memory = sphere_vb_mem;

            let (sphere_ib, sphere_ib_mem) = Self::create_index_buffer(
                ctx.instance,
                ctx.physical_device,
                ctx.device,
                ctx.command_pool,
                ctx.graphics_queue,
                &self.sphere_mesh.indices,
            )?;
            self.sphere_index_buffer = sphere_ib;
            self.sphere_index_buffer_memory = sphere_ib_mem;

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

            // Create uniform buffers
            let (uniform_buffers, uniform_buffers_memory) = Self::create_uniform_buffers(
                ctx.instance,
                ctx.physical_device,
                ctx.device,
                self.max_frames_in_flight,
            )?;
            self.uniform_buffers = uniform_buffers;
            self.uniform_buffers_memory = uniform_buffers_memory;

            // Create descriptor pool and sets
            self.descriptor_pool = Self::create_descriptor_pool(ctx.device, self.max_frames_in_flight)?;
            self.descriptor_sets = Self::create_descriptor_sets(
                ctx.device,
                self.descriptor_pool,
                self.descriptor_set_layout,
                &self.uniform_buffers,
                self.max_frames_in_flight,
            )?;

            Ok(())
        }
    }

    fn update(
        &mut self,
        _ctx: &crate::core::RenderContext,
        _frame_index: usize,
        _game: &Game,
    ) -> Result<()> {
        // No per-frame updates needed outside of render
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

            let visible_spheres = game.get_visible_spheres();
            if visible_spheres.is_empty() {
                return Ok(());
            }

            // Bind star pipeline
            ctx.device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline,
            );

            // Bind sphere mesh buffers
            let vertex_buffers = [self.sphere_vertex_buffer];
            let offsets = [0_u64];
            ctx.device.cmd_bind_vertex_buffers(command_buffer, 0, &vertex_buffers, &offsets);
            ctx.device.cmd_bind_index_buffer(command_buffer, self.sphere_index_buffer, 0, vk::IndexType::UINT32);

            let indices_per_sphere = self.sphere_mesh.indices.len() as u32;

            // Render each sphere
            for model_matrix in visible_spheres.iter() {
                // Update uniform buffer for this star
                self.update_uniform_buffer(ctx.device, frame_index, game, *model_matrix)?;

                // Bind descriptor set
                ctx.device.cmd_bind_descriptor_sets(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.pipeline_layout,
                    0,
                    &[self.descriptor_sets[frame_index]],
                    &[],
                );

                // Draw star
                ctx.device.cmd_draw_indexed(command_buffer, indices_per_sphere, 1, 0, 0, 0);
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

            // Create new pipeline with new extent
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
            // Cleanup buffers
            if self.sphere_vertex_buffer != vk::Buffer::null() {
                device.destroy_buffer(self.sphere_vertex_buffer, None);
                device.free_memory(self.sphere_vertex_buffer_memory, None);
            }
            if self.sphere_index_buffer != vk::Buffer::null() {
                device.destroy_buffer(self.sphere_index_buffer, None);
                device.free_memory(self.sphere_index_buffer_memory, None);
            }

            // Cleanup pipeline
            if self.pipeline != vk::Pipeline::null() {
                device.destroy_pipeline(self.pipeline, None);
            }
            if self.pipeline_layout != vk::PipelineLayout::null() {
                device.destroy_pipeline_layout(self.pipeline_layout, None);
            }
            if self.descriptor_set_layout != vk::DescriptorSetLayout::null() {
                device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
            }

            // Cleanup descriptor pool
            if self.descriptor_pool != vk::DescriptorPool::null() {
                device.destroy_descriptor_pool(self.descriptor_pool, None);
            }

            // Cleanup uniform buffers
            for i in 0..self.uniform_buffers.len() {
                device.destroy_buffer(self.uniform_buffers[i], None);
                device.free_memory(self.uniform_buffers_memory[i], None);
            }
        }
    }

    fn name(&self) -> &str {
        "Star"
    }
}

// Static helper methods for resource creation
impl StarPass {
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

        let vert_shader_code = include_bytes!("../../../shaders/star.vert.spv");
        let frag_shader_code = include_bytes!("../../../shaders/star.frag.spv");

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
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .depth_bias_enable(false);

        let multisampling = vk::PipelineMultisampleStateCreateInfo::default()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS);

        // Additive blending for star glow
        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::ONE)
            .dst_color_blend_factor(vk::BlendFactor::ONE)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD);

        let color_blending = vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .attachments(std::slice::from_ref(&color_blend_attachment));

        let set_layouts = [descriptor_set_layout];
        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default().set_layouts(&set_layouts);

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
            .map_err(|e| anyhow::anyhow!("Failed to create star pipeline: {:?}", e.1))?;

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

    unsafe fn create_uniform_buffers(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: &ash::Device,
        max_frames_in_flight: usize,
    ) -> Result<(Vec<vk::Buffer>, Vec<vk::DeviceMemory>)> {
        let buffer_size = std::mem::size_of::<StarUniformBufferObject>() as vk::DeviceSize;

        let mut buffers = Vec::new();
        let mut buffer_memories = Vec::new();

        for _ in 0..max_frames_in_flight {
            let (buffer, buffer_memory) = Self::create_buffer(
                instance,
                physical_device,
                device,
                buffer_size,
                vk::BufferUsageFlags::UNIFORM_BUFFER,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )?;

            buffers.push(buffer);
            buffer_memories.push(buffer_memory);
        }

        Ok((buffers, buffer_memories))
    }

    unsafe fn create_descriptor_pool(device: &ash::Device, max_frames_in_flight: usize) -> Result<vk::DescriptorPool> {
        let pool_size = vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(max_frames_in_flight as u32);

        let pool_sizes = [pool_size];
        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets(max_frames_in_flight as u32);

        Ok(device.create_descriptor_pool(&pool_info, None)?)
    }

    unsafe fn create_descriptor_sets(
        device: &ash::Device,
        descriptor_pool: vk::DescriptorPool,
        descriptor_set_layout: vk::DescriptorSetLayout,
        uniform_buffers: &[vk::Buffer],
        max_frames_in_flight: usize,
    ) -> Result<Vec<vk::DescriptorSet>> {
        let layouts = vec![descriptor_set_layout; max_frames_in_flight];
        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&layouts);

        let descriptor_sets = device.allocate_descriptor_sets(&alloc_info)?;

        for i in 0..max_frames_in_flight {
            let buffer_info = vk::DescriptorBufferInfo::default()
                .buffer(uniform_buffers[i])
                .offset(0)
                .range(std::mem::size_of::<StarUniformBufferObject>() as vk::DeviceSize);

            let descriptor_write = vk::WriteDescriptorSet::default()
                .dst_set(descriptor_sets[i])
                .dst_binding(0)
                .dst_array_element(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(std::slice::from_ref(&buffer_info));

            device.update_descriptor_sets(std::slice::from_ref(&descriptor_write), &[]);
        }

        Ok(descriptor_sets)
    }
}
