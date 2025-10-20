# Snapshot System

## Overview
Snapshots are periodic captures of the complete game state, enabling efficient replay, fast-forward/rewind, and reducing the computational cost of rebuilding state from events. Critical for particles, physics, and visual effects that are expensive to recalculate.

---

## Core Concepts

### Why Snapshots?

Event sourcing alone requires replaying all events from the beginning:
```
To get state at Turn 50:
- Replay Event 1
- Replay Event 2
- ...
- Replay Event 4,327
- Replay Event 4,328
```

With snapshots:
```
To get state at Turn 50:
- Load Snapshot at Turn 40
- Replay Events 40-50 (maybe 100-200 events)
```

### Snapshot Strategy

Create snapshots at regular intervals:
- **Every Turn**: For small games, snapshot after each turn completes
- **Every N Turns**: For large games, snapshot every 5-10 turns
- **Key Events**: Snapshot before major battles or important moments
- **Player Request**: Allow manual snapshot creation

---

## Snapshot Structure

### Complete State Snapshot
```rust
#[derive(Clone, Serialize, Deserialize)]
pub struct GameSnapshot {
    /// Snapshot metadata
    pub snapshot_id: u64,
    pub turn: u32,
    pub timestamp: f64,
    pub session_id: String,

    /// Core game state
    pub world_state: WorldState,
    pub particle_state: ParticleState,
    pub physics_state: PhysicsState,
    pub visual_effects_state: VisualEffectsState,

    /// Turn/simulation state
    pub current_turn: u32,
    pub simulation_time: f32,
    pub game_mode: GameMode,

    /// Event store reference
    pub last_event_id: u64,
}
```

### World State
```rust
#[derive(Clone, Serialize, Deserialize)]
pub struct WorldState {
    /// All entities and their components
    pub entities: Vec<EntitySnapshot>,

    /// Camera state
    pub camera_origin: DVec3,
    pub camera_position: Vec3,
    pub camera_rotation: Quat,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct EntitySnapshot {
    pub entity_id: EntityId,
    pub components: EntityComponents,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct EntityComponents {
    pub position: Option<DVec3>,
    pub rotation: Option<DQuat>,
    pub velocity: Option<DVec3>,
    pub angular_velocity: Option<DVec3>,
    pub ship: Option<ShipSnapshot>,
    pub health: Option<HealthSnapshot>,
    // ... other components
}
```

### Ship State
```rust
#[derive(Clone, Serialize, Deserialize)]
pub struct ShipSnapshot {
    pub max_thruster_range: f32,
    pub max_rotation_per_turn: f32,
    pub moveable: bool,

    /// Movement state
    pub has_boosted: bool,
    pub initiated_full_stop: bool,
    pub confirmed_move: bool,
    pub movement_mode: MovementMode,

    /// Planned movement
    pub target_position: DVec3,
    pub target_rotation: DQuat,

    /// Velocity tracking
    pub last_velocity: DVec3,
    pub control_point: DVec3,

    /// Subsystems
    pub subsystems: Vec<SubsystemSnapshot>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SubsystemSnapshot {
    pub subsystem_id: SubsystemId,
    pub subsystem_type: SubsystemType,
    pub health: f32,
    pub max_health: f32,
    pub enabled: bool,
}
```

### Particle State
```rust
#[derive(Clone, Serialize, Deserialize)]
pub struct ParticleState {
    /// Active particle systems
    pub systems: Vec<ParticleSystemSnapshot>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ParticleSystemSnapshot {
    pub system_id: u64,
    pub system_type: ParticleSystemType,
    pub position: DVec3,
    pub active: bool,

    /// All active particles
    pub particles: Vec<ParticleSnapshot>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ParticleSnapshot {
    pub position: Vec3,
    pub velocity: Vec3,
    pub life_remaining: f32,
    pub size: f32,
    pub color: Vec4,
    pub rotation: f32,
}
```

