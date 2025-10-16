# Tribal Engine

A Rust + Vulkan game engine featuring:
- **Procedurally generated raymarched SDF nebula** rendering
- **Traditional polygon mesh rendering** with OBJ file support
- **PBR (Physically Based Rendering)** lit material shaders
- **Directional and point light** sources
- Procedural cube mesh generation for testing

## Prerequisites

1. **Rust** (latest stable) - [Install Rust](https://rustup.rs/)
2. **Vulkan SDK** - [Download Vulkan SDK](https://vulkan.lunarg.com/)
   - Make sure `glslc` is in your PATH
3. **Windows** (tested on Windows, but should work on Linux/macOS with minor adjustments)

## Building

### 1. Compile Shaders

First, compile the GLSL shaders to SPIR-V:

**Windows:**
```bash
compile_shaders.bat
```

**Linux/macOS:**
```bash
chmod +x compile_shaders.sh
./compile_shaders.sh
```

### 2. Build the Engine

```bash
cargo build --release
```

### 3. Run

```bash
cargo run --release
```

## Project Structure

```
tribal-engine/
├── src/
│   ├── main.rs              # Entry point
│   ├── engine.rs            # Core engine loop and window management
│   ├── renderer.rs          # Vulkan renderer implementation
│   ├── mesh.rs              # Mesh data structures and generation
│   ├── lighting.rs          # Lighting system (directional/point lights)
│   └── raymarcher.rs        # SDF raymarching configuration
├── shaders/
│   ├── mesh.vert            # Mesh vertex shader
│   ├── mesh.frag            # PBR fragment shader for meshes
│   ├── raymarch.vert        # Raymarching fullscreen quad vertex shader
│   └── raymarch.frag        # SDF nebula raymarching fragment shader
└── Cargo.toml
```

## Features

### Mesh Rendering
- Procedural cube generation with proper normals and UVs
- Vertex/Index buffer management
- OBJ file loading support (via `tobj` crate)
- PBR material shading with:
  - Albedo
  - Metallic/Roughness workflow
  - Normal mapping support
  - Cook-Torrance BRDF

### Lighting System
- **Directional Light**: Sun-like directional lighting with color and intensity
- **Point Lights**: Positional lights with attenuation (up to 4 concurrent)
- PBR lighting calculations using:
  - GGX normal distribution
  - Schlick-GGX geometry function
  - Fresnel-Schlick approximation

### Raymarched SDF Nebula
- Procedural nebula generation using signed distance fields
- Fractal Brownian Motion (FBM) for organic noise
- Configurable parameters:
  - Octaves, lacunarity, gain, frequency
  - Primary and secondary colors
  - Density control
- Atmospheric scattering
- Volumetric rendering
- Starfield background

### Vulkan Features
- Modern Vulkan 1.2 API usage
- Proper synchronization with semaphores and fences
- Swapchain with mailbox presentation mode
- Depth testing
- Command buffer management
- Descriptor sets for uniform buffers
- Validation layers in debug mode

## Controls

Currently, the engine renders a spinning cube with the lighting setup. Camera controls can be added as needed.

## Extending the Engine

### Loading OBJ Files

```rust
use crate::mesh::Mesh;

let mesh = Mesh::from_obj("path/to/model.obj")?;
```

### Adjusting Nebula Parameters

Modify the nebula configuration in `raymarcher.rs`:

```rust
pub struct NebulaConfig {
    pub octaves: u32,          // Number of noise octaves (default: 6)
    pub lacunarity: f32,       // Frequency multiplier (default: 2.0)
    pub gain: f32,             // Amplitude multiplier (default: 0.5)
    pub frequency: f32,        // Base frequency (default: 1.0)
    pub color_primary: Vec3,   // Primary nebula color
    pub color_secondary: Vec3, // Secondary nebula color
    pub density: f32,          // Nebula density (default: 0.5)
}
```

### Adding More Lights

In `renderer.rs`, modify the initialization:

```rust
let point_lights = vec![
    PointLight {
        position: Vec3::new(2.0, 2.0, 2.0),
        color: Vec3::new(1.0, 0.3, 0.3),
        intensity: 5.0,
    },
    // Add more point lights here (max 4)
];
```

## Future Enhancements

- [ ] Raymarched nebula rendering integrated with mesh pipeline
- [ ] Camera controls (WASD movement, mouse look)
- [ ] Texture loading and binding
- [ ] Shadow mapping
- [ ] Deferred rendering pipeline
- [ ] Entity component system
- [ ] Physics integration
- [ ] ImGui integration for runtime parameter tweaking
- [ ] Multi-pass rendering for post-processing effects
- [ ] Compute shader support

## Technical Details

### Coordinate System
- Right-handed coordinate system
- Y-up axis convention
- Vulkan's inverted Y-axis for projection is handled in the vertex shader

### Performance
- Optimized build recommended (`--release` flag)
- Uses GPU-local memory for vertex/index buffers
- Staging buffers for efficient data transfer
- Double buffering for frame synchronization

## Troubleshooting

### Validation Errors
If you see validation layer errors:
1. Make sure Vulkan SDK is properly installed
2. Update your graphics drivers
3. Check that `VK_LAYER_KHRONOS_validation` is available

### Shader Compilation Errors
Make sure `glslc` is in your PATH:
```bash
glslc --version
```

### Window Not Appearing
- Check your graphics drivers support Vulkan 1.2
- Try running in debug mode for more verbose output

## License

MIT License - Feel free to use this engine for your projects!

## Credits

Built with:
- [ash](https://github.com/ash-rs/ash) - Vulkan bindings for Rust
- [winit](https://github.com/rust-windowing/winit) - Cross-platform window creation
- [glam](https://github.com/bitshifter/glam-rs) - Fast linear algebra
- [tobj](https://github.com/Twinklebear/tobj) - OBJ file loading
