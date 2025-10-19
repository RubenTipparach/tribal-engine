use ash::vk;
use glam::{Mat4, Vec2, Vec3};

/// Nebula configuration
#[derive(Clone, Copy)]
pub struct NebulaConfig {
    // Basic parameters
    pub zoom: f32,
    pub density: f32,
    pub brightness: f32,
    pub scale: f32,

    // Color parameters
    pub color_center: Vec3,
    pub color_edge: Vec3,
    pub color_density_low: Vec3,
    pub color_density_high: Vec3,

    // Light parameters
    pub light_color: Vec3,
    pub light_intensity: f32,

    // Raymarch distance
    pub max_distance: f32,
}

impl From<crate::config::NebulaConfigData> for NebulaConfig {
    fn from(data: crate::config::NebulaConfigData) -> Self {
        Self {
            zoom: data.zoom,
            density: data.density,
            brightness: data.brightness,
            scale: data.scale,
            color_center: data.color_center,
            color_edge: data.color_edge,
            color_density_low: data.color_density_low,
            color_density_high: data.color_density_high,
            light_color: data.light_color,
            light_intensity: data.light_intensity,
            max_distance: data.max_distance,
        }
    }
}

impl From<&NebulaConfig> for crate::config::NebulaConfigData {
    fn from(config: &NebulaConfig) -> Self {
        Self {
            zoom: config.zoom,
            density: config.density,
            brightness: config.brightness,
            scale: config.scale,
            color_center: config.color_center,
            color_edge: config.color_edge,
            color_density_low: config.color_density_low,
            color_density_high: config.color_density_high,
            light_color: config.light_color,
            light_intensity: config.light_intensity,
            max_distance: config.max_distance,
        }
    }
}

impl Default for NebulaConfig {
    fn default() -> Self {
        Self {
            // Basic parameters (keep existing defaults)
            zoom: 0.01,
            density: 2.0,
            brightness: 1.0,
            scale: 20.0,
            
            // Color parameters (from shader defaults)
            color_center: Vec3::new(0.8, 1.0, 1.0) * 7.0,
            color_edge: Vec3::new(0.48, 0.53, 0.5) * 1.5,
            color_density_low: Vec3::new(1.0, 0.9, 0.8),
            color_density_high: Vec3::new(0.4, 0.15, 0.1),
            
            // Light parameters (from shader)
            light_color: Vec3::new(1.0, 0.5, 0.25),
            light_intensity: 1.0 / 30.0,

            // Raymarch distance - default to 10.0 (original shader value)
            max_distance: 10.0,
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
    
    // Color parameters
    pub color_center: Vec3,
    pub _padding1: f32,
    pub color_edge: Vec3,
    pub _padding2: f32,
    pub color_density_low: Vec3,
    pub _padding3: f32,
    pub color_density_high: Vec3,
    pub _padding4: f32,
    
    // Light parameters
    pub light_color: Vec3,
    pub light_intensity: f32,

    // Raymarch distance
    pub max_distance: f32,
    pub _padding5: [f32; 3],
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
            
            // Color parameters
            color_center: config.color_center,
            _padding1: 0.0,
            color_edge: config.color_edge,
            _padding2: 0.0,
            color_density_low: config.color_density_low,
            _padding3: 0.0,
            color_density_high: config.color_density_high,
            _padding4: 0.0,
            
            // Light parameters
            light_color: config.light_color,
            light_intensity: config.light_intensity,

            // Raymarch distance
            max_distance: config.max_distance,
            _padding5: [0.0; 3],
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
