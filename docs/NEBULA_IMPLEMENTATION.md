# Nebula Shader Implementation Guide

## Overview
The nebula shader has been prepared and is ready for integration. This document outlines what's been done and what's needed to complete the implementation.

## Completed Work

### 1. Shader Files Created
- **[shaders/nebula.vert](shaders/nebula.vert)** - Fullscreen triangle vertex shader
- **[shaders/nebula.frag](shaders/nebula.frag)** - Port of "Dusty nebula 4" by Duke
  - Implements spiral noise and volumetric raymarching
  - Supports density, brightness, and zoom controls
  - Uses procedural noise (hash-based, no texture dependency)
  - Compiled to `.spv` format

### 2. Module Structure
- **[src/nebula.rs](src/nebula.rs)** - Contains:
  - `NebulaConfig` struct with zoom, density, brightness
  - `NebulaUniformBufferObject` for shader uniforms
  - `NebulaRenderer` struct (partial)

### 3. Game Integration
- `NebulaConfig` added to `Game` struct in [src/game.rs](src/game.rs)
- Module declared in [src/main.rs](src/main.rs)

## Implementation Steps

### Step 1: Complete NebulaRenderer Creation

In `src/renderer.rs`, add methods to create the nebula pipeline. You'll need:

```rust
// Add to VulkanRenderer struct
nebula: NebulaRenderer,

// Add creation function similar to skybox
unsafe fn create_nebula_pipeline(
    device: &ash::Device,
    swapchain_extent: vk::Extent2D,
    render_pass: vk::RenderPass,
    descriptor_set_layout: vk::DescriptorSetLayout,
) -> anyhow::Result<(vk::PipelineLayout, vk::Pipeline)> {
    let vert_shader_code = include_bytes!("../shaders/nebula.vert.spv");
    let frag_shader_code = include_bytes!("../shaders/nebula.frag.spv");

    // Create shader modules
    // Create pipeline layout with push constants or uniform buffer
    // Create graphics pipeline with:
    //   - Blend mode: ALPHA blending for transparency
    //   - No depth write (render after skybox, before objects)
    //   - Fullscreen quad (3 vertices, no vertex buffer)
}
```

### Step 2: Create Uniform Buffers

```rust
unsafe fn create_nebula_uniform_buffers(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    device: &ash::Device,
    count: usize,
) -> anyhow::Result<(Vec<vk::Buffer>, Vec<vk::DeviceMemory>)> {
    // Similar to skybox uniform buffer creation
    // Size: std::mem::size_of::<NebulaUniformBufferObject>()
}
```

### Step 3: Initialize in VulkanRenderer::new()

After creating the skybox, add:

```rust
// Create nebula resources
let nebula_descriptor_set_layout = Self::create_descriptor_set_layout(&device)?;
let (nebula_pipeline_layout, nebula_pipeline) =
    Self::create_nebula_pipeline(&device, swapchain_extent, render_pass, nebula_descriptor_set_layout)?;

let (nebula_uniform_buffers, nebula_uniform_buffers_memory) =
    Self::create_nebula_uniform_buffers(&instance, physical_device, &device, MAX_FRAMES_IN_FLIGHT)?;

let nebula_descriptor_pool = Self::create_descriptor_pool(&device, MAX_FRAMES_IN_FLIGHT)?;
let nebula_descriptor_sets = Self::create_descriptor_sets(
    &device,
    nebula_descriptor_pool,
    nebula_descriptor_set_layout,
    &nebula_uniform_buffers,
    MAX_FRAMES_IN_FLIGHT,
)?;

let nebula = NebulaRenderer {
    descriptor_set_layout: nebula_descriptor_set_layout,
    pipeline_layout: nebula_pipeline_layout,
    pipeline: nebula_pipeline,
    uniform_buffers: nebula_uniform_buffers,
    uniform_buffers_memory: nebula_uniform_buffers_memory,
    descriptor_pool: nebula_descriptor_pool,
    descriptor_sets: nebula_descriptor_sets,
};
```

### Step 4: Update Uniform Buffer Each Frame

In `render()` method, after updating skybox:

