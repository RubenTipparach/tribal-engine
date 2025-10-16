use ash::vk;
use imgui::{Context, DrawCmd, DrawCmdParams, DrawData, DrawVert, TextureId};
use std::mem::size_of;

pub struct ImGuiRenderer {
    vertex_buffer: Option<vk::Buffer>,
    vertex_buffer_memory: Option<vk::DeviceMemory>,
    vertex_buffer_size: vk::DeviceSize,
    index_buffer: Option<vk::Buffer>,
    index_buffer_memory: Option<vk::DeviceMemory>,
    index_buffer_size: vk::DeviceSize,
    descriptor_set_layout: vk::DescriptorSetLayout,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    font_texture: vk::Image,
    font_texture_memory: vk::DeviceMemory,
    font_texture_view: vk::ImageView,
    font_sampler: vk::Sampler,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set: vk::DescriptorSet,
}

impl ImGuiRenderer {
    pub fn new(
        context: &mut Context,
        device: &ash::Device,
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        render_pass: vk::RenderPass,
        command_pool: vk::CommandPool,
        graphics_queue: vk::Queue,
        swapchain_extent: vk::Extent2D,
    ) -> anyhow::Result<Self> {
        // Build font atlas
        let mut fonts = context.fonts();
        let font_texture_data = fonts.build_rgba32_texture();

        unsafe {
            // Create font texture
            let (font_texture, font_texture_memory) = Self::create_font_texture(
                device,
                instance,
                physical_device,
                command_pool,
                graphics_queue,
                font_texture_data.width,
                font_texture_data.height,
                font_texture_data.data,
            )?;

            let font_texture_view = Self::create_texture_view(device, font_texture)?;
            let font_sampler = Self::create_sampler(device)?;

            // Create descriptor set layout
            let descriptor_set_layout = Self::create_descriptor_set_layout(device)?;

            // Create descriptor pool and set
            let descriptor_pool = Self::create_descriptor_pool(device)?;
            let descriptor_set = Self::allocate_descriptor_set(device, descriptor_pool, descriptor_set_layout)?;

            // Update descriptor set with font texture
            Self::update_descriptor_set(device, descriptor_set, font_texture_view, font_sampler)?;

            // Create pipeline
            let (pipeline_layout, pipeline) = Self::create_pipeline(
                device,
                render_pass,
                descriptor_set_layout,
                swapchain_extent,
            )?;

            fonts.tex_id = TextureId::from(1);

            Ok(Self {
                vertex_buffer: None,
                vertex_buffer_memory: None,
                vertex_buffer_size: 0,
                index_buffer: None,
                index_buffer_memory: None,
                index_buffer_size: 0,
                descriptor_set_layout,
                pipeline_layout,
                pipeline,
                font_texture,
                font_texture_memory,
                font_texture_view,
                font_sampler,
                descriptor_pool,
                descriptor_set,
            })
        }
    }