### Physics State
```rust
#[derive(Clone, Serialize, Deserialize)]
pub struct PhysicsState {
    /// Rapier physics world state
    pub rigid_bodies: Vec<RigidBodySnapshot>,
    pub colliders: Vec<ColliderSnapshot>,

    /// Collision pairs being tracked
    pub active_collisions: Vec<CollisionPair>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RigidBodySnapshot {
    pub entity_id: EntityId,
    pub position: DVec3,
    pub rotation: DQuat,
    pub linear_velocity: DVec3,
    pub angular_velocity: DVec3,
    pub mass: f64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ColliderSnapshot {
    pub entity_id: EntityId,
    pub shape: ColliderShape,
    pub friction: f32,
    pub restitution: f32,
}
```

### Visual Effects State
```rust
#[derive(Clone, Serialize, Deserialize)]
pub struct VisualEffectsState {
    /// Active explosions
    pub explosions: Vec<ExplosionSnapshot>,

    /// Active thruster effects
    pub thrusters: Vec<ThrusterEffectSnapshot>,

    /// Active beam weapons
    pub beams: Vec<BeamSnapshot>,

    /// Active projectiles (visual representation)
    pub projectiles: Vec<ProjectileSnapshot>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ExplosionSnapshot {
    pub explosion_id: u64,
    pub position: DVec3,
    pub start_time: f32,
    pub lifetime: f32,
    pub scale: f32,
    pub explosion_type: ExplosionType,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ThrusterEffectSnapshot {
    pub ship_id: EntityId,
    pub thruster_positions: Vec<Vec3>,
    pub intensity: f32,
    pub active: bool,
}
```

---

## Snapshot Manager

### Implementation
```rust
pub struct SnapshotManager {
    /// All snapshots, indexed by turn
    snapshots: HashMap<u32, GameSnapshot>,

    /// Snapshot interval (e.g., every 5 turns)
    snapshot_interval: u32,

    /// Maximum snapshots to keep in memory
    max_in_memory: usize,

    /// Snapshot counter
    next_snapshot_id: u64,
}

impl SnapshotManager {
    pub fn new(snapshot_interval: u32, max_in_memory: usize) -> Self {
        Self {
            snapshots: HashMap::new(),
            snapshot_interval,
            max_in_memory,
            next_snapshot_id: 0,
        }
    }

    /// Create snapshot of current game state
    pub fn create_snapshot(&mut self, game: &Game) -> u64 {
        let snapshot = GameSnapshot {
            snapshot_id: self.next_snapshot_id,
            turn: game.current_turn,
            timestamp: game.time(),
            session_id: game.session_id.clone(),
            world_state: self.capture_world_state(&game.world),
            particle_state: self.capture_particle_state(&game.particle_systems),
            physics_state: self.capture_physics_state(&game.physics_world),
            visual_effects_state: self.capture_visual_effects(&game.visual_effects),
            current_turn: game.current_turn,
            simulation_time: game.simulation_time,
            game_mode: game.game_manager.mode,
            last_event_id: game.event_store.last_event_id(),
        };

        let snapshot_id = snapshot.snapshot_id;
        self.snapshots.insert(game.current_turn, snapshot);
        self.next_snapshot_id += 1;

        // Cleanup old snapshots if over limit
        self.cleanup_old_snapshots();

        snapshot_id
    }

    /// Should we create a snapshot this turn?
    pub fn should_snapshot(&self, turn: u32) -> bool {
        turn % self.snapshot_interval == 0
    }

    /// Get snapshot at or before specified turn
    pub fn get_snapshot(&self, turn: u32) -> Option<&GameSnapshot> {
        // Try exact turn first
        if let Some(snapshot) = self.snapshots.get(&turn) {
            return Some(snapshot);
        }

        // Find closest snapshot before this turn
        let mut best_turn = 0;
        for &snapshot_turn in self.snapshots.keys() {
            if snapshot_turn <= turn && snapshot_turn > best_turn {
                best_turn = snapshot_turn;
            }
        }

        if best_turn > 0 {
            self.snapshots.get(&best_turn)
        } else {
            None
        }
    }

    /// Remove old snapshots to conserve memory
    fn cleanup_old_snapshots(&mut self) {
        if self.snapshots.len() <= self.max_in_memory {
            return;
        }

        // Sort turns
        let mut turns: Vec<u32> = self.snapshots.keys().copied().collect();
        turns.sort();

        // Keep only the most recent snapshots
        let to_remove = turns.len() - self.max_in_memory;
        for turn in turns.iter().take(to_remove) {
            self.snapshots.remove(turn);
        }
    }

    /// Save snapshots to disk
    pub fn save_to_disk(&self, path: &str) -> Result<(), Error> {
        for (turn, snapshot) in &self.snapshots {
            let snapshot_path = format!("{}/snapshot_turn_{}.json.gz", path, turn);
            snapshot.save_compressed(&snapshot_path)?;
        }
        Ok(())
    }

    /// Load snapshots from disk
    pub fn load_from_disk(path: &str) -> Result<Self, Error> {
        let mut manager = Self::new(5, 20); // Default values

        // Find all snapshot files
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension() == Some(std::ffi::OsStr::new("gz")) {
                let snapshot = GameSnapshot::load_compressed(&path)?;
                manager.snapshots.insert(snapshot.turn, snapshot);
            }
        }

        Ok(manager)
    }
}
```

