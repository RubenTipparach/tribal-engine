/// Line rendering pass for debug visualization and bezier curves
///
/// Renders colored line segments for:
/// - Ship movement paths (bezier curves)
/// - Debug visualization
/// - Grid lines
/// - Selection indicators

use ash::vk;
use anyhow::Result;
use glam::{Mat4, Vec3, Vec4};
use std::ffi::CStr;

/// Push constants for line rendering
#[repr(C)]
#[derive(Copy, Clone)]
pub struct LinePushConstants {
    pub view_proj: Mat4,
    pub color: Vec4,
}

unsafe impl bytemuck::Pod for LinePushConstants {}
unsafe impl bytemuck::Zeroable for LinePushConstants {}

pub struct LinePass {
    descriptor_set_layout: vk::DescriptorSetLayout,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,

    // Dynamic line vertex buffer (updated per frame)
    vertex_buffer: vk::Buffer,
    vertex_buffer_memory: vk::DeviceMemory,
    vertex_buffer_capacity: usize,  // Maximum number of vertices

    // Cached data for rendering (updated in update() phase)
    cached_vertices: Vec<Vec3>,
    cached_draw_commands: Vec<(usize, usize, Vec4)>, // (vertex_offset, vertex_count, color)
}

impl LinePass {
    pub fn new(capacity: usize) -> Self {
        Self {
            descriptor_set_layout: vk::DescriptorSetLayout::null(),
            pipeline_layout: vk::PipelineLayout::null(),
            pipeline: vk::Pipeline::null(),
            vertex_buffer: vk::Buffer::null(),
            vertex_buffer_memory: vk::DeviceMemory::null(),
            vertex_buffer_capacity: capacity,
            cached_vertices: Vec::new(),
            cached_draw_commands: Vec::new(),
        }
    }

    /// Generate bezier curve points
    /// Returns Vec<Vec3> of points along the curve
    pub fn generate_bezier_curve(
        start: Vec3,
        control: Vec3,
        end: Vec3,
        segments: usize,
    ) -> Vec<Vec3> {
        let mut points = Vec::with_capacity(segments + 1);

        for i in 0..=segments {
            let t = i as f32 / segments as f32;
            let t2 = t * t;
            let mt = 1.0 - t;
            let mt2 = mt * mt;

            // Quadratic bezier: B(t) = (1-t)²P0 + 2(1-t)tP1 + t²P2
            let point = start * mt2 + control * (2.0 * mt * t) + end * t2;
            points.push(point);
        }

        points
    }

    /// Update vertex buffer with new line data
    /// vertices: list of line segment endpoints (pairs of vertices)
    pub unsafe fn update_lines(
        &mut self,
        device: &ash::Device,
        vertices: &[Vec3],
    ) -> Result<()> {
        if vertices.is_empty() || self.vertex_buffer == vk::Buffer::null() {
            return Ok(());
        }

        // Check capacity
        if vertices.len() > self.vertex_buffer_capacity {
            anyhow::bail!("Line vertex count ({}) exceeds capacity ({})",
                vertices.len(), self.vertex_buffer_capacity);
        }

        // Map and copy vertex data
        // Using HOST_COHERENT memory means we don't need to flush
        let data_ptr = match device.map_memory(
            self.vertex_buffer_memory,
            0,
            (vertices.len() * std::mem::size_of::<Vec3>()) as vk::DeviceSize,
            vk::MemoryMapFlags::empty(),
        ) {
            Ok(ptr) => ptr,
            Err(e) => {
                // If mapping fails (e.g., device lost), just return error gracefully
                return Err(anyhow::anyhow!("Failed to map vertex buffer memory: {:?}", e));
            }
        };

        std::ptr::copy_nonoverlapping(
            vertices.as_ptr(),
            data_ptr as *mut Vec3,
            vertices.len(),
        );

        device.unmap_memory(self.vertex_buffer_memory);

        Ok(())
    }
}

// ===== Vulkan Resource Creation =====