    unsafe fn create_font_texture(
        device: &ash::Device,
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        command_pool: vk::CommandPool,
        graphics_queue: vk::Queue,
        width: u32,
        height: u32,
        data: &[u8],
    ) -> anyhow::Result<(vk::Image, vk::DeviceMemory)> {
        let image_size = (width * height * 4) as vk::DeviceSize;

        // Create staging buffer
        let staging_buffer_info = vk::BufferCreateInfo::default()
            .size(image_size)
            .usage(vk::BufferUsageFlags::TRANSFER_SRC)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let staging_buffer = device.create_buffer(&staging_buffer_info, None)?;
        let mem_requirements = device.get_buffer_memory_requirements(staging_buffer);

        let mem_type_index = Self::find_memory_type(
            instance,
            physical_device,
            mem_requirements.memory_type_bits,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;

        let alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(mem_requirements.size)
            .memory_type_index(mem_type_index);

        let staging_buffer_memory = device.allocate_memory(&alloc_info, None)?;
        device.bind_buffer_memory(staging_buffer, staging_buffer_memory, 0)?;

        // Copy data to staging buffer
        let ptr = device.map_memory(staging_buffer_memory, 0, image_size, vk::MemoryMapFlags::empty())?;
        std::ptr::copy_nonoverlapping(data.as_ptr(), ptr as *mut u8, data.len());
        device.unmap_memory(staging_buffer_memory);

        // Create image
        let image_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(vk::Extent3D { width, height, depth: 1 })
            .mip_levels(1)
            .array_layers(1)
            .format(vk::Format::R8G8B8A8_UNORM)
            .tiling(vk::ImageTiling::OPTIMAL)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .samples(vk::SampleCountFlags::TYPE_1);

        let image = device.create_image(&image_info, None)?;
        let mem_requirements = device.get_image_memory_requirements(image);

        let mem_type_index = Self::find_memory_type(
            instance,
            physical_device,
            mem_requirements.memory_type_bits,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        let alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(mem_requirements.size)
            .memory_type_index(mem_type_index);

        let image_memory = device.allocate_memory(&alloc_info, None)?;
        device.bind_image_memory(image, image_memory, 0)?;

        // Transition image layout and copy buffer to image
        Self::transition_image_layout(device, command_pool, graphics_queue, image, vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL)?;
        Self::copy_buffer_to_image(device, command_pool, graphics_queue, staging_buffer, image, width, height)?;
        Self::transition_image_layout(device, command_pool, graphics_queue, image, vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)?;

        device.destroy_buffer(staging_buffer, None);
        device.free_memory(staging_buffer_memory, None);

        Ok((image, image_memory))
    }

    unsafe fn create_texture_view(device: &ash::Device, image: vk::Image) -> anyhow::Result<vk::ImageView> {
        let view_info = vk::ImageViewCreateInfo::default()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(vk::Format::R8G8B8A8_UNORM)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });

        Ok(device.create_image_view(&view_info, None)?)
    }

