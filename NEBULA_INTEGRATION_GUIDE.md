# Nebula Integration Guide

## Overview
The raymarching shaders for the SDF nebula are already compiled and ready. This guide explains how to integrate them into the rendering pipeline.

## Current Status
✅ Shaders compiled:
- `shaders/raymarch.vert.spv` - Fullscreen quad vertex shader
- `shaders/raymarch.frag.spv` - SDF nebula raymarching fragment shader

✅ Mesh rendering working perfectly
✅ PBR lighting implemented
✅ Smooth animation working

## Integration Steps

### 1. Add Nebula Uniform Buffer Structure
Already added in `renderer.rs`:
```rust
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct NebulaUniformBufferObject {
    view: Mat4,
    proj: Mat4,
    view_pos: Vec3,
    time: f32,
    octaves: u32,
    lacunarity: f32,
    gain: f32,
    frequency: f32,
    color_primary: Vec3,
    _padding1: f32,
    color_secondary: Vec3,
    _padding2: f32,
    density: f32,
    _padding3: [f32; 3],
}
```

### 2. Create Nebula Pipeline

Add a function similar to `create_graphics_pipeline` but for raymarching:

```rust
unsafe fn create_nebula_pipeline(
    device: &ash::Device,
    extent: vk::Extent2D,
    render_pass: vk::RenderPass,
    descriptor_set_layout: vk::DescriptorSetLayout,
) -> anyhow::Result<(vk::PipelineLayout, vk::Pipeline)> {
    // Load raymarch shaders
    let vert_shader_code = include_bytes!("../shaders/raymarch.vert.spv");
    let frag_shader_code = include_bytes!("../shaders/raymarch.frag.spv");

    let vert_shader_module = Self::create_shader_module(device, vert_shader_code)?;
    let frag_shader_module = Self::create_shader_module(device, frag_shader_code)?;

    // ... (similar to graphics pipeline but no vertex input)
    let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default();
    // Fullscreen triangle topology
    let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

    // ... rest similar to mesh pipeline
}
```

### 3. Render Order

The nebula should render FIRST (as background), then the mesh on top:

```rust
unsafe fn record_command_buffer(&self, command_buffer: vk::CommandBuffer, image_index: usize) -> anyhow::Result<()> {
    self.device.begin_command_buffer(command_buffer, &begin_info)?;

    self.device.cmd_begin_render_pass(...);

    // 1. Render nebula (fullscreen quad)
    self.device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, self.nebula_pipeline);
    self.device.cmd_bind_descriptor_sets(command_buffer, ..., &[self.nebula_descriptor_sets[...]]);
    self.device.cmd_draw(command_buffer, 3, 1, 0, 0); // 3 vertices for fullscreen triangle

    // 2. Render mesh (cube)
    self.device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, self.graphics_pipeline);
    self.device.cmd_bind_vertex_buffers(...);
    self.device.cmd_bind_index_buffer(...);
    self.device.cmd_bind_descriptor_sets(command_buffer, ..., &[self.descriptor_sets[...]]);
    self.device.cmd_draw_indexed(command_buffer, ...);

    self.device.cmd_end_render_pass(command_buffer);
    self.device.end_command_buffer(command_buffer)?;
}
```

### 4. Update Nebula Uniforms

