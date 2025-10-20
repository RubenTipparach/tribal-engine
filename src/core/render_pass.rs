/// Render pass plugin system for modular rendering
///
/// Each rendering feature (skybox, nebula, meshes, etc.) implements RenderPass
/// and is registered with the renderer as a plugin

use ash::vk;
use anyhow::Result;
use std::collections::HashMap;
use crate::mesh::Mesh;

/// Context provided to each render pass during initialization and rendering
pub struct RenderContext<'a> {
    pub device: &'a ash::Device,
    pub instance: &'a ash::Instance,
    pub physical_device: vk::PhysicalDevice,
    pub command_pool: vk::CommandPool,
    pub graphics_queue: vk::Queue,
    pub extent: vk::Extent2D,
    // Optional resources that some passes need
    pub depth_image_view: Option<vk::ImageView>,
    pub depth_sampler: Option<vk::Sampler>,
    // Shared mesh rendering resources (for MeshPass)
    pub mesh_pipeline: Option<vk::Pipeline>,
    pub mesh_pipeline_layout: Option<vk::PipelineLayout>,
    pub mesh_descriptor_sets: Option<&'a [vk::DescriptorSet]>,
    pub custom_meshes: Option<&'a HashMap<String, (Mesh, vk::Buffer, vk::DeviceMemory, vk::Buffer, vk::DeviceMemory)>>,
}

/// Render pass trait - each rendering system implements this
pub trait RenderPass {
    /// Initialize the render pass (create pipelines, buffers, etc.)
    fn initialize(
        &mut self,
        ctx: &RenderContext,
        render_pass: vk::RenderPass,
        extent: vk::Extent2D,
    ) -> Result<()>;

    /// Update uniforms and prepare for rendering
    fn update(&mut self, ctx: &RenderContext, frame_index: usize, game: &crate::game::Game) -> Result<()>;

    /// Record rendering commands
    fn render(
        &mut self,
        ctx: &RenderContext,
        command_buffer: vk::CommandBuffer,
        frame_index: usize,
        game: &crate::game::Game,
    ) -> Result<()>;

    /// Recreate pipelines/resources when swapchain changes
    fn recreate_swapchain(
        &mut self,
        ctx: &RenderContext,
        render_pass: vk::RenderPass,
        extent: vk::Extent2D,
    ) -> Result<()>;

    /// Cleanup resources
    fn cleanup(&mut self, device: &ash::Device);

    /// Get pass name for debugging
    fn name(&self) -> &str;

    /// Check if this pass should render this frame
    fn should_render(&self, game: &crate::game::Game) -> bool {
        let _ = game;
        true
    }
}

/// Registry of render passes - executed in order
pub struct RenderPassRegistry {
    passes: Vec<Box<dyn RenderPass>>,
}

impl RenderPassRegistry {
    pub fn new() -> Self {
        Self {
            passes: Vec::new(),
        }
    }

    /// Register a new render pass
    pub fn register(&mut self, pass: Box<dyn RenderPass>) {
        self.passes.push(pass);
    }

    /// Initialize all passes
    pub fn initialize_all(
        &mut self,
        ctx: &RenderContext,
        render_pass: vk::RenderPass,
        extent: vk::Extent2D,
    ) -> Result<()> {
        for pass in &mut self.passes {
            println!("Initializing render pass: {}", pass.name());
            pass.initialize(ctx, render_pass, extent)?;
        }
        Ok(())
    }

    /// Update all passes
    pub fn update_all(
        &mut self,
        ctx: &RenderContext,
        frame_index: usize,
        game: &crate::game::Game,
    ) -> Result<()> {
        for pass in &mut self.passes {
            if pass.should_render(game) {
                pass.update(ctx, frame_index, game)?;
            }
        }
        Ok(())
    }

    /// Render all passes
    pub fn render_all(
        &mut self,
        ctx: &RenderContext,
        command_buffer: vk::CommandBuffer,
        frame_index: usize,
        game: &crate::game::Game,
    ) -> Result<()> {
        for pass in &mut self.passes {
            if pass.should_render(game) {
                pass.render(ctx, command_buffer, frame_index, game)?;
            }
        }
        Ok(())
    }

    /// Recreate swapchain for all passes
    pub fn recreate_swapchain_all(
        &mut self,
        ctx: &RenderContext,
        render_pass: vk::RenderPass,
        extent: vk::Extent2D,
    ) -> Result<()> {
        for pass in &mut self.passes {
            pass.recreate_swapchain(ctx, render_pass, extent)?;
        }
        Ok(())
    }

    /// Cleanup all passes
    pub fn cleanup_all(&mut self, device: &ash::Device) {
        for pass in &mut self.passes {
            println!("Cleaning up render pass: {}", pass.name());
            pass.cleanup(device);
        }
    }
}
