use ash::vk;
use glam::{Mat4, Vec3};
use crate::mesh::Mesh;
use crate::game::SkyboxConfig;

/// Manages skybox rendering and related resources
pub struct SkyboxRenderer {
    pub mesh: Mesh,
    pub vertex_buffer: vk::Buffer,
    pub vertex_buffer_memory: vk::DeviceMemory,
    pub index_buffer: vk::Buffer,
    pub index_buffer_memory: vk::DeviceMemory,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub pipeline_layout: vk::PipelineLayout,
    pub pipeline: vk::Pipeline,
    pub uniform_buffers: Vec<vk::Buffer>,
    pub uniform_buffers_memory: Vec<vk::DeviceMemory>,
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_sets: Vec<vk::DescriptorSet>,
}

/// Uniform buffer object for skybox shader (std140 layout)
/// Matches nebula pattern - vec3 followed by scalar fills the padding slot
#[repr(C)]
#[derive(Copy, Clone)]
pub struct SkyboxUniformBufferObject {
    pub model: Mat4,
    pub view: Mat4,
    pub proj: Mat4,
    pub view_pos: Vec3,
    pub star_density: f32,           // fills vec3 padding slot
    pub star_brightness: f32,
    pub _pad0: [f32; 2],             // align next vec3 to 16 bytes
    pub nebula_primary_color: Vec3,
    pub nebula_intensity: f32,       // fills vec3 padding slot
    pub nebula_secondary_color: Vec3,
    pub background_brightness: f32,  // fills vec3 padding slot
}

unsafe impl bytemuck::Pod for SkyboxUniformBufferObject {}
unsafe impl bytemuck::Zeroable for SkyboxUniformBufferObject {}

impl SkyboxRenderer {
    /// Create uniform buffer object from config
    pub fn create_ubo(
        view: Mat4,
        proj: Mat4,
        view_pos: Vec3,
        config: &SkyboxConfig,
    ) -> SkyboxUniformBufferObject {
        SkyboxUniformBufferObject {
            model: Mat4::IDENTITY,
            view,
            proj,
            view_pos,
            star_density: config.star_density,
            star_brightness: config.star_brightness,
            _pad0: [0.0; 2],
            nebula_primary_color: config.nebula_primary_color,
            nebula_intensity: config.nebula_intensity,
            nebula_secondary_color: config.nebula_secondary_color,
            background_brightness: config.background_brightness,
        }
    }

    /// Cleanup resources
    pub unsafe fn cleanup(&self, device: &ash::Device) {
        for i in 0..self.uniform_buffers.len() {
            device.destroy_buffer(self.uniform_buffers[i], None);
            device.free_memory(self.uniform_buffers_memory[i], None);
        }
        device.destroy_descriptor_pool(self.descriptor_pool, None);
        device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
        device.destroy_pipeline(self.pipeline, None);
        device.destroy_pipeline_layout(self.pipeline_layout, None);
        device.destroy_buffer(self.index_buffer, None);
        device.free_memory(self.index_buffer_memory, None);
        device.destroy_buffer(self.vertex_buffer, None);
        device.free_memory(self.vertex_buffer_memory, None);
    }
}
