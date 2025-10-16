use ash::vk;
use glam::{Mat4, Vec2, Vec3};

/// Nebula configuration
#[derive(Clone, Copy)]
pub struct NebulaConfig {
    pub zoom: f32,
    pub density: f32,
    pub brightness: f32,
    pub scale: f32,
}

impl Default for NebulaConfig {
    fn default() -> Self {
        Self {
            zoom: 0.0,
            density: 1.0,
            brightness: 1.0,
            scale: 1.0,
        }
    }
}

/// Uniform buffer for nebula shader
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct NebulaUniformBufferObject {
    pub model: Mat4,
    pub view: Mat4,
    pub proj: Mat4,
    pub view_pos: Vec3,
    pub time: f32,
    pub resolution: Vec2,
    pub mouse: Vec2,
    pub zoom: f32,
    pub density: f32,
    pub brightness: f32,
    pub scale: f32,
}

/// Nebula renderer managing all nebula-related Vulkan resources
pub struct NebulaRenderer {
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub pipeline_layout: vk::PipelineLayout,
    pub pipeline: vk::Pipeline,
    pub uniform_buffers: Vec<vk::Buffer>,
    pub uniform_buffers_memory: Vec<vk::DeviceMemory>,
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_sets: Vec<vk::DescriptorSet>,
}

impl NebulaRenderer {
    /// Create UBO from current state
    pub fn create_ubo(
        time: f32,
        resolution: Vec2,
        mouse: Vec2,
        view: Mat4,
        proj: Mat4,
        view_pos: Vec3,
        config: &NebulaConfig,
    ) -> NebulaUniformBufferObject {
        NebulaUniformBufferObject {
            model: Mat4::IDENTITY,
            view,
            proj,
            view_pos,
            time,
            resolution,
            mouse,
            zoom: config.zoom,
            density: config.density,
            brightness: config.brightness,
            scale: config.scale,
        }
    }

    /// Cleanup Vulkan resources
    pub unsafe fn cleanup(&self, device: &ash::Device) {
        device.destroy_buffer(self.uniform_buffers[0], None);
        device.free_memory(self.uniform_buffers_memory[0], None);
        device.destroy_buffer(self.uniform_buffers[1], None);
        device.free_memory(self.uniform_buffers_memory[1], None);

        device.destroy_descriptor_pool(self.descriptor_pool, None);
        device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
        device.destroy_pipeline(self.pipeline, None);
        device.destroy_pipeline_layout(self.pipeline_layout, None);
    }
}