### Capturing State

```rust
impl SnapshotManager {
    fn capture_world_state(&self, world: &EcsWorld) -> WorldState {
        let mut entities = Vec::new();

        for entity in world.iter() {
            let components = EntityComponents {
                position: world.get::<Position>(entity).ok().map(|p| p.0),
                rotation: world.get::<Rotation>(entity).ok().map(|r| r.0),
                velocity: world.get::<Velocity>(entity).ok().map(|v| v.0),
                angular_velocity: world.get::<AngularVelocity>(entity).ok().map(|av| av.0),
                ship: world.get::<Ship>(entity).ok().map(|s| self.capture_ship(s)),
                health: world.get::<Health>(entity).ok().map(|h| HealthSnapshot {
                    current: h.current,
                    max: h.max,
                }),
                // ... capture other components
            };

            entities.push(EntitySnapshot {
                entity_id: entity,
                components,
            });
        }

        WorldState {
            entities,
            camera_origin: world.camera_origin,
            camera_position: Vec3::ZERO, // Capture from camera system
            camera_rotation: Quat::IDENTITY,
        }
    }

    fn capture_ship(&self, ship: &Ship) -> ShipSnapshot {
        ShipSnapshot {
            max_thruster_range: ship.max_thruster_range,
            max_rotation_per_turn: ship.max_rotation_per_turn,
            moveable: ship.moveable,
            has_boosted: ship.has_boosted,
            initiated_full_stop: ship.initiated_full_stop,
            confirmed_move: ship.confirmed_move,
            movement_mode: ship.movement_mode,
            target_position: ship.target_position,
            target_rotation: ship.target_rotation,
            last_velocity: ship.last_velocity,
            control_point: ship.control_point,
            subsystems: ship.subsystems.iter()
                .map(|s| self.capture_subsystem(s))
                .collect(),
        }
    }

    fn capture_particle_state(&self, particle_systems: &[ParticleSystem]) -> ParticleState {
        ParticleState {
            systems: particle_systems.iter()
                .map(|ps| ParticleSystemSnapshot {
                    system_id: ps.id,
                    system_type: ps.system_type,
                    position: ps.position,
                    active: ps.active,
                    particles: ps.particles.iter()
                        .map(|p| ParticleSnapshot {
                            position: p.position,
                            velocity: p.velocity,
                            life_remaining: p.life_remaining,
                            size: p.size,
                            color: p.color,
                            rotation: p.rotation,
                        })
                        .collect(),
                })
                .collect(),
        }
    }

    // ... similar methods for physics and visual effects
}
```

---

## Restoring from Snapshot

