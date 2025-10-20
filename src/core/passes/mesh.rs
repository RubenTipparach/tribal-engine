use ash::vk;
use anyhow::Result;
use std::ffi::CString;
use glam::{Mat4, Vec3};

use crate::core::RenderPass;
use crate::mesh::{Mesh, Vertex};
use crate::game::Game;

/// Uniform buffer object shared across all mesh rendering
#[repr(C)]
#[derive(Copy, Clone)]
struct UniformBufferObject {
    view: Mat4,
    proj: Mat4,
    view_pos: Vec3,
    _padding: f32,
    dir_light_direction: Vec3,
    _padding2: f32,
    dir_light_color: Vec3,
    dir_light_intensity: f32,
    dir_light_shadow_color: Vec3,
    star_density: f32,
    star_brightness: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
    nebula_primary_color: Vec3,
    nebula_intensity: f32,
    nebula_secondary_color: Vec3,
    background_brightness: f32,
    point_light_count: u32,
    ssao_enabled: u32,
    _padding3: [u32; 2],
}

/// Push constants for mesh rendering (model matrix + material properties)
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MeshPushConstants {
    pub model: Mat4,
    pub albedo: Vec3,
    pub metallic: f32,
    pub roughness: f32,
    pub ambient_strength: f32,
    pub gi_strength: f32,
    pub _padding: f32,
}

pub struct MeshPass {
    // Built-in meshes
    cube_mesh: Mesh,
    cube_vertex_buffer: vk::Buffer,
    cube_vertex_buffer_memory: vk::DeviceMemory,
    cube_index_buffer: vk::Buffer,
    cube_index_buffer_memory: vk::DeviceMemory,

    // Pipeline and descriptor references (borrowed from renderer)
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    descriptor_sets: Vec<vk::DescriptorSet>,
}

impl MeshPass {
    pub fn new() -> Self {
        Self {
            cube_mesh: Mesh::create_cube(),
            cube_vertex_buffer: vk::Buffer::null(),
            cube_vertex_buffer_memory: vk::DeviceMemory::null(),
            cube_index_buffer: vk::Buffer::null(),
            cube_index_buffer_memory: vk::DeviceMemory::null(),
            pipeline: vk::Pipeline::null(),
            pipeline_layout: vk::PipelineLayout::null(),
            descriptor_sets: Vec::new(),
        }
    }

    /// Set pipeline resources from renderer (called during initialization)
    pub fn set_pipeline_resources(
        &mut self,
        pipeline: vk::Pipeline,
        pipeline_layout: vk::PipelineLayout,
        descriptor_sets: Vec<vk::DescriptorSet>,
    ) {
        self.pipeline = pipeline;
        self.pipeline_layout = pipeline_layout;
        self.descriptor_sets = descriptor_sets;
    }