```rust
unsafe fn update_nebula_uniform_buffer(&mut self, image_index: usize, game: &crate::game::Game) -> anyhow::Result<()> {
    let time = game.get_time();
    let resolution = glam::Vec2::new(
        self.swapchain_extent.width as f32,
        self.swapchain_extent.height as f32,
    );
    let mouse = glam::Vec2::ZERO; // Or get from input

    let ubo = NebulaRenderer::create_ubo(time, resolution, mouse, &game.nebula_config);

    let data = self.device.map_memory(
        self.nebula.uniform_buffers_memory[image_index],
        0,
        std::mem::size_of::<NebulaUniformBufferObject>() as vk::DeviceSize,
        vk::MemoryMapFlags::empty(),
    )?;
    std::ptr::copy_nonoverlapping(&ubo, data as *mut NebulaUniformBufferObject, 1);
    self.device.unmap_memory(self.nebula.uniform_buffers_memory[image_index]);

    Ok(())
}
```

### Step 5: Render Nebula in Command Buffer

In `record_command_buffer()`, add after skybox rendering but before mesh:

```rust
// 2. Render nebula (after skybox, before objects)
self.device.cmd_bind_pipeline(
    command_buffer,
    vk::PipelineBindPoint::GRAPHICS,
    self.nebula.pipeline,
);

self.device.cmd_bind_descriptor_sets(
    command_buffer,
    vk::PipelineBindPoint::GRAPHICS,
    self.nebula.pipeline_layout,
    0,
    &[self.nebula.descriptor_sets[self.current_frame]],
    &[],
);

// Draw fullscreen triangle (3 vertices, 1 instance)
self.device.cmd_draw(command_buffer, 3, 1, 0, 0);
```

### Step 6: Cleanup Resources

In `Drop` implementation:

```rust
// Cleanup nebula resources
self.nebula.cleanup(&self.device);
```

In `recreate_swapchain()`, you may need to recreate the nebula pipeline if viewport-dependent.

### Step 7: Add UI Controls

In `src/ui/mod.rs`, add nebula controls to the UI:

```rust
pub fn build_nebula_settings(ui: &Ui, game: &mut Game) {
    GuiPanelBuilder::new(ui, "Nebula Settings")
        .size(350.0, 200.0)
        .position(370.0, 10.0)
        .build(|content| {
            content
                .header("Nebula")
                .slider_f32("Zoom", &mut game.nebula_config.zoom, -2.0, 5.0)
                .slider_f32("Density", &mut game.nebula_config.density, 0.0, 2.0)
                .slider_f32("Brightness", &mut game.nebula_config.brightness, 0.1, 3.0);
        });
}

// Call in build_ui:
pub fn build_ui(context: &mut Context, game: &mut Game) {
    let ui = context.frame();
    Self::build_skybox_settings(&ui, game);
    Self::build_nebula_settings(&ui, game);  // Add this
}
```

## Shader Uniforms

The nebula shader expects these uniforms (already defined in NebulaUniformBufferObject):

- `time` - Animation time
- `resolution` - Screen resolution (width, height)
- `mouse` - Mouse position (for interaction, optional)
- `zoom` - Camera zoom level
- `density` - Nebula density multiplier
- `brightness` - Nebula brightness multiplier

## Rendering Order

1. **Skybox** - Furthest back, no depth write
2. **Nebula** - Volumetric raymarch, alpha blended
3. **Mesh** - Solid objects with depth testing
4. **ImGui** - UI overlay

## Pipeline Requirements

- **Topology**: TRIANGLE_LIST (3 vertices for fullscreen triangle)
- **Vertex Input**: None (generated in vertex shader)
- **Blend Mode**: Alpha blending
  ```rust
  vk::PipelineColorBlendAttachmentState {
      blend_enable: vk::TRUE,
      src_color_blend_factor: vk::BlendFactor::SRC_ALPHA,
      dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
      color_blend_op: vk::BlendOp::ADD,
      src_alpha_blend_factor: vk::BlendFactor::ONE,
      dst_alpha_blend_factor: vk::BlendFactor::ZERO,
      alpha_blend_op: vk::BlendOp::ADD,
      color_write_mask: vk::ColorComponentFlags::RGBA,
  }
  ```
- **Depth**: Enable depth test, disable depth write
- **Cull Mode**: NONE

## Performance Notes

- The shader performs 56 raymarch steps - adjust if needed
- Uses procedural noise (no texture fetches except for dithering seed)
- Alpha blending may impact performance on mobile/low-end hardware

## Testing

1. Build: `cargo build --release`
2. Run and check console for any Vulkan validation errors
3. Use UI sliders to adjust zoom, density, brightness
4. Verify nebula appears between skybox and cube

## References

- Original shader: https://www.shadertoy.com/view/MsVXWW
- License: Creative Commons Attribution-NonCommercial-ShareAlike 3.0
