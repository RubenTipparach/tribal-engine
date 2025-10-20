# Tribal Engine

A turn-based space tactics game engine built on Vulkan featuring:

## Core Features
- **64-bit coordinate system** for true-to-scale solar systems and planetary environments
- **ECS architecture** (hecs) with deterministic physics (Rapier)
- **Camera-relative rendering** enabling massive scales without precision loss
- **1000x scaled nebula** with 64-bit precision (20,000 km scale)
- **Procedural star with limb darkening** - physically-based solar simulation parented to nebula
- **SSAO (Screen-Space Ambient Occlusion)** with bilateral blur
- **Scene graph system** with object selection, transforms, and gizmos
- **Procedurally generated raymarched SDF nebula** rendering at planetary scale
- **Traditional polygon mesh rendering** with OBJ file support
- **PBR (Physically Based Rendering)** material system
- **Directional and point light** sources
- **ImGui** integration for runtime tweaking
- **Persistent JSON configuration** for all engine parameters and scene data

## Architecture
- **Turn-based event system** for deterministic gameplay and replay
- **Spatial partitioning** for efficient large-scale battles
- **LOD (Level of Detail)** management for rendering optimization
- **Multi-scale rendering** from ship combat to solar system view

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
│   ├── mesh.rs              # Mesh data structures and OBJ loader
│   ├── material.rs          # Material properties (PBR)
│   ├── material_library.rs  # Material storage and management
│   ├── nebula.rs            # SDF nebula rendering
│   ├── background.rs        # Skybox rendering
│   ├── scene.rs             # Scene graph and transform system
│   ├── config.rs            # JSON configuration persistence
│   ├── gizmo.rs             # 3D transform gizmos
│   ├── core/                # Core Vulkan abstractions
│   │   ├── renderer.rs      # Vulkan renderer implementation
│   │   ├── camera.rs        # Camera system
│   │   ├── lighting.rs      # Lighting system (directional/point lights)
│   │   └── ...
│   ├── ecs/                 # ECS system (NEW - 64-bit coordinates)
│   │   ├── mod.rs           # ECS world, camera-relative rendering
│   │   ├── components.rs    # Position, Rotation, Star, Nebula, Ship, etc.
│   │   ├── hierarchy.rs     # Parent-child transform system
│   │   ├── init.rs          # Entity creation helpers
│   │   ├── physics.rs       # Rapier physics integration
│   │   ├── spatial.rs       # Spatial partitioning, LOD management
│   │   └── rendering.rs     # Extract render batch from ECS
│   ├── ui/                  # ImGui integration
│   │   ├── mod.rs           # UI manager and panels
│   │   └── gui_builder.rs   # ImGui helper widgets
│   └── imgui_renderer.rs    # ImGui Vulkan integration
├── shaders/
│   ├── mesh.vert            # Mesh vertex shader
│   ├── mesh.frag            # PBR fragment shader with SSAO
│   ├── nebula.vert          # Nebula vertex shader
│   ├── nebula.frag          # SDF raymarching fragment shader
│   ├── skybox.vert          # Skybox vertex shader
│   ├── skybox.frag          # Skybox fragment shader (multiple variants)
│   ├── gizmo.vert           # Gizmo vertex shader
│   ├── gizmo.frag           # Gizmo fragment shader
│   ├── star.vert            # Star vertex shader (NEW!)
│   ├── star.frag            # Star fragment shader with limb darkening (NEW!)
│   ├── ssao.vert            # SSAO generation vertex shader
│   ├── ssao.frag            # SSAO generation fragment shader
│   ├── ssao_blur.vert       # SSAO bilateral blur vertex shader
│   ├── ssao_blur.frag       # SSAO bilateral blur fragment shader
│   ├── imgui.vert           # ImGui vertex shader
│   └── imgui.frag           # ImGui fragment shader
├── config/
│   ├── default.json         # Engine settings (camera, nebula, skybox, SSAO)
│   ├── scene.json           # Scene objects and transforms
│   └── materials.json       # Material library
├── docs/
│   ├── 64BIT_COORDINATE_SYSTEM.md           # 64-bit coordinate documentation
│   ├── ARCHITECTURE_ECS.md                  # ECS architecture guide
│   ├── ECS_NEBULA_STAR_IMPLEMENTATION.md    # Nebula & star implementation (NEW!)
│   └── ...
└── Cargo.toml
```

## Features

### 64-Bit Coordinate System (NEW!)
- **True-to-scale solar systems**: Render Earth, Moon, planets at real astronomical distances
- **Camera-relative rendering**: GPU receives 32-bit positions near (0,0,0) for perfect precision
- **1000x nebula scaling**: Massive nebulas spanning millions of kilometers without artifacts
- **No jitter or z-fighting**: Sub-millimeter precision at any scale
- See [docs/64BIT_COORDINATE_SYSTEM.md](docs/64BIT_COORDINATE_SYSTEM.md) for details

### ECS Architecture (NEW!)
- **hecs**: Fast, deterministic Entity Component System
- **Rapier3D**: Deterministic physics for collision detection
- **Spatial partitioning**: Efficient queries for large-scale battles (10,000+ entities)
- **LOD management**: Automatic detail reduction based on distance
- **Turn-based event system**: Event sourcing for replay and undo
- See [docs/ARCHITECTURE_ECS.md](docs/ARCHITECTURE_ECS.md) for architecture guide

### SSAO (Screen-Space Ambient Occlusion)
- Real-time ambient occlusion calculation
- Bilateral blur for smooth results
- Configurable radius, bias, power, and kernel size
- Toggle on/off in ImGui
- Enhances depth perception and realism

### Transform Gizmos
- Visual 3D manipulation tools
- Translate, rotate, and scale modes
- Color-coded axes (X=red, Y=green, Z=blue)
- Click and drag to transform objects
- Screen-space projection for intuitive control

### Mesh Rendering
- OBJ file loader with proper vertex/normal/UV support
- Procedural mesh generation (cube, inverted sphere)
- Vertex/Index buffer management
- PBR material shading:
  - Metallic/Roughness workflow
  - Cook-Torrance BRDF
  - GGX normal distribution
  - Schlick-GGX geometry function
  - Fresnel-Schlick approximation
  - Normal mapping ready

### Lighting System
- **Directional Light**: Sun-like directional lighting with color, intensity, and shadow color
- **Point Lights**: Positional lights with attenuation (up to 4 concurrent)
- **Global Illumination**: Approximate GI using ambient term
- Interactive light direction control via gizmo

### Raymarched SDF Nebula
- Procedural volumetric nebula using signed distance fields
- Configurable colors, density, brightness, and scale
- **Scalable to planetary sizes** with 64-bit coordinates (1000x scale = 20,000 km!)
- Real-time parameter tweaking via ImGui
- Multiple color zones for realistic appearance

### Procedural Star with Limb Darkening (NEW!)
- **Physically-based solar simulation** with realistic limb darkening
- **Wavelength-dependent darkening** using real NASA solar physics equations
- **Surface features**:
  - Multi-octave procedural turbulence
  - Animated flow patterns
  - Dynamic sunspots
  - Realistic color gradients
- **Parented to nebula**: Star automatically follows nebula position via ECS hierarchy
- **True-to-scale**: Default Sun radius (695,700 km) using 64-bit coordinates
- Configurable color, gamma, and exposure
- Based on real limb darkening coefficients from NASA GSFC
- See [docs/ECS_NEBULA_STAR_IMPLEMENTATION.md](docs/ECS_NEBULA_STAR_IMPLEMENTATION.md) for details

### Skybox System
- Inverted sphere mesh for skybox rendering
- **Procedural starfield** with configurable density and brightness
- **Nebula clouds** with primary/secondary colors
- Background brightness control
- Multiple shader variants (simple, starry)

### Scene Graph & Transform System
- Hierarchical scene organization with selection
- Per-object transforms (position, rotation, scale)
- **Singletons category** for unique objects (Skybox, Nebula, SSAO, Lights)
- Scene Hierarchy panel for object selection
- Transform editor for modifying objects
- Visibility toggles per object
- **Focus camera** on selected object (double-click)
- Duplicate objects
- Scene persistence in `config/scene.json`

### Configuration System
- JSON-based persistence for all engine parameters
- **Unified save system**: One "Save Config" button saves everything
- Separation of concerns:
  - `config/default.json` - Engine settings (nebula, skybox, camera, SSAO)
  - `config/scene.json` - Scene objects and transforms
  - `config/materials.json` - Material library
- Auto-load on startup
- Easy benchmarking by reverting to defaults
- All configs stored in source control

### Material System
- Material library with save/load
- Per-object material assignment
- PBR parameters: albedo, metallic, roughness, ambient strength
- GI (Global Illumination) strength
- Material editor with real-time preview

### Vulkan Renderer
- Forward rendering pipeline with SSAO post-processing
- Depth testing and proper blending
- Multi-pass rendering (geometry → SSAO → blur → lighting)
- Command buffer management
- Descriptor sets for uniform buffers
- Push constants for per-draw data
- Validation layers in debug mode

## Controls

### Camera
- **WASD**: Move camera (forward/left/back/right)
- **Space**: Move up
- **Left Shift**: Move down
- **Right Mouse + Drag**: Look around (free camera)
- **Middle Mouse**: Toggle camera up-lock (world Y-up vs. free orientation)

### Object Selection & Manipulation
- **Left Click**: Select object in scene hierarchy
- **Double Click**: Select and focus camera on object
- **Gizmo**: Click and drag colored axes to transform selected object
  - Red axis = X
  - Green axis = Y
  - Blue axis = Z

### ImGui Panels
- **Scene Hierarchy**:
  - Select objects to edit
  - Singletons (Skybox, Nebula, SSAO, Lights) at top
  - Regular objects (Cubes, Meshes) below
  - Save Config button saves EVERYTHING
- **Transform**: Edit position, rotation, scale of selected object
- **Object-Specific Settings** (appears when selected):
  - **Nebula Settings**: Colors, density, brightness, scale
  - **Skybox Settings**: Stars, nebula clouds, background
  - **SSAO Settings**: Enable/disable, radius, bias, power, kernel size
  - **Directional Light**: Color, intensity, shadow color
- **Material Editor**: Edit PBR materials, save to library

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
    "nebula_primary_color": { "x": 0.1, "y": 0.2, "z": 0.4 },
    "nebula_secondary_color": { "x": 0.6, "y": 0.3, "z": 0.8 },
    "nebula_intensity": 1.0,
    "background_brightness": 0.0
  },
  "camera": {
    "position": { "x": 0.0, "y": 2.0, "z": 5.0 },
    "pitch": 0.0,
    "yaw": 0.0,
    "roll": 0.0,
    "move_speed": 5.0,
    "mouse_sensitivity": 0.003,
    "fov": 70.0
  },
  "ssao": {
    "enabled": true,
    "radius": 1.0,
    "bias": 0.1,
    "power": 2.0,
    "kernel_size": 64
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

### Turn-Based Gameplay Systems

To support deterministic turn-based tactical gameplay, the engine will need:

- **Event Sourcing** - Store all player actions as a sequence of events
  - Enables complete action history and state reconstruction
  - Foundation for replay and undo functionality
  - Ensures deterministic gameplay across sessions

- **Snapshots** - Periodic state captures for efficient replay
  - Critical for particles, physics, and visual effects
  - Allows fast-forward/rewind during replay
  - Reduces computational cost of rebuilding state from events

- **Replay System** - Rewatch and analyze combat encounters
  - **Session Replay**: Rewatch current battle from any point
  - **Saved Replay**: Load and analyze past encounters
  - **Tactical Analysis**: Study enemy behavior and improve strategies
  - **Timeline Scrubbing**: Jump to any moment in the battle
  - **Multiple Camera Angles**: Free camera during replay

- **Async Multiplayer** - Turn-based play-by-email/internet
  - **Action Files**: Compressed, deterministic turn data
  - **Session State**: Full battle state for each player's turn
  - **Storage Solutions**: Cloud storage integration (S3, Azure Blob, etc.)
  - **Workflow**:
    1. Player receives session file with current state
    2. Replays entire battle up to their turn
    3. Plans and executes their move
    4. Sends updated action file to next player
    5. When all players submit, turn advances
    6. All players watch simultaneous execution
  - **Security**: Action validation, anti-cheat measures
  - **Compression**: Efficient encoding of game state and actions

- **Movement Range Visualization** - Visual feedback for tactical planning
  - **Position Range Sphere**: Wireframe sphere showing maximum movement distance
  - **Rotation Arc Indicators**: Visual representation of available rotation angles
  - **Partial Rotation Support**:
    - Allocate rotation budget across turn timeline (e.g., X degrees total per turn)
    - Split rotation: Y degrees at time T1, Z degrees at time T2
    - Visual timeline showing rotation keyframes
    - Smooth interpolation between rotation states
  - **Movement Prediction Zones**: Color-coded areas showing reachable positions
  - **Thruster Visual Feedback**: Real-time thruster firing effects during planning

- **Collision & Damage System** - Dynamic combat interruptions
  - **Mid-Turn Collisions**: Detect and resolve collisions during 10-second simulation
  - **Involuntary Movement Interruption**:
    - Collision severity determines impact on planned movement
    - Ships may be knocked off-course, reducing or redirecting momentum
    - Visual feedback showing deviation from planned path
  - **Subsystem Damage**: Collision damage can disable critical systems:
    - **Impulse Engines**: Reduced movement range or complete loss of maneuverability (drift mode)
    - **Maneuvering Thrusters**: Loss of rotation capability or reduced rotation speed
    - **Other Subsystems**: Weapons, shields, sensors may be damaged
  - **Damage Severity Tiers**:
    - Minor: Small trajectory deviation, no system damage
    - Moderate: Significant course change, possible subsystem damage
    - Severe: Major velocity change, high chance of critical system failure
    - Catastrophic: Ship enters uncontrolled drift, multiple systems offline

- **Enhanced Rotation & Thruster Systems** - Realistic motion within deterministic framework
  - **Improved Slerp Interpolation**:
    - Non-linear rotation curves based on thruster physics
    - Acceleration/deceleration phases for rotation
    - Realistic angular momentum visualization
  - **Thruster Visual Effects**:
    - Individual thruster firing animations based on rotation direction
    - Main engine thrust visualization during acceleration
    - Maneuvering thruster bursts for rotation changes
    - Damage states affect thruster visuals (flickering, reduced output)
  - **Deterministic Physics-Feel**:
    - Predictable movement that feels physically realistic
    - Maintains frame-perfect replay despite visual complexity
    - Separation of deterministic simulation from visual effects

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