### Loading Snapshot
```rust
impl Game {
    /// Restore game state from snapshot
    pub fn restore_from_snapshot(&mut self, snapshot: &GameSnapshot) {
        // Restore turn/simulation state
        self.current_turn = snapshot.current_turn;
        self.simulation_time = snapshot.simulation_time;
        self.game_manager.mode = snapshot.game_mode;

        // Restore world state
        self.restore_world_state(&snapshot.world_state);

        // Restore particle state
        self.restore_particle_state(&snapshot.particle_state);

        // Restore physics state
        self.restore_physics_state(&snapshot.physics_state);

        // Restore visual effects
        self.restore_visual_effects(&snapshot.visual_effects_state);

        println!("Restored game state from Turn {} snapshot", snapshot.turn);
    }

    fn restore_world_state(&mut self, world_state: &WorldState) {
        // Clear existing entities
        self.world.clear();

        // Restore camera
        self.world.camera_origin = world_state.camera_origin;

        // Restore entities
        for entity_snapshot in &world_state.entities {
            let entity = self.world.spawn();

            // Restore components
            if let Some(pos) = entity_snapshot.components.position {
                self.world.insert_one(entity, Position(pos)).ok();
            }
            if let Some(rot) = entity_snapshot.components.rotation {
                self.world.insert_one(entity, Rotation(rot)).ok();
            }
            if let Some(vel) = entity_snapshot.components.velocity {
                self.world.insert_one(entity, Velocity(vel)).ok();
            }
            // ... restore other components
        }
    }

    fn restore_particle_state(&mut self, particle_state: &ParticleState) {
        self.particle_systems.clear();

        for system_snapshot in &particle_state.systems {
            let mut system = ParticleSystem::new(
                system_snapshot.system_type,
                system_snapshot.position,
            );

            system.id = system_snapshot.system_id;
            system.active = system_snapshot.active;

            // Restore particles
            for particle_snapshot in &system_snapshot.particles {
                system.particles.push(Particle {
                    position: particle_snapshot.position,
                    velocity: particle_snapshot.velocity,
                    life_remaining: particle_snapshot.life_remaining,
                    size: particle_snapshot.size,
                    color: particle_snapshot.color,
                    rotation: particle_snapshot.rotation,
                });
            }

            self.particle_systems.push(system);
        }
    }
}
```

---

## Snapshot + Event Replay

### Fast State Reconstruction
```rust
impl Game {
    /// Jump to specific turn efficiently
    pub fn jump_to_turn(&mut self, target_turn: u32) {
        // Find closest snapshot
        if let Some(snapshot) = self.snapshot_manager.get_snapshot(target_turn) {
            // Restore from snapshot
            self.restore_from_snapshot(snapshot);

            // Replay events from snapshot turn to target turn
            let events = self.event_store.get_turn_range(
                snapshot.turn + 1,
                target_turn
            );

            for event in events {
                self.apply_event(event);
            }

            println!("Jumped to Turn {} using snapshot at Turn {}",
                target_turn, snapshot.turn);
        } else {
            // No snapshot available, replay from beginning
            self.reset_to_initial_state();

            let events = self.event_store.get_turn_range(0, target_turn);
            for event in events {
                self.apply_event(event);
            }

            println!("Jumped to Turn {} by replaying all events", target_turn);
        }
    }
}
```

---

## File Format

