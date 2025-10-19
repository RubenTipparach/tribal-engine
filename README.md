# Tribal Engine

A forward-rendered Vulkan game engine featuring:
- **Scene graph system** with object selection and transforms
- **Procedurally generated raymarched SDF nebula** rendering
- **Traditional polygon mesh rendering** with OBJ file support
- **PBR (Physically Based Rendering)** material system
- **Directional and point light** sources
- **ImGui** integration for runtime tweaking
- **Persistent JSON configuration** for all engine parameters and scene data

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
│   ├── game.rs              # Game state and logic
│   ├── renderer.rs          # Vulkan renderer implementation
│   ├── mesh.rs              # Mesh data structures and OBJ loader
│   ├── lighting.rs          # Lighting system (directional/point lights)
│   ├── nebula.rs            # SDF nebula rendering
│   ├── background.rs        # Skybox rendering
│   ├── scene.rs             # Scene graph and transform system
│   ├── config.rs            # JSON configuration persistence
│   ├── core/                # Core Vulkan abstractions
│   │   ├── camera.rs        # Camera system
│   │   └── ...
│   └── ui/                  # ImGui integration
│       ├── mod.rs           # UI manager and panels
│       └── gui_builder.rs   # ImGui helper widgets
├── shaders/
│   ├── mesh.vert            # Mesh vertex shader
│   ├── mesh.frag            # PBR fragment shader
│   ├── nebula.vert          # Nebula vertex shader
│   ├── nebula.frag          # SDF raymarching fragment shader
│   ├── skybox.vert          # Skybox vertex shader
│   ├── skybox.frag          # Skybox fragment shader
│   ├── imgui.vert           # ImGui vertex shader
│   └── imgui.frag           # ImGui fragment shader
├── config/
│   ├── default.json         # Engine settings (camera, nebula, skybox)
│   └── scene.json           # Scene objects and transforms
└── Cargo.toml
```

## Features

### Mesh Rendering
- OBJ file loader with proper vertex/normal/UV support
- Procedural mesh generation (cube, inverted sphere)
- Vertex/Index buffer management
- PBR material shading:
  - Metallic/Roughness workflow
  - Cook-Torrance BRDF
  - Normal mapping ready

### Lighting System
- **Directional Light**: Sun-like directional lighting with color and intensity
- **Point Lights**: Positional lights with attenuation (up to 4 concurrent)
- PBR lighting calculations using:
  - GGX normal distribution
  - Schlick-GGX geometry function
  - Fresnel-Schlick approximation

### Raymarched SDF Nebula
- Procedural volumetric nebula using signed distance fields
- Configurable colors, density, brightness, and scale
- Real-time parameter tweaking via ImGui

### Skybox System
- Inverted sphere mesh for skybox rendering
- Procedural starfield with configurable density
- Nebula clouds with primary/secondary colors
- Background brightness control

### Scene Graph & Transform System
- Hierarchical scene organization with selection
- Per-object transforms (position, rotation, scale)
- Scene Hierarchy panel for object selection
- Transform editor for modifying objects
- Visibility toggles per object
- Scene persistence in `config/scene.json`

### Configuration System
- JSON-based persistence for all engine parameters
- Separation of concerns:
  - `config/default.json` - Engine settings (nebula, skybox, camera)
  - `config/scene.json` - Scene objects and transforms
- Save/Load buttons in ImGui panels
- Easy benchmarking by reverting to defaults
- All configs stored in source control

### Vulkan Renderer
- Forward rendering pipeline
- Depth testing and proper blending
- Command buffer management
- Descriptor sets for uniform buffers
- Validation layers in debug mode

## Controls

- **WASD**: Camera movement
- **Right Mouse + Drag**: Look around
- **ImGui Panels**:
  - **Scene Hierarchy**: Select objects to edit
  - **Transform**: Edit position, rotation, scale of selected object
  - **Nebula Settings**: Appears when nebula selected
  - **Skybox Settings**: Appears when skybox selected
  - **Save/Load buttons**: Persist your changes

## Configuration Files

All JSON files are stored in the `config/` directory at the project root.

### `config/default.json` - Engine Settings

Object-specific properties that extend beyond basic transforms:

```json
{
  "nebula": {
    "zoom": 0.01,
    "density": 2.0,
    "brightness": 1.0,
    "scale": 20.0,
    "color_center": { "x": 5.6, "y": 7.0, "z": 7.0 },
    ...
  },
  "skybox": {
    "star_density": 2.0,
    "star_brightness": 3.0,
    ...
  },
  "camera": {
    "position": { "x": 0.0, "y": 2.0, "z": 5.0 },
    "pitch": 0.0,
    "yaw": 0.0,
    "roll": 0.0,
    "fov": 70.0,
    ...
  }
}
```

### `config/scene.json` - Scene Objects

Basic transforms for all objects in the scene:

```json
{
  "objects": [
    {
      "id": 0,
      "name": "Cube",
      "object_type": "Cube",
      "transform": {
        "position": { "x": 0.0, "y": 0.0, "z": 0.0 },
        "rotation": { "x": 0.0, "y": 0.0, "z": 0.0, "w": 1.0 },
        "scale": { "x": 1.0, "y": 1.0, "z": 1.0 }
      },
      "visible": true
    },
    ...
  ]
}
```

**Design Philosophy**: Objects with only position/rotation/scale are saved in `scene.json`. Objects with additional properties (like nebula colors, skybox settings) have their extended properties saved in `default.json`.

## Extending the Engine

### Loading OBJ Files

```rust
use crate::mesh::Mesh;

let mesh = Mesh::from_obj("path/to/model.obj")?;
```

### Configuring Engine Parameters

All parameters can be tweaked via ImGui and saved to JSON files:

```rust
// Nebula parameters
pub struct NebulaConfig {
    pub zoom: f32,
    pub density: f32,
    pub brightness: f32,
    pub scale: f32,
    pub color_center: Vec3,
    pub color_edge: Vec3,
    // ... and more
}

// Skybox parameters
pub struct SkyboxConfig {
    pub star_density: f32,
    pub star_brightness: f32,
    pub nebula_intensity: f32,
    // ... and more
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

- [ ] Texture loading and binding
- [ ] Shadow mapping
- [ ] Entity component system
- [ ] Physics integration
- [ ] Post-processing effects
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
