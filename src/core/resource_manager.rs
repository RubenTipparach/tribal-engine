use ash::vk;

/// Reusable resource management utilities for Vulkan buffers, images, and memory
pub struct ResourceManager;

impl ResourceManager {
    /// Create a generic buffer with the given size, usage, and memory properties
    pub unsafe fn create_buffer(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: &ash::Device,
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

    /// Find a suitable memory type for the given requirements
    pub unsafe fn find_memory_type(
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

        Err(anyhow::anyhow!("Failed to find suitable memory type"))
    }

    /// Copy data from one buffer to another
    pub unsafe fn copy_buffer(
        device: &ash::Device,
        command_pool: vk::CommandPool,
        queue: vk::Queue,
        src_buffer: vk::Buffer,
        dst_buffer: vk::Buffer,
        size: vk::DeviceSize,
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

        let copy_region = vk::BufferCopy::default().size(size);
        device.cmd_copy_buffer(command_buffer, src_buffer, dst_buffer, &[copy_region]);

        device.end_command_buffer(command_buffer)?;

        let command_buffers_slice = [command_buffer];
        let submit_info = vk::SubmitInfo::default().command_buffers(&command_buffers_slice);

        device.queue_submit(queue, &[submit_info], vk::Fence::null())?;
        device.queue_wait_idle(queue)?;

        device.free_command_buffers(command_pool, &[command_buffer]);

        Ok(())
    }

    /// Create a vertex buffer from vertex data
    pub unsafe fn create_vertex_buffer<T: Copy>(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: &ash::Device,
        command_pool: vk::CommandPool,
        queue: vk::Queue,
        vertices: &[T],
    ) -> anyhow::Result<(vk::Buffer, vk::DeviceMemory)> {
        let buffer_size = (std::mem::size_of::<T>() * vertices.len()) as vk::DeviceSize;

        // Create staging buffer
        let (staging_buffer, staging_buffer_memory) = Self::create_buffer(
            instance,
            physical_device,
            device,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;

        // Copy vertex data to staging buffer
        let data = device.map_memory(
            staging_buffer_memory,
            0,
            buffer_size,
            vk::MemoryMapFlags::empty(),
        )?;
        std::ptr::copy_nonoverlapping(vertices.as_ptr(), data as *mut T, vertices.len());
        device.unmap_memory(staging_buffer_memory);

        // Create device local buffer
        let (vertex_buffer, vertex_buffer_memory) = Self::create_buffer(
            instance,
            physical_device,
            device,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        // Copy from staging to device local
        Self::copy_buffer(
            device,
            command_pool,
            queue,
            staging_buffer,
            vertex_buffer,
            buffer_size,
        )?;

        // Cleanup staging buffer
        device.destroy_buffer(staging_buffer, None);
        device.free_memory(staging_buffer_memory, None);

        Ok((vertex_buffer, vertex_buffer_memory))
    }

    /// Create an index buffer from index data
    pub unsafe fn create_index_buffer(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: &ash::Device,
        command_pool: vk::CommandPool,
        queue: vk::Queue,
        indices: &[u32],
    ) -> anyhow::Result<(vk::Buffer, vk::DeviceMemory)> {
        let buffer_size = (std::mem::size_of::<u32>() * indices.len()) as vk::DeviceSize;

        // Create staging buffer
        let (staging_buffer, staging_buffer_memory) = Self::create_buffer(
            instance,
            physical_device,
            device,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )?;

        // Copy index data to staging buffer
        let data = device.map_memory(
            staging_buffer_memory,
            0,
            buffer_size,
            vk::MemoryMapFlags::empty(),
        )?;
        std::ptr::copy_nonoverlapping(indices.as_ptr(), data as *mut u32, indices.len());
        device.unmap_memory(staging_buffer_memory);

        // Create device local buffer
        let (index_buffer, index_buffer_memory) = Self::create_buffer(
            instance,
            physical_device,
            device,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )?;

        // Copy from staging to device local
        Self::copy_buffer(
            device,
            command_pool,
            queue,
            staging_buffer,
            index_buffer,
            buffer_size,
        )?;

        // Cleanup staging buffer
        device.destroy_buffer(staging_buffer, None);
        device.free_memory(staging_buffer_memory, None);

        Ok((index_buffer, index_buffer_memory))
    }

    /// Create uniform buffers for multiple frames in flight
    pub unsafe fn create_uniform_buffers<T>(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: &ash::Device,
        frame_count: usize,
    ) -> anyhow::Result<(Vec<vk::Buffer>, Vec<vk::DeviceMemory>)> {
        let buffer_size = std::mem::size_of::<T>() as vk::DeviceSize;

        let mut buffers = Vec::with_capacity(frame_count);
        let mut memories = Vec::with_capacity(frame_count);

        for _ in 0..frame_count {
            let (buffer, memory) = Self::create_buffer(
                instance,
                physical_device,
                device,
                buffer_size,
                vk::BufferUsageFlags::UNIFORM_BUFFER,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )?;

            buffers.push(buffer);
            memories.push(memory);
        }

        Ok((buffers, memories))
    }

    /// Create a command pool
    pub unsafe fn create_command_pool(
        device: &ash::Device,
        queue_family_index: u32,
    ) -> anyhow::Result<vk::CommandPool> {
        let pool_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(queue_family_index);

        Ok(device.create_command_pool(&pool_info, None)?)
    }

    /// Allocate command buffers
    pub unsafe fn create_command_buffers(
        device: &ash::Device,
        command_pool: vk::CommandPool,
        count: usize,
    ) -> anyhow::Result<Vec<vk::CommandBuffer>> {
        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(count as u32);

        Ok(device.allocate_command_buffers(&alloc_info)?)
    }

    /// Create synchronization objects (semaphores and fences)
    pub unsafe fn create_sync_objects(
        device: &ash::Device,
        frame_count: usize,
    ) -> anyhow::Result<(Vec<vk::Semaphore>, Vec<vk::Semaphore>, Vec<vk::Fence>)> {
        let semaphore_info = vk::SemaphoreCreateInfo::default();
        let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);

        let mut image_available = Vec::with_capacity(frame_count);
        let mut render_finished = Vec::with_capacity(frame_count);
        let mut in_flight_fences = Vec::with_capacity(frame_count);

        for _ in 0..frame_count {
            image_available.push(device.create_semaphore(&semaphore_info, None)?);
            render_finished.push(device.create_semaphore(&semaphore_info, None)?);
            in_flight_fences.push(device.create_fence(&fence_info, None)?);
        }

        Ok((image_available, render_finished, in_flight_fences))
    }

    /// Create a depth image with view
    pub unsafe fn create_depth_resources(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: &ash::Device,
        extent: vk::Extent2D,
        format: vk::Format,
        usage: vk::ImageUsageFlags,
    ) -> anyhow::Result<(vk::Image, vk::DeviceMemory, vk::ImageView)> {
        let image_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(vk::Extent3D {
                width: extent.width,
                height: extent.height,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .format(format)
            .tiling(vk::ImageTiling::OPTIMAL)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(usage)
            .samples(vk::SampleCountFlags::TYPE_1)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        let image = device.create_image(&image_info, None)?;

        let mem_requirements = device.get_image_memory_requirements(image);

        let alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(mem_requirements.size)
            .memory_type_index(Self::find_memory_type(
                instance,
                physical_device,
                mem_requirements.memory_type_bits,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
            )?);

        let image_memory = device.allocate_memory(&alloc_info, None)?;
        device.bind_image_memory(image, image_memory, 0)?;

        // Create image view
        let view_info = vk::ImageViewCreateInfo::default()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(format)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::DEPTH,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });

        let image_view = device.create_image_view(&view_info, None)?;

        Ok((image, image_memory, image_view))
    }

    /// Create a shader module from SPIR-V bytecode
    pub unsafe fn create_shader_module(
        device: &ash::Device,
        code: &[u8],
    ) -> anyhow::Result<vk::ShaderModule> {
        let code_aligned = ash::util::read_spv(&mut std::io::Cursor::new(code))?;
        let create_info = vk::ShaderModuleCreateInfo::default().code(&code_aligned);

        Ok(device.create_shader_module(&create_info, None)?)
    }
}
