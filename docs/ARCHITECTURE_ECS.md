# ECS Architecture for Turn-Based Space Tactics

## Overview

This document describes the new ECS (Entity Component System) architecture designed for large-scale turn-based space combat with 64-bit coordinate precision.

## Key Features

### 1. **64-bit Coordinate System**

**Problem**: GPUs only support 32-bit floats (~7 decimal digits precision), causing jitter at planetary distances.

**Solution**: Camera-relative rendering
- Store all positions as `DVec3` (f64) in ECS world
- Render everything relative to camera origin
- Convert to `Vec3` (f32) only at render time

**Precision**:
- Near camera: ~1 nanometer precision
- At 1000 km: ~1 millimeter precision
- At 1 million km: ~1 meter precision

### 2. **ECS with hecs**

**Why hecs?**
- **Zero-cost abstractions**: As fast as handwritten code
- **Deterministic iteration**: Critical for turn-based replay
- **Simple API**: Easy to understand and use
- **No complex scheduling**: We control the update order

**Core Components**:
```rust
Position(DVec3)         // 64-bit world position
Rotation(DQuat)         // 64-bit quaternion rotation
Scale(DVec3)            // 64-bit scale
Velocity(DVec3)         // Meters per second
Mass(f64)               // Kilograms
Health                  // HP system
Ship                    // Ship-specific data
TurnState               // Turn-based orders
```

### 3. **Rapier Physics**

**Why Rapier?**
- **Deterministic**: Same inputs = same outputs (critical for multiplayer)
- **Fast**: Written in Rust, highly optimized
- **Feature-complete**: Collision detection, raycasting, joints

**Configuration**:
```rust
// Fixed timestep for determinism
integration_params.dt = 1.0 / 60.0;  // 60Hz

// Enhanced determinism feature
features = ["enhanced-determinism"]
```

**Use Cases**:
- Ship-to-ship collisions
- Projectile impact detection
- Raycast targeting (line of sight)
- Asteroid field navigation

### 4. **Spatial Partitioning**

**Problem**: Checking every entity against every other entity is O(n²).

**Solution**: Sector-based spatial index
- Divide space into cubic sectors (e.g. 10km x 10km x 10km)
- Only check entities in nearby sectors
- Reduces collision checks from O(n²) to O(n*k) where k = avg entities per sector

**LOD (Level of Detail)**:
```
0-1000m:      High detail (full mesh + physics)
1000-10000m:  Medium detail (simplified)
10000-100km:  Low detail (billboard)
>100km:       Culled
```

### 5. **Turn-Based Event System**

**Design Pattern**: Event Sourcing
- All actions are stored as events
- State can be reconstructed from events
- Enables replay, undo, and deterministic multiplayer

**Event Types**:
```rust
EntitySpawned
EntityDestroyed
EntityMoved
Combat { attacker, defender, damage }
TurnEnded
```

**Turn Resolution**:
1. Player submits orders (Move, Attack, Defend, Wait)
2. All orders collected for the turn
3. Physics simulation runs (deterministic)
4. Combat resolution (health, damage)
5. Events generated for replay
6. Next turn begins

## File Structure

```
src/ecs/
├── mod.rs           # EcsWorld, camera origin management
├── components.rs    # All component definitions
├── physics.rs       # Rapier integration
├── spatial.rs       # Spatial index, LOD manager
└── rendering.rs     # Extract render batch from ECS
```

## Usage Example

### Creating the ECS World

```rust
use crate::ecs::*;
use glam::{DVec3, DQuat};

// Create ECS world
let mut ecs_world = EcsWorld::new();

// Spawn a ship
let ship_entity = ecs_world.world.spawn((
    Position(DVec3::new(1000.0, 500.0, 0.0)),
    Rotation(DQuat::IDENTITY),
    Scale(DVec3::ONE),
    Velocity(DVec3::ZERO),
    Mass(50_000.0),  // 50 tons
    Health::new(100.0),
    Ship {
        name: "Destroyer-01".to_string(),
        faction: "Earth".to_string(),
        thrust_force: 100_000.0,  // 100 kN
        rotation_torque: 50_000.0,
    },
    Visual {
        mesh_name: "destroyer".to_string(),
        material_name: "metal".to_string(),
    },
    EntityType::Ship,
));
```