impl LinePass {
    unsafe fn create_descriptor_set_layout(device: &ash::Device) -> Result<vk::DescriptorSetLayout> {
        // No descriptors needed - everything goes through push constants
        let layout_info = vk::DescriptorSetLayoutCreateInfo::default();

        Ok(device.create_descriptor_set_layout(&layout_info, None)?)
    }

    unsafe fn create_pipeline_layout(
        device: &ash::Device,
        descriptor_set_layout: vk::DescriptorSetLayout,
    ) -> Result<vk::PipelineLayout> {
        let push_constant_range = vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
            .offset(0)
            .size(std::mem::size_of::<LinePushConstants>() as u32);

        let layouts = [descriptor_set_layout];
        let push_constant_ranges = [push_constant_range];

        let layout_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(&layouts)
            .push_constant_ranges(&push_constant_ranges);

        Ok(device.create_pipeline_layout(&layout_info, None)?)
    }

    unsafe fn create_pipeline(
        device: &ash::Device,
        render_pass: vk::RenderPass,
        pipeline_layout: vk::PipelineLayout,
        extent: vk::Extent2D,
    ) -> Result<vk::Pipeline> {
        // Load shaders
        let vert_code = include_bytes!("../../../shaders/line.vert.spv");
        let frag_code = include_bytes!("../../../shaders/line.frag.spv");

        let vert_module = Self::create_shader_module(device, vert_code)?;
        let frag_module = Self::create_shader_module(device, frag_code)?;

        let entry_point = CStr::from_bytes_with_nul_unchecked(b"main\0");

        let vert_stage = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vert_module)
            .name(entry_point);

        let frag_stage = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(frag_module)
            .name(entry_point);

        let stages = [vert_stage, frag_stage];

        // Vertex input: just position (Vec3)
        let binding_description = vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(std::mem::size_of::<Vec3>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX);

        let attribute_description = vk::VertexInputAttributeDescription::default()
            .binding(0)
            .location(0)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(0);

        let binding_descriptions = [binding_description];
        let attribute_descriptions = [attribute_description];