### Snapshot File
```json
{
  "snapshot_id": 5,
  "turn": 25,
  "timestamp": 1705334650.0,
  "session_id": "game_2024_01_15_001",
  "world_state": {
    "entities": [
      {
        "entity_id": 42,
        "components": {
          "position": { "x": 150.0, "y": 25.0, "z": -30.0 },
          "rotation": { "x": 0.0, "y": 0.707, "z": 0.0, "w": 0.707 },
          "velocity": { "x": 5.0, "y": 0.0, "z": 2.0 },
          "ship": {
            "max_thruster_range": 20.0,
            "moveable": true,
            "has_boosted": false,
            "movement_mode": "MoveAndTurn",
            "subsystems": [
              {
                "subsystem_id": 0,
                "subsystem_type": "ImpulseEngine",
                "health": 75.0,
                "max_health": 100.0,
                "enabled": true
              }
            ]
          }
        }
      }
    ],
    "camera_origin": { "x": 0.0, "y": 0.0, "z": 0.0 }
  },
  "particle_state": {
    "systems": [
      {
        "system_id": 12,
        "system_type": "ThrusterExhaust",
        "position": { "x": 150.0, "y": 25.0, "z": -30.0 },
        "active": true,
        "particles": [
          {
            "position": { "x": 0.5, "y": 0.0, "z": -1.0 },
            "velocity": { "x": 0.0, "y": 0.0, "z": -2.0 },
            "life_remaining": 0.8,
            "size": 0.5,
            "color": { "r": 1.0, "g": 0.5, "b": 0.2, "a": 0.8 },
            "rotation": 45.0
          }
        ]
      }
    ]
  }
}
```

---

## Performance Considerations

### Snapshot Size
- **Small game (5 ships)**: ~50-100 KB per snapshot
- **Medium game (20 ships)**: ~200-500 KB per snapshot
- **Large game (100 ships)**: ~1-5 MB per snapshot
- **With compression**: 60-80% size reduction

### Memory Usage
- Keep 10-20 recent snapshots in memory (~10-100 MB)
- Older snapshots stored on disk
- Load on-demand for replay

### Optimization Strategies
1. **Selective Snapshots**: Don't snapshot every component, only what's needed
2. **Delta Encoding**: Store changes from previous snapshot
3. **Lazy Loading**: Load snapshot components on-demand
4. **Compression**: Use gzip or custom binary format
5. **Pruning**: Remove snapshots that are rarely accessed

---

## Integration with Replay System

### Timeline Scrubbing
```rust
impl ReplayController {
    /// Jump to specific time in replay
    pub fn scrub_to_time(&mut self, target_time: f32) {
        let target_turn = (target_time / 10.0) as u32;
        let time_in_turn = target_time % 10.0;

        // Jump to turn using snapshot
        self.game.jump_to_turn(target_turn);

        // Simulate to exact time within turn
        self.game.simulation_time = time_in_turn;
        self.game.update_simulation_to_time(time_in_turn);
    }

    /// Rewind replay
    pub fn rewind(&mut self, seconds: f32) {
        let current_time = self.game.current_turn as f32 * 10.0
            + self.game.simulation_time;
        let target_time = (current_time - seconds).max(0.0);
        self.scrub_to_time(target_time);
    }

    /// Fast-forward replay
    pub fn fast_forward(&mut self, seconds: f32) {
        let current_time = self.game.current_turn as f32 * 10.0
            + self.game.simulation_time;
        let target_time = current_time + seconds;
        self.scrub_to_time(target_time);
    }
}
```

---

## Testing

### Snapshot Consistency
```rust
#[test]
fn test_snapshot_restore() {
    let mut game = Game::new();

    // Play 10 turns
    for _ in 0..10 {
        game.advance_turn();
    }

    // Create snapshot
    let snapshot = game.snapshot_manager.create_snapshot(&game);

    // Continue playing
    for _ in 0..5 {
        game.advance_turn();
    }

    // Restore from snapshot
    let snapshot = game.snapshot_manager.get_snapshot(10).unwrap();
    game.restore_from_snapshot(snapshot);

    // Verify state matches
    assert_eq!(game.current_turn, 10);
}
```

---

## Future Enhancements

- **Incremental Snapshots**: Only store changes since last snapshot
- **Snapshot Diff**: Compare two snapshots to see what changed
- **Snapshot Branching**: Create alternate timelines from snapshots
- **Cloud Snapshots**: Upload snapshots to cloud storage
- **Snapshot Verification**: Checksum validation for anti-cheat
- **Snapshot Compression**: Custom binary format for 90%+ compression