### Rendering Frame

```rust
// Update camera origin for rendering
ecs_world.set_camera_origin(camera_position);

// Extract entities to render (camera-relative)
let render_batch = rendering::extract_render_batch(
    &ecs_world.world,
    ecs_world.camera_origin,
    1_000_000.0,  // Max render distance: 1000 km
);

// Send to renderer
for entity in render_batch.entities {
    renderer.draw_mesh(
        &entity.mesh_name,
        &entity.material_name,
        entity.model_matrix,
    );
}
```

### Physics Simulation

```rust
use crate::ecs::physics::PhysicsWorld;

let mut physics = PhysicsWorld::new();

// Add ship collider
let ship_handle = physics.add_ship_collider(
    ship_position,
    ship_rotation,
    glam::Vec3::new(10.0, 5.0, 20.0),  // Ship size
);

// Step physics (deterministic)
physics.step();

// Update ECS positions from physics
let ship_body = physics.rigid_body_set.get(ship_handle).unwrap();
let new_position = DVec3::new(
    ship_body.translation().x as f64,
    ship_body.translation().y as f64,
    ship_body.translation().z as f64,
);
```

### Spatial Queries

```rust
use crate::ecs::spatial::SpatialIndex;

let mut spatial_index = SpatialIndex::new(10_000.0);  // 10km sectors

// Insert entities
for (entity, position) in ecs_world.world.query::<&Position>().iter() {
    spatial_index.insert(entity.id(), position.0);
}

// Find nearby entities
let nearby = spatial_index.query_nearby(player_position);
println!("Found {} nearby entities", nearby.len());
```

## Migration Strategy

### Phase 1: Foundation (Current)
- ✅ Add dependencies (hecs, rapier3d, nalgebra)
- ✅ Create ECS module structure
- ✅ Define core components
- ✅ Implement 64-bit coordinate system
- ✅ Create camera-relative rendering

### Phase 2: Integration
- [ ] Create parallel ECS world alongside existing scene graph
- [ ] Add toggle to switch between old/new systems
- [ ] Migrate camera to use 64-bit coordinates
- [ ] Test rendering with large coordinates (1000x nebula scale)

### Phase 3: Migration
- [ ] Port ships to ECS entities
- [ ] Port asteroids to ECS entities
- [ ] Integrate physics collisions
- [ ] Remove old scene graph

### Phase 4: Turn-Based System
- [ ] Implement order system
- [ ] Implement turn resolution
- [ ] Add event logging
- [ ] Implement replay system

### Phase 5: Large-Scale Battles
- [ ] Optimize spatial partitioning
- [ ] Implement LOD system
- [ ] Add fleet management
- [ ] Test with 1000+ ships

## Nebula Scaling

With the new 64-bit system, we can scale the nebula by 1000x:

**Before** (32-bit):
- Nebula scale: ~1000 meters
- Precision issues at this scale
- Jitter visible

**After** (64-bit):
- Nebula scale: ~1,000,000 meters (1000 km)
- No precision issues
- Smooth rendering via camera-relative coordinates

## Performance Considerations

1. **ECS Iteration**: O(n) - very fast
2. **Spatial Index**: O(n*k) - k = avg entities per sector
3. **Physics**: O(n*log(n)) - Rapier's internal optimizations
4. **Rendering**: O(m) - m = visible entities (after culling)

**Target Performance**:
- 10,000 entities in ECS: < 1ms update
- 1,000 visible ships: 60 FPS
- Physics simulation: 60Hz fixed timestep

## Next Steps

1. **Test the foundation**:
   - Build the project with new dependencies
   - Create test entity in ECS
   - Verify camera-relative rendering works

2. **Create migration toggle**:
   - Add `use_ecs` flag to Game struct
   - Render from ECS when enabled
   - Keep old system working for comparison

3. **Implement turn system**:
   - Create order submission UI
   - Implement turn resolution
   - Add event logging

Would you like me to proceed with Phase 2 (Integration) or would you prefer to test the foundation first?