```rust
unsafe fn update_nebula_uniform_buffer(&mut self, image_index: usize) -> anyhow::Result<()> {
    let time = self.frame_count as f32 * 0.0005;

    let view = Mat4::look_at_rh(
        Vec3::new(3.0, 3.0, 3.0),
        Vec3::ZERO,
        Vec3::Y,
    );

    let aspect = self.swapchain_extent.width as f32 / self.swapchain_extent.height as f32;
    let mut proj = Mat4::perspective_rh(45.0_f32.to_radians(), aspect, 0.1, 100.0);
    proj.y_axis.y *= -1.0;

    let nebula_ubo = NebulaUniformBufferObject {
        view,
        proj,
        view_pos: Vec3::new(3.0, 3.0, 3.0),
        time,
        octaves: 6,
        lacunarity: 2.0,
        gain: 0.5,
        frequency: 1.0,
        color_primary: Vec3::new(0.8, 0.3, 0.9),  // Purple
        _padding1: 0.0,
        color_secondary: Vec3::new(0.3, 0.6, 1.0), // Blue
        _padding2: 0.0,
        density: 0.5,
        _padding3: [0.0; 3],
    };

    let data = self.device.map_memory(...)?;
    std::ptr::copy_nonoverlapping(&nebula_ubo, data as *mut NebulaUniformBufferObject, 1);
    self.device.unmap_memory(...);

    Ok(())
}
```

### 5. Render Pass Considerations

The current render pass clears the screen, which is perfect:
- First pass: Nebula renders to background
- Second pass: Mesh renders on top with depth testing

No changes needed to render pass!

### 6. Initialization in `new()`

Add after creating the mesh pipeline:

```rust
// Create nebula pipeline
let nebula_descriptor_set_layout = Self::create_nebula_descriptor_set_layout(&device)?;
let (nebula_pipeline_layout, nebula_pipeline) = Self::create_nebula_pipeline(
    &device,
    swapchain_extent,
    render_pass,
    nebula_descriptor_set_layout,
)?;

// Create nebula uniform buffers
let (nebula_uniform_buffers, nebula_uniform_buffers_memory) = Self::create_uniform_buffers(
    &instance,
    physical_device,
    &device,
    MAX_FRAMES_IN_FLIGHT,
)?;

// Create nebula descriptor pool and sets
let nebula_descriptor_pool = Self::create_descriptor_pool(&device, MAX_FRAMES_IN_FLIGHT)?;
let nebula_descriptor_sets = Self::create_descriptor_sets(
    &device,
    nebula_descriptor_pool,
    nebula_descriptor_set_layout,
    &nebula_uniform_buffers,
    MAX_FRAMES_IN_FLIGHT,
)?;
```

## Simpler Alternative: Test the Nebula Standalone

If you want to see the nebula working first, you can:

1. **Temporarily replace** the mesh pipeline with nebula pipeline
2. Comment out mesh rendering
3. Just render the fullscreen nebula
4. Once it works, add back the mesh rendering

This lets you test the nebula in isolation!

## Expected Result

When fully integrated:
- **Background**: Swirling purple/blue nebula with 3D noise
- **Foreground**: Spinning PBR-lit cube
- **Animation**: Nebula slowly evolves, cube rotates
- **Performance**: Should still run at 1000+ FPS

## Nebula Parameters to Tweak

In the shader or uniforms:
- `octaves`: More = more detail (default: 6)
- `lacunarity`: Frequency multiplier (default: 2.0)
- `gain`: Amplitude multiplier (default: 0.5)
- `frequency`: Base noise frequency (default: 1.0)
- `color_primary`: First color (purple: 0.8, 0.3, 0.9)
- `color_secondary`: Second color (blue: 0.3, 0.6, 1.0)
- `density`: Nebula opacity (default: 0.5)

## Why This is Complex

Adding a second pipeline requires:
1. New descriptor set layout (~50 lines)
2. New pipeline creation (~150 lines)
3. New uniform buffer management (~50 lines)
4. Modified command buffer recording (~30 lines)
5. Cleanup code (~20 lines)
6. Initialization code (~30 lines)

**Total**: ~330 lines of additional Vulkan code

This is doable but requires careful implementation to avoid errors!

## Next Steps

Would you like me to:
1. **Implement the full integration** (will be a large change ~330 lines)
2. **Create a standalone nebula-only version** (simpler, ~100 lines of changes)
3. **Keep the current working engine** and document this for future work

The engine is working beautifully as-is with the spinning cube! 🎮