    unsafe fn create_vertex_buffer(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: &ash::Device,
        command_pool: vk::CommandPool,
        graphics_queue: vk::Queue,
        vertices: &[crate::mesh::Vertex],
    ) -> Result<(vk::Buffer, vk::DeviceMemory)> {
        // Implementation similar to renderer's create_vertex_buffer
        let buffer_size = (std::mem::size_of::<crate::mesh::Vertex>() * vertices.len()) as vk::DeviceSize;

        // Create staging buffer
        let (staging_buffer, staging_memory) = Self::create_buffer(
            instance,
            physical_device,
            device,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;

        // Copy data to staging buffer
        let data = device.map_memory(staging_memory, 0, buffer_size, vk::MemoryMapFlags::empty())?;
        std::ptr::copy_nonoverlapping(vertices.as_ptr(), data as *mut crate::mesh::Vertex, vertices.len());
        device.unmap_memory(staging_memory);

        // Create vertex buffer
        let (vertex_buffer, vertex_memory) = Self::create_buffer(
            instance,
            physical_device,
            device,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        // Copy from staging to vertex buffer
        Self::copy_buffer(device, command_pool, graphics_queue, staging_buffer, vertex_buffer, buffer_size)?;

        // Cleanup staging buffer
        device.destroy_buffer(staging_buffer, None);
        device.free_memory(staging_memory, None);

        Ok((vertex_buffer, vertex_memory))
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

        // Create staging buffer
        let (staging_buffer, staging_memory) = Self::create_buffer(
            instance,
            physical_device,
            device,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;

        // Copy data to staging buffer
        let data = device.map_memory(staging_memory, 0, buffer_size, vk::MemoryMapFlags::empty())?;
        std::ptr::copy_nonoverlapping(indices.as_ptr(), data as *mut u32, indices.len());
        device.unmap_memory(staging_memory);

        // Create index buffer
        let (index_buffer, index_memory) = Self::create_buffer(
            instance,
            physical_device,
            device,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        // Copy from staging to index buffer
        Self::copy_buffer(device, command_pool, graphics_queue, staging_buffer, index_buffer, buffer_size)?;

        // Cleanup staging buffer
        device.destroy_buffer(staging_buffer, None);
        device.free_memory(staging_memory, None);

        Ok((index_buffer, index_memory))
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

        let alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(mem_requirements.size)
            .memory_type_index(Self::find_memory_type(
                instance,
                physical_device,
                mem_requirements.memory_type_bits,
                properties,
            )?);

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
                && mem_properties.memory_types[i as usize].property_flags.contains(properties)
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
}

impl RenderPass for MeshPass {
    fn initialize(
        &mut self,
        ctx: &crate::core::RenderContext,
        _render_pass: vk::RenderPass,
        _extent: vk::Extent2D,
    ) -> Result<()> {
        unsafe {
            // Get pipeline resources from context
            if let (Some(pipeline), Some(pipeline_layout), Some(descriptor_sets)) =
                (ctx.mesh_pipeline, ctx.mesh_pipeline_layout, ctx.mesh_descriptor_sets) {
                self.pipeline = pipeline;
                self.pipeline_layout = pipeline_layout;
                self.descriptor_sets = descriptor_sets.to_vec();
            }

            // Create cube mesh buffers
            let (cube_vb, cube_vb_mem) = Self::create_vertex_buffer(
                ctx.instance,
                ctx.physical_device,
                ctx.device,
                ctx.command_pool,
                ctx.graphics_queue,
                &self.cube_mesh.vertices,
            )?;
            self.cube_vertex_buffer = cube_vb;
            self.cube_vertex_buffer_memory = cube_vb_mem;

            let (cube_ib, cube_ib_mem) = Self::create_index_buffer(
                ctx.instance,
                ctx.physical_device,
                ctx.device,
                ctx.command_pool,
                ctx.graphics_queue,
                &self.cube_mesh.indices,
            )?;
            self.cube_index_buffer = cube_ib;
            self.cube_index_buffer_memory = cube_ib_mem;

            Ok(())
        }
    }

    fn update(
        &mut self,
        _ctx: &crate::core::RenderContext,
        _frame_index: usize,
        _game: &Game,
    ) -> Result<()> {
        // Custom meshes are loaded by the renderer, not by MeshPass
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
                return Ok(()); // Not initialized yet
            }

            // Bind graphics pipeline
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

            // 1. Render cubes
            let visible_cubes = game.get_visible_cubes();
            if !visible_cubes.is_empty() {
                // Bind cube buffers once for all cubes
                let vertex_buffers = [self.cube_vertex_buffer];
                let offsets = [0];
                ctx.device.cmd_bind_vertex_buffers(command_buffer, 0, &vertex_buffers, &offsets);
                ctx.device.cmd_bind_index_buffer(command_buffer, self.cube_index_buffer, 0, vk::IndexType::UINT32);

                let indices_per_cube = self.cube_mesh.indices.len() as u32;

                // Render each cube with push constants
                for model_matrix in visible_cubes.iter() {
                    let push_data = MeshPushConstants {
                        model: *model_matrix,
                        albedo: game.material.albedo,
                        metallic: game.material.metallic,
                        roughness: game.material.roughness,
                        ambient_strength: game.material.ambient_strength,
                        gi_strength: game.material.gi_strength,
                        _padding: 0.0,
                    };
                    let push_constants = bytemuck::bytes_of(&push_data);
                    ctx.device.cmd_push_constants(
                        command_buffer,
                        self.pipeline_layout,
                        vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT,
                        0,
                        push_constants,
                    );

                    ctx.device.cmd_draw_indexed(command_buffer, indices_per_cube, 1, 0, 0, 0);
                }
            }

            // 2. Render custom meshes (loaded by renderer)
            let visible_meshes = game.get_visible_meshes();
            if !visible_meshes.is_empty() {
                if let Some(custom_meshes) = ctx.custom_meshes {
                    for (mesh_path, model_matrix) in visible_meshes.iter() {
                        if let Some((mesh, vertex_buffer, _vertex_memory, index_buffer, _index_memory)) = custom_meshes.get(mesh_path) {
                            // Bind this mesh's buffers
                            let vertex_buffers = [*vertex_buffer];
                            let offsets = [0];
                            ctx.device.cmd_bind_vertex_buffers(command_buffer, 0, &vertex_buffers, &offsets);
                            ctx.device.cmd_bind_index_buffer(command_buffer, *index_buffer, 0, vk::IndexType::UINT32);

                            // Push constants
                            let push_data = MeshPushConstants {
                                model: *model_matrix,
                                albedo: game.material.albedo,
                                metallic: game.material.metallic,
                                roughness: game.material.roughness,
                                ambient_strength: game.material.ambient_strength,
                                gi_strength: game.material.gi_strength,
                                _padding: 0.0,
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
            }

            // Note: Spheres (stars) are rendered by the renderer using the star shader pipeline, not here

            Ok(())
        }
    }

    fn recreate_swapchain(
        &mut self,
        ctx: &crate::core::RenderContext,
        _render_pass: vk::RenderPass,
        _extent: vk::Extent2D,
    ) -> Result<()> {
        // Update pipeline references after swapchain recreation
        if let (Some(pipeline), Some(pipeline_layout), Some(descriptor_sets)) =
            (ctx.mesh_pipeline, ctx.mesh_pipeline_layout, ctx.mesh_descriptor_sets) {
            self.pipeline = pipeline;
            self.pipeline_layout = pipeline_layout;
            self.descriptor_sets = descriptor_sets.to_vec();
        }
        Ok(())
    }

    fn cleanup(&mut self, device: &ash::Device) {
        unsafe {
            // Cleanup buffers
            if self.cube_vertex_buffer != vk::Buffer::null() {
                device.destroy_buffer(self.cube_vertex_buffer, None);
                device.free_memory(self.cube_vertex_buffer_memory, None);
            }
            if self.cube_index_buffer != vk::Buffer::null() {
                device.destroy_buffer(self.cube_index_buffer, None);
                device.free_memory(self.cube_index_buffer_memory, None);
            }

            // Custom meshes are owned and cleaned up by renderer
            // Spheres (stars) are owned and rendered by the renderer with the star shader
        }
    }

    fn name(&self) -> &str {
        "Mesh"
    }
}