        let vertex_input = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&binding_descriptions)
            .vertex_attribute_descriptions(&attribute_descriptions);

        // Input assembly: line list
        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::LINE_LIST)
            .primitive_restart_enable(false);

        // Viewport and scissor
        let viewport = vk::Viewport::default()
            .x(0.0)
            .y(0.0)
            .width(extent.width as f32)
            .height(extent.height as f32)
            .min_depth(0.0)
            .max_depth(1.0);

        let scissor = vk::Rect2D::default()
            .offset(vk::Offset2D { x: 0, y: 0 })
            .extent(extent);

        let viewports = [viewport];
        let scissors = [scissor];

        let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .viewports(&viewports)
            .scissors(&scissors);

        // Rasterization
        let rasterizer = vk::PipelineRasterizationStateCreateInfo::default()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::NONE)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .depth_bias_enable(false);

        // Multisampling
        let multisampling = vk::PipelineMultisampleStateCreateInfo::default()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        // Depth testing
        let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default()
            .depth_test_enable(true)
            .depth_write_enable(false)  // Don't write to depth buffer
            .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL)
            .depth_bounds_test_enable(false)
            .stencil_test_enable(false);

        // Color blending (no blending for opaque lines)
        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(false);

        let color_blend_attachments = [color_blend_attachment];
        let color_blending = vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .attachments(&color_blend_attachments);

        // Pipeline creation
        let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&stages)
            .vertex_input_state(&vertex_input)
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
            &[pipeline_info],
            None,
        ).map_err(|(_, e)| e)?;

        // Clean up shader modules
        device.destroy_shader_module(vert_module, None);
        device.destroy_shader_module(frag_module, None);

        Ok(pipelines[0])
    }

    unsafe fn create_shader_module(device: &ash::Device, code: &[u8]) -> Result<vk::ShaderModule> {
        let code_aligned = ash::util::read_spv(&mut std::io::Cursor::new(code))?;
        let create_info = vk::ShaderModuleCreateInfo::default().code(&code_aligned);
        Ok(device.create_shader_module(&create_info, None)?)
    }

    unsafe fn create_vertex_buffer(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: &ash::Device,
        capacity: usize,
    ) -> Result<(vk::Buffer, vk::DeviceMemory)> {
        let buffer_size = (capacity * std::mem::size_of::<Vec3>()) as vk::DeviceSize;

        // Create vertex buffer
        let buffer_info = vk::BufferCreateInfo::default()
            .size(buffer_size)
            .usage(vk::BufferUsageFlags::VERTEX_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let buffer = device.create_buffer(&buffer_info, None)?;

        let mem_requirements = device.get_buffer_memory_requirements(buffer);

        let mem_properties = instance.get_physical_device_memory_properties(physical_device);
        let memory_type_index = Self::find_memory_type(
            mem_requirements.memory_type_bits,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            &mem_properties,
        )?;

        let alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(mem_requirements.size)
            .memory_type_index(memory_type_index);

        let buffer_memory = device.allocate_memory(&alloc_info, None)?;
        device.bind_buffer_memory(buffer, buffer_memory, 0)?;

        Ok((buffer, buffer_memory))
    }

    fn find_memory_type(
        type_filter: u32,
        properties: vk::MemoryPropertyFlags,
        mem_properties: &vk::PhysicalDeviceMemoryProperties,
    ) -> Result<u32> {
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

// ===== RenderPass Trait Implementation =====

impl crate::core::RenderPass for LinePass {
    fn initialize(
        &mut self,
        ctx: &crate::core::RenderContext,
        render_pass: vk::RenderPass,
        extent: vk::Extent2D,
    ) -> Result<()> {
        unsafe {
            self.descriptor_set_layout = Self::create_descriptor_set_layout(ctx.device)?;
            self.pipeline_layout = Self::create_pipeline_layout(ctx.device, self.descriptor_set_layout)?;
            self.pipeline = Self::create_pipeline(ctx.device, render_pass, self.pipeline_layout, extent)?;

            let (vertex_buffer, vertex_buffer_memory) = Self::create_vertex_buffer(
                ctx.instance,
                ctx.physical_device,
                ctx.device,
                self.vertex_buffer_capacity,
            )?;

            self.vertex_buffer = vertex_buffer;
            self.vertex_buffer_memory = vertex_buffer_memory;

            Ok(())
        }
    }

    fn update(
        &mut self,
        ctx: &crate::core::RenderContext,
        _frame_index: usize,
        game: &crate::game::Game,
    ) -> Result<()> {
        // Only update in play mode when hologram is active
        if game.game_manager.mode != crate::game_manager::GameMode::Play {
            self.cached_vertices.clear();
            self.cached_draw_commands.clear();
            return Ok(());
        }

        if game.hologram_ship_position.is_none() {
            self.cached_vertices.clear();
            self.cached_draw_commands.clear();
            return Ok(());
        }

        // Collect all line vertices
        let mut all_vertices = Vec::new();
        let mut draw_commands = Vec::new();

        // 1. Collect bezier curve vertices
        if let Some(hologram_pos) = game.hologram_ship_position {
            if let Some(fed_entity) = game.fed_cruiser_entity {
                if let Ok(mut query) = game.ecs_world.world.query_one::<(&crate::ecs::components::Position, &crate::ecs::components::Ship)>(fed_entity) {
                    if let Some((position, ship)) = query.get() {
                        let ship_pos = position.0;

                        // Use the control point from ship (calculated for car-like arc)
                        let control_point = ship.control_point;

                        // Generate bezier curve points
                        let curve_points = Self::generate_bezier_curve(
                            ship_pos.as_vec3(),
                            control_point.as_vec3(),
                            hologram_pos.as_vec3(),
                            32,  // 32 segments for smooth curve
                        );

                        // Convert to line segments
                        let start_offset = all_vertices.len();
                        for i in 0..curve_points.len() - 1 {
                            all_vertices.push(curve_points[i]);
                            all_vertices.push(curve_points[i + 1]);
                        }
                        let vertex_count = all_vertices.len() - start_offset;

                        if vertex_count > 0 {
                            draw_commands.push((
                                start_offset,
                                vertex_count,
                                Vec4::new(0.0, 1.0, 1.0, 1.0), // Cyan
                            ));
                        }
                    }
                }
            }
        }

        // 2. Collect rotation arc boundary vertices
        let arc_vertices = game.generate_rotation_arc_boundary();
        if !arc_vertices.is_empty() {
            let start_offset = all_vertices.len();
            all_vertices.extend_from_slice(&arc_vertices);
            let vertex_count = arc_vertices.len();

            draw_commands.push((
                start_offset,
                vertex_count,
                Vec4::new(1.0, 1.0, 0.0, 0.5), // Yellow semi-transparent
            ));
        }

        // 3. Draw picking area debug visualization (sphere wireframe around hologram)
        if game.game_manager.mode == crate::game_manager::GameMode::Play {
            if let Some(hologram_pos) = game.hologram_ship_position {
                // Get scale from ECS to match picking logic
                if let Some(fed_entity) = game.fed_cruiser_entity {
                    if let Ok(mut query) = game.ecs_world.world.query_one::<&crate::ecs::components::Scale>(fed_entity) {
                        if let Some(scale_comp) = query.get() {
                            // Use same picking radius as object picker: max scale * 1.5
                            let scale = glam::Vec3::new(scale_comp.0.x as f32, scale_comp.0.y as f32, scale_comp.0.z as f32);
                            let radius = scale.x.max(scale.y).max(scale.z) * 1.5;

                            let sphere_vertices = Self::generate_wireframe_sphere(
                                hologram_pos.as_vec3(),
                                radius,
                                16, // Latitude segments
                                16, // Longitude segments
                            );

                            if !sphere_vertices.is_empty() {
                                let start_offset = all_vertices.len();
                                all_vertices.extend_from_slice(&sphere_vertices);
                                let vertex_count = sphere_vertices.len();

                                draw_commands.push((
                                    start_offset,
                                    vertex_count,
                                    Vec4::new(1.0, 0.0, 1.0, 0.8), // Magenta for debug
                                ));
                            }
                        }
                    }
                }
            }
        }

        // 4. Draw camera center cursor (appears when using WASD free camera)
        if game.show_camera_cursor {
            let cursor_pos = game.camera_cursor_position.as_vec3();
            let cursor_radius = 0.3; // Small sphere to mark the center

            let sphere_vertices = Self::generate_wireframe_sphere(
                cursor_pos,
                cursor_radius,
                8,  // Latitude segments (fewer for small cursor)
                8,  // Longitude segments
            );

            if !sphere_vertices.is_empty() {
                let start_offset = all_vertices.len();
                all_vertices.extend_from_slice(&sphere_vertices);
                let vertex_count = sphere_vertices.len();

                draw_commands.push((
                    start_offset,
                    vertex_count,
                    Vec4::new(1.0, 1.0, 0.0, 1.0), // Yellow cursor
                ));
            }
        }

        // Update vertex buffer with collected data
        if !all_vertices.is_empty() {
            unsafe {
                self.update_lines(ctx.device, &all_vertices)?;
            }
        }

        // Cache for rendering
        self.cached_vertices = all_vertices;
        self.cached_draw_commands = draw_commands;

        Ok(())
    }

    fn render(
        &mut self,
        ctx: &crate::core::RenderContext,
        command_buffer: vk::CommandBuffer,
        _frame_index: usize,
        game: &crate::game::Game,
    ) -> Result<()> {
        // Only render if we have cached data
        if self.cached_vertices.is_empty() || self.cached_draw_commands.is_empty() {
            return Ok(());
        }

        unsafe {
            ctx.device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline,
            );

            // Bind vertex buffer
            ctx.device.cmd_bind_vertex_buffers(
                command_buffer,
                0,
                &[self.vertex_buffer],
                &[0],
            );

            let view_proj = game.camera.projection_matrix(ctx.extent.width as f32 / ctx.extent.height as f32)
                * game.camera.view_matrix();

            // Execute all cached draw commands
            for &(vertex_offset, vertex_count, color) in &self.cached_draw_commands {
                let push_constants = LinePushConstants {
                    view_proj,
                    color,
                };

                let push_constants_bytes = bytemuck::bytes_of(&push_constants);
                ctx.device.cmd_push_constants(
                    command_buffer,
                    self.pipeline_layout,
                    vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                    0,
                    push_constants_bytes,
                );

                ctx.device.cmd_draw(
                    command_buffer,
                    vertex_count as u32,
                    1,
                    vertex_offset as u32,
                    0,
                );
            }
        }

        Ok(())
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
            }

            // Recreate pipeline with new extent
            self.pipeline = Self::create_pipeline(ctx.device, render_pass, self.pipeline_layout, extent)?;
        }
        Ok(())
    }

    fn cleanup(&mut self, device: &ash::Device) {
        unsafe {
            if self.vertex_buffer != vk::Buffer::null() {
                device.destroy_buffer(self.vertex_buffer, None);
            }
            if self.vertex_buffer_memory != vk::DeviceMemory::null() {
                device.free_memory(self.vertex_buffer_memory, None);
            }
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
        "LinePass"
    }

    fn should_render(&self, game: &crate::game::Game) -> bool {
        // Only render in play mode when hologram exists
        game.game_manager.mode == crate::game_manager::GameMode::Play
            && game.hologram_ship_position.is_some()
    }
}