    unsafe fn create_sampler(device: &ash::Device) -> anyhow::Result<vk::Sampler> {
        let sampler_info = vk::SamplerCreateInfo::default()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::REPEAT)
            .address_mode_v(vk::SamplerAddressMode::REPEAT)
            .address_mode_w(vk::SamplerAddressMode::REPEAT)
            .anisotropy_enable(false)
            .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
            .unnormalized_coordinates(false)
            .compare_enable(false)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR);

        Ok(device.create_sampler(&sampler_info, None)?)
    }

    unsafe fn create_descriptor_set_layout(device: &ash::Device) -> anyhow::Result<vk::DescriptorSetLayout> {
        let binding = vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT);

        let layout_info = vk::DescriptorSetLayoutCreateInfo::default()
            .bindings(std::slice::from_ref(&binding));

        Ok(device.create_descriptor_set_layout(&layout_info, None)?)
    }

    unsafe fn create_descriptor_pool(device: &ash::Device) -> anyhow::Result<vk::DescriptorPool> {
        let pool_size = vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1);

        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(std::slice::from_ref(&pool_size))
            .max_sets(1);

        Ok(device.create_descriptor_pool(&pool_info, None)?)
    }

    unsafe fn allocate_descriptor_set(
        device: &ash::Device,
        pool: vk::DescriptorPool,
        layout: vk::DescriptorSetLayout,
    ) -> anyhow::Result<vk::DescriptorSet> {
        let layouts = [layout];
        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(pool)
            .set_layouts(&layouts);

        let sets = device.allocate_descriptor_sets(&alloc_info)?;
        Ok(sets[0])
    }

    unsafe fn update_descriptor_set(
        device: &ash::Device,
        descriptor_set: vk::DescriptorSet,
        image_view: vk::ImageView,
        sampler: vk::Sampler,
    ) -> anyhow::Result<()> {
        let image_info = vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(image_view)
            .sampler(sampler);

        let write = vk::WriteDescriptorSet::default()
            .dst_set(descriptor_set)
            .dst_binding(0)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(std::slice::from_ref(&image_info));

        device.update_descriptor_sets(std::slice::from_ref(&write), &[]);
        Ok(())
    }

    unsafe fn create_pipeline(
        device: &ash::Device,
        render_pass: vk::RenderPass,
        descriptor_set_layout: vk::DescriptorSetLayout,
        extent: vk::Extent2D,
    ) -> anyhow::Result<(vk::PipelineLayout, vk::Pipeline)> {
        // For now, we'll create a simple placeholder pipeline
        // In a full implementation, you'd need to create proper ImGui shaders
        let vert_shader_code = include_bytes!("../shaders/imgui.vert.spv");
        let frag_shader_code = include_bytes!("../shaders/imgui.frag.spv");

        let vert_module = Self::create_shader_module(device, vert_shader_code)?;
        let frag_module = Self::create_shader_module(device, frag_shader_code)?;

        let entry_name = std::ffi::CString::new("main")?;

        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vert_module)
                .name(&entry_name),
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(frag_module)
                .name(&entry_name),
        ];

        // Vertex input description for ImGui vertices
        let binding_desc = vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(size_of::<DrawVert>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX);

        let attribute_descs = [
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(0),
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(1)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(8),
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(2)
                .format(vk::Format::R8G8B8A8_UNORM)
                .offset(16),
        ];

        let vertex_input = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(std::slice::from_ref(&binding_desc))
            .vertex_attribute_descriptions(&attribute_descs);

        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

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
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::NONE)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE);

        let multisampling = vk::PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD)
            .color_write_mask(vk::ColorComponentFlags::RGBA);

        let color_blending = vk::PipelineColorBlendStateCreateInfo::default()
            .attachments(std::slice::from_ref(&color_blend_attachment));

        let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default()
            .depth_test_enable(false)
            .depth_write_enable(false);

        // Push constants for projection matrix
        let push_constant = vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .offset(0)
            .size(16 * size_of::<f32>() as u32);

        let layout_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(std::slice::from_ref(&descriptor_set_layout))
            .push_constant_ranges(std::slice::from_ref(&push_constant));

        let pipeline_layout = device.create_pipeline_layout(&layout_info, None)?;

        let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterizer)
            .multisample_state(&multisampling)
            .color_blend_state(&color_blending)
            .depth_stencil_state(&depth_stencil)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0);

        let pipelines = device.create_graphics_pipelines(
            vk::PipelineCache::null(),
            std::slice::from_ref(&pipeline_info),
            None,
        ).map_err(|e| anyhow::anyhow!("Failed to create ImGui pipeline: {:?}", e.1))?;

        device.destroy_shader_module(vert_module, None);
        device.destroy_shader_module(frag_module, None);

        Ok((pipeline_layout, pipelines[0]))
    }

    unsafe fn create_shader_module(device: &ash::Device, code: &[u8]) -> anyhow::Result<vk::ShaderModule> {
        let code_aligned = ash::util::read_spv(&mut std::io::Cursor::new(code))?;
        let create_info = vk::ShaderModuleCreateInfo::default().code(&code_aligned);
        Ok(device.create_shader_module(&create_info, None)?)
    }

    pub unsafe fn render(
        &mut self,
        device: &ash::Device,
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        command_buffer: vk::CommandBuffer,
        command_pool: vk::CommandPool,
        graphics_queue: vk::Queue,
        draw_data: &DrawData,
    ) -> anyhow::Result<()> {
        if draw_data.total_vtx_count == 0 {
            return Ok(());
        }

        // Create/resize vertex buffer
        let vertex_size = (draw_data.total_vtx_count as usize * size_of::<DrawVert>()) as vk::DeviceSize;
        if self.vertex_buffer.is_none() || self.vertex_buffer_size < vertex_size {
            if let Some(buffer) = self.vertex_buffer {
                device.destroy_buffer(buffer, None);
            }
            if let Some(memory) = self.vertex_buffer_memory {
                device.free_memory(memory, None);
            }

            let (buffer, memory) = Self::create_buffer(
                device,
                instance,
                physical_device,
                vertex_size,
                vk::BufferUsageFlags::VERTEX_BUFFER,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )?;

            self.vertex_buffer = Some(buffer);
            self.vertex_buffer_memory = Some(memory);
            self.vertex_buffer_size = vertex_size;
        }

        // Create/resize index buffer
        let index_size = (draw_data.total_idx_count as usize * size_of::<u16>()) as vk::DeviceSize;
        if self.index_buffer.is_none() || self.index_buffer_size < index_size {
            if let Some(buffer) = self.index_buffer {
                device.destroy_buffer(buffer, None);
            }
            if let Some(memory) = self.index_buffer_memory {
                device.free_memory(memory, None);
            }

            let (buffer, memory) = Self::create_buffer(
                device,
                instance,
                physical_device,
                index_size,
                vk::BufferUsageFlags::INDEX_BUFFER,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )?;

            self.index_buffer = Some(buffer);
            self.index_buffer_memory = Some(memory);
            self.index_buffer_size = index_size;
        }

        // Upload vertex/index data
        let vtx_dst = device.map_memory(
            self.vertex_buffer_memory.unwrap(),
            0,
            vertex_size,
            vk::MemoryMapFlags::empty(),
        )? as *mut DrawVert;

        let idx_dst = device.map_memory(
            self.index_buffer_memory.unwrap(),
            0,
            index_size,
            vk::MemoryMapFlags::empty(),
        )? as *mut u16;

        let mut vtx_offset = 0;
        let mut idx_offset = 0;

        for draw_list in draw_data.draw_lists() {
            let vtx_buffer = draw_list.vtx_buffer();
            let idx_buffer = draw_list.idx_buffer();

            std::ptr::copy_nonoverlapping(
                vtx_buffer.as_ptr(),
                vtx_dst.add(vtx_offset),
                vtx_buffer.len(),
            );

            std::ptr::copy_nonoverlapping(
                idx_buffer.as_ptr(),
                idx_dst.add(idx_offset),
                idx_buffer.len(),
            );

            vtx_offset += vtx_buffer.len();
            idx_offset += idx_buffer.len();
        }

        device.unmap_memory(self.vertex_buffer_memory.unwrap());
        device.unmap_memory(self.index_buffer_memory.unwrap());

        // Bind pipeline
        device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline);

        // Bind vertex and index buffers
        device.cmd_bind_vertex_buffers(
            command_buffer,
            0,
            &[self.vertex_buffer.unwrap()],
            &[0],
        );
        device.cmd_bind_index_buffer(
            command_buffer,
            self.index_buffer.unwrap(),
            0,
            vk::IndexType::UINT16,
        );

        // Setup viewport
        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: draw_data.display_size[0],
            height: draw_data.display_size[1],
            min_depth: 0.0,
            max_depth: 1.0,
        };
        device.cmd_set_viewport(command_buffer, 0, &[viewport]);

        // Setup projection matrix
        let scale = [
            2.0 / draw_data.display_size[0],
            2.0 / draw_data.display_size[1],
        ];
        let translate = [
            -1.0 - draw_data.display_pos[0] * scale[0],
            -1.0 - draw_data.display_pos[1] * scale[1],
        ];

        #[rustfmt::skip]
        let projection: [[f32; 4]; 4] = [
            [scale[0], 0.0,      0.0, 0.0],
            [0.0,      scale[1], 0.0, 0.0],
            [0.0,      0.0,      1.0, 0.0],
            [translate[0], translate[1], 0.0, 1.0],
        ];

        device.cmd_push_constants(
            command_buffer,
            self.pipeline_layout,
            vk::ShaderStageFlags::VERTEX,
            0,
            std::slice::from_raw_parts(projection.as_ptr() as *const u8, 64),
        );

        // Bind descriptor set
        device.cmd_bind_descriptor_sets(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.pipeline_layout,
            0,
            &[self.descriptor_set],
            &[],
        );

        // Render command lists
        let mut vtx_offset = 0;
        let mut idx_offset = 0;

        for draw_list in draw_data.draw_lists() {
            for cmd in draw_list.commands() {
                match cmd {
                    DrawCmd::Elements { count, cmd_params } => {
                        let clip_rect = cmd_params.clip_rect;
                        let scissor = vk::Rect2D {
                            offset: vk::Offset2D {
                                x: (clip_rect[0] - draw_data.display_pos[0]).max(0.0) as i32,
                                y: (clip_rect[1] - draw_data.display_pos[1]).max(0.0) as i32,
                            },
                            extent: vk::Extent2D {
                                width: (clip_rect[2] - clip_rect[0]).abs() as u32,
                                height: (clip_rect[3] - clip_rect[1]).abs() as u32,
                            },
                        };

                        device.cmd_set_scissor(command_buffer, 0, &[scissor]);
                        device.cmd_draw_indexed(
                            command_buffer,
                            count as u32,
                            1,
                            (idx_offset + cmd_params.idx_offset) as u32,
                            (vtx_offset + cmd_params.vtx_offset) as i32,
                            0,
                        );
                    }
                    DrawCmd::ResetRenderState => {}
                    DrawCmd::RawCallback { .. } => {}
                }
            }

            vtx_offset += draw_list.vtx_buffer().len();
            idx_offset += draw_list.idx_buffer().len();
        }

        Ok(())
    }

    unsafe fn create_buffer(
        device: &ash::Device,
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        size: vk::DeviceSize,
        usage: vk::BufferUsageFlags,
        properties: vk::MemoryPropertyFlags,
    ) -> anyhow::Result<(vk::Buffer, vk::DeviceMemory)> {
        let buffer_info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = device.create_buffer(&buffer_info, None)?;
        let mem_requirements = device.get_buffer_memory_requirements(buffer);

        let mem_type_index = Self::find_memory_type(
            instance,
            physical_device,
            mem_requirements.memory_type_bits,
            properties,
        )?;

        let alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(mem_requirements.size)
            .memory_type_index(mem_type_index);

        let memory = device.allocate_memory(&alloc_info, None)?;
        device.bind_buffer_memory(buffer, memory, 0)?;

        Ok((buffer, memory))
    }

    unsafe fn find_memory_type(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        type_filter: u32,
        properties: vk::MemoryPropertyFlags,
    ) -> anyhow::Result<u32> {
        let mem_properties = instance.get_physical_device_memory_properties(physical_device);

        for i in 0..mem_properties.memory_type_count {
            if (type_filter & (1 << i)) != 0
                && mem_properties.memory_types[i as usize]
                    .property_flags
                    .contains(properties)
            {
                return Ok(i);
            }
        }

        anyhow::bail!("Failed to find suitable memory type")
    }

    unsafe fn transition_image_layout(
        device: &ash::Device,
        command_pool: vk::CommandPool,
        graphics_queue: vk::Queue,
        image: vk::Image,
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
    ) -> anyhow::Result<()> {
        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_pool(command_pool)
            .command_buffer_count(1);

        let command_buffers = device.allocate_command_buffers(&alloc_info)?;
        let command_buffer = command_buffers[0];

        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        device.begin_command_buffer(command_buffer, &begin_info)?;

        let (src_access_mask, dst_access_mask, src_stage, dst_stage) =
            match (old_layout, new_layout) {
                (vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL) => (
                    vk::AccessFlags::empty(),
                    vk::AccessFlags::TRANSFER_WRITE,
                    vk::PipelineStageFlags::TOP_OF_PIPE,
                    vk::PipelineStageFlags::TRANSFER,
                ),
                (vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL) => (
                    vk::AccessFlags::TRANSFER_WRITE,
                    vk::AccessFlags::SHADER_READ,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::FRAGMENT_SHADER,
                ),
                _ => anyhow::bail!("Unsupported layout transition"),
            };

        let barrier = vk::ImageMemoryBarrier::default()
            .old_layout(old_layout)
            .new_layout(new_layout)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(image)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            })
            .src_access_mask(src_access_mask)
            .dst_access_mask(dst_access_mask);

        device.cmd_pipeline_barrier(
            command_buffer,
            src_stage,
            dst_stage,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            std::slice::from_ref(&barrier),
        );

        device.end_command_buffer(command_buffer)?;

        let submit_info = vk::SubmitInfo::default()
            .command_buffers(std::slice::from_ref(&command_buffer));

        device.queue_submit(graphics_queue, std::slice::from_ref(&submit_info), vk::Fence::null())?;
        device.queue_wait_idle(graphics_queue)?;

        device.free_command_buffers(command_pool, &command_buffers);

        Ok(())
    }

    unsafe fn copy_buffer_to_image(
        device: &ash::Device,
        command_pool: vk::CommandPool,
        graphics_queue: vk::Queue,
        buffer: vk::Buffer,
        image: vk::Image,
        width: u32,
        height: u32,
    ) -> anyhow::Result<()> {
        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_pool(command_pool)
            .command_buffer_count(1);

        let command_buffers = device.allocate_command_buffers(&alloc_info)?;
        let command_buffer = command_buffers[0];

        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        device.begin_command_buffer(command_buffer, &begin_info)?;

        let region = vk::BufferImageCopy::default()
            .buffer_offset(0)
            .buffer_row_length(0)
            .buffer_image_height(0)
            .image_subresource(vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: 1,
            })
            .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
            .image_extent(vk::Extent3D { width, height, depth: 1 });

        device.cmd_copy_buffer_to_image(
            command_buffer,
            buffer,
            image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            std::slice::from_ref(&region),
        );

        device.end_command_buffer(command_buffer)?;

        let submit_info = vk::SubmitInfo::default()
            .command_buffers(std::slice::from_ref(&command_buffer));

        device.queue_submit(graphics_queue, std::slice::from_ref(&submit_info), vk::Fence::null())?;
        device.queue_wait_idle(graphics_queue)?;

        device.free_command_buffers(command_pool, &command_buffers);

        Ok(())
    }

    pub unsafe fn recreate_pipeline(
        &mut self,
        device: &ash::Device,
        render_pass: vk::RenderPass,
        swapchain_extent: vk::Extent2D,
    ) -> anyhow::Result<()> {
        // Destroy old pipeline
        device.destroy_pipeline(self.pipeline, None);
        device.destroy_pipeline_layout(self.pipeline_layout, None);

        // Create new pipeline with updated extent
        let (pipeline_layout, pipeline) = Self::create_pipeline(
            device,
            render_pass,
            self.descriptor_set_layout,
            swapchain_extent,
        )?;

        self.pipeline_layout = pipeline_layout;
        self.pipeline = pipeline;

        Ok(())
    }

    pub unsafe fn cleanup(&mut self, device: &ash::Device) {
        if let Some(buffer) = self.vertex_buffer {
            device.destroy_buffer(buffer, None);
        }
        if let Some(memory) = self.vertex_buffer_memory {
            device.free_memory(memory, None);
        }
        if let Some(buffer) = self.index_buffer {
            device.destroy_buffer(buffer, None);
        }
        if let Some(memory) = self.index_buffer_memory {
            device.free_memory(memory, None);
        }

        device.destroy_pipeline(self.pipeline, None);
        device.destroy_pipeline_layout(self.pipeline_layout, None);
        device.destroy_descriptor_pool(self.descriptor_pool, None);
        device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
        device.destroy_sampler(self.font_sampler, None);
        device.destroy_image_view(self.font_texture_view, None);
        device.destroy_image(self.font_texture, None);
        device.free_memory(self.font_texture_memory, None);
    }
}