impl LinePass {
    /// Generate wireframe sphere for debug visualization
    /// Returns line segments (pairs of vertices)
    fn generate_wireframe_sphere(center: Vec3, radius: f32, lat_segments: usize, lon_segments: usize) -> Vec<Vec3> {
        let mut vertices = Vec::new();

        // Generate latitude circles
        for lat in 0..lat_segments {
            let theta1 = std::f32::consts::PI * (lat as f32 / lat_segments as f32);

            for lon in 0..lon_segments {
                let phi1 = 2.0 * std::f32::consts::PI * (lon as f32 / lon_segments as f32);
                let phi2 = 2.0 * std::f32::consts::PI * ((lon + 1) as f32 / lon_segments as f32);

                // Current latitude circle
                let x1 = radius * theta1.sin() * phi1.cos();
                let y1 = radius * theta1.cos();
                let z1 = radius * theta1.sin() * phi1.sin();

                let x2 = radius * theta1.sin() * phi2.cos();
                let y2 = radius * theta1.cos();
                let z2 = radius * theta1.sin() * phi2.sin();

                vertices.push(center + Vec3::new(x1, y1, z1));
                vertices.push(center + Vec3::new(x2, y2, z2));
            }
        }

        // Generate longitude circles
        for lon in 0..lon_segments {
            let phi = 2.0 * std::f32::consts::PI * (lon as f32 / lon_segments as f32);

            for lat in 0..lat_segments {
                let theta1 = std::f32::consts::PI * (lat as f32 / lat_segments as f32);
                let theta2 = std::f32::consts::PI * ((lat + 1) as f32 / lat_segments as f32);

                let x1 = radius * theta1.sin() * phi.cos();
                let y1 = radius * theta1.cos();
                let z1 = radius * theta1.sin() * phi.sin();

                let x2 = radius * theta2.sin() * phi.cos();
                let y2 = radius * theta2.cos();
                let z2 = radius * theta2.sin() * phi.sin();

                vertices.push(center + Vec3::new(x1, y1, z1));
                vertices.push(center + Vec3::new(x2, y2, z2));
            }
        }

        vertices
    }
}
