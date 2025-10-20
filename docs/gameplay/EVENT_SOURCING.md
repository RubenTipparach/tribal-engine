# Event Sourcing System

## Overview
Event sourcing is the foundation of the turn-based gameplay system, storing all player actions as a sequence of immutable events rather than just the current game state. This enables complete action history, state reconstruction, replay functionality, and deterministic gameplay.

---

## Core Concepts

### What is Event Sourcing?

Instead of storing only the current state:
```
Game State: {
  ship1: { position: (10, 5, 0), health: 75 },
  ship2: { position: (-5, 0, 10), health: 100 }
}
```

We store a sequence of events:
```
Event 1: ShipSpawned { id: ship1, position: (0, 0, 0), health: 100 }
Event 2: ShipSpawned { id: ship2, position: (-5, 0, 10), health: 100 }
Event 3: ShipMoved { id: ship1, from: (0, 0, 0), to: (10, 5, 0), turn: 1 }
Event 4: ShipDamaged { id: ship1, damage: 25, source: ship2, turn: 2 }
```

### Benefits

1. **Complete History**: Every action ever taken is preserved
2. **State Reconstruction**: Rebuild any past game state by replaying events
3. **Replay System**: Re-execute battles from any point
4. **Debugging**: See exact sequence of events that led to a bug
5. **Audit Trail**: Verify no cheating in multiplayer
6. **Undo/Redo**: Trivial to implement by applying/reverting events
7. **Determinism**: Same events always produce same results

---

## Event Types

### Movement Events
```rust
pub enum MovementEvent {
    /// Player planned a movement for their ship
    MovementPlanned {
        turn: u32,
        ship_id: EntityId,
        movement_mode: MovementMode,
        target_position: DVec3,
        target_rotation: DQuat,
        timestamp: f64,
    },

    /// Movement was confirmed and locked in
    MovementConfirmed {
        turn: u32,
        ship_id: EntityId,
        timestamp: f64,
    },

    /// Ship position updated during simulation
    ShipPositionUpdated {
        turn: u32,
        ship_id: EntityId,
        position: DVec3,
        rotation: DQuat,
        velocity: DVec3,
        simulation_time: f32, // 0.0 to 10.0 seconds
    },
}
```

### Combat Events
```rust
pub enum CombatEvent {
    /// Weapon fired at target
    WeaponFired {
        turn: u32,
        attacker_id: EntityId,
        weapon_id: u32,
        target_id: EntityId,
        target_subsystem: Option<SubsystemId>,
        fire_time: f32, // Time within turn (0-10s)
        timestamp: f64,
    },

    /// Projectile hit target
    ProjectileHit {
        turn: u32,
        projectile_id: u32,
        target_id: EntityId,
        hit_position: DVec3,
        hit_time: f32,
        timestamp: f64,
    },

    /// Damage applied to ship or subsystem
    DamageDealt {
        turn: u32,
        target_id: EntityId,
        subsystem: Option<SubsystemId>,
        damage_amount: f32,
        damage_type: DamageType,
        source_id: EntityId,
        timestamp: f64,
    },

    /// Ship destroyed
    ShipDestroyed {
        turn: u32,
        ship_id: EntityId,
        killer_id: Option<EntityId>,
        timestamp: f64,
    },
}
```

### Collision Events
```rust
pub enum CollisionEvent {
    /// Collision occurred during simulation
    CollisionDetected {
        turn: u32,
        entity_a: EntityId,
        entity_b: EntityId,
        collision_point: DVec3,
        collision_time: f32, // Time within turn
        relative_velocity: f32,
        timestamp: f64,
    },

    /// Subsystem damaged by collision
    CollisionDamage {
        turn: u32,
        ship_id: EntityId,
        subsystem: SubsystemId,
        damage: f32,
        severity: CollisionSeverity,
        timestamp: f64,
    },

    /// Ship trajectory altered by collision
    TrajectoryAltered {
        turn: u32,
        ship_id: EntityId,
        old_velocity: DVec3,
        new_velocity: DVec3,
        impulse_applied: DVec3,
        timestamp: f64,
    },
}
```

### Turn Events
```rust
pub enum TurnEvent {
    /// New turn started
    TurnStarted {
        turn: u32,
        timestamp: f64,
    },

    /// Player confirmed all moves, ready to simulate
    PlayerReady {
        turn: u32,
        player_id: PlayerId,
        timestamp: f64,
    },

    /// Turn simulation began
    SimulationStarted {
        turn: u32,
        timestamp: f64,
    },

    /// Turn simulation completed
    SimulationCompleted {
        turn: u32,
        duration: f32,
        timestamp: f64,
    },

    /// Turn ended
    TurnEnded {
        turn: u32,
        timestamp: f64,
    },
}
```

### System Events
```rust
pub enum SystemEvent {
    /// Game session started
    GameStarted {
        session_id: String,
        scenario_name: String,
        players: Vec<PlayerId>,
        timestamp: f64,
    },

    /// Game session ended
    GameEnded {
        session_id: String,
        winner: Option<PlayerId>,
        reason: EndReason,
        timestamp: f64,
    },

    /// Subsystem disabled
    SubsystemDisabled {
        turn: u32,
        ship_id: EntityId,
        subsystem: SubsystemId,
        reason: DisableReason,
        timestamp: f64,
    },

    /// Subsystem repaired/re-enabled
    SubsystemRepaired {
        turn: u32,
        ship_id: EntityId,
        subsystem: SubsystemId,
        timestamp: f64,
    },
}
```

---

## Event Store Implementation

### Event Structure
```rust
#[derive(Clone, Serialize, Deserialize)]
pub struct GameEvent {
    /// Unique event ID
    pub id: u64,

    /// Turn number when event occurred
    pub turn: u32,

    /// Time within turn (0.0 to 10.0 seconds), None for instant events
    pub simulation_time: Option<f32>,

    /// Real-world timestamp (for debugging/logging)
    pub timestamp: f64,

    /// The actual event data
    pub event_type: EventType,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum EventType {
    Movement(MovementEvent),
    Combat(CombatEvent),
    Collision(CollisionEvent),
    Turn(TurnEvent),
    System(SystemEvent),
}
```

### Event Store
```rust
pub struct EventStore {
    /// All events in chronological order
    events: Vec<GameEvent>,

    /// Event counter for unique IDs
    next_event_id: u64,

    /// Index: turn number -> event indices
    turn_index: HashMap<u32, Vec<usize>>,

    /// Index: entity ID -> event indices
    entity_index: HashMap<EntityId, Vec<usize>>,
}

impl EventStore {
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            next_event_id: 0,
            turn_index: HashMap::new(),
            entity_index: HashMap::new(),
        }
    }

    /// Append a new event to the store
    pub fn append(&mut self, event: GameEvent) {
        let index = self.events.len();

        // Update turn index
        self.turn_index
            .entry(event.turn)
            .or_insert_with(Vec::new)
            .push(index);

        // Update entity index (if event involves entities)
        for entity_id in event.get_involved_entities() {
            self.entity_index
                .entry(entity_id)
                .or_insert_with(Vec::new)
                .push(index);
        }

        self.events.push(event);
        self.next_event_id += 1;
    }

    /// Get all events for a specific turn
    pub fn get_turn_events(&self, turn: u32) -> Vec<&GameEvent> {
        self.turn_index
            .get(&turn)
            .map(|indices| {
                indices.iter()
                    .map(|&i| &self.events[i])
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all events involving a specific entity
    pub fn get_entity_events(&self, entity_id: EntityId) -> Vec<&GameEvent> {
        self.entity_index
            .get(&entity_id)
            .map(|indices| {
                indices.iter()
                    .map(|&i| &self.events[i])
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get events in range [start_turn, end_turn]
    pub fn get_turn_range(&self, start_turn: u32, end_turn: u32) -> Vec<&GameEvent> {
        (start_turn..=end_turn)
            .flat_map(|turn| self.get_turn_events(turn))
            .collect()
    }

    /// Save event store to disk
    pub fn save(&self, path: &str) -> Result<(), Error> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load event store from disk
    pub fn load(path: &str) -> Result<Self, Error> {
        let json = std::fs::read_to_string(path)?;
        let store = serde_json::from_str(&json)?;
        Ok(store)
    }
}
```

---

## State Reconstruction

### Replaying Events
```rust
pub struct GameState {
    world: EcsWorld,
    current_turn: u32,
    // ... other state
}

impl GameState {
    /// Rebuild state from event store
    pub fn from_events(event_store: &EventStore, until_turn: u32) -> Self {
        let mut state = GameState::new();

        // Replay all events up to specified turn
        for turn in 0..=until_turn {
            let events = event_store.get_turn_events(turn);

            for event in events {
                state.apply_event(event);
            }
        }

        state
    }

    /// Apply a single event to the state
    fn apply_event(&mut self, event: &GameEvent) {
        match &event.event_type {
            EventType::Movement(mov) => self.apply_movement_event(mov),
            EventType::Combat(combat) => self.apply_combat_event(combat),
            EventType::Collision(collision) => self.apply_collision_event(collision),
            EventType::Turn(turn) => self.apply_turn_event(turn),
            EventType::System(system) => self.apply_system_event(system),
        }
    }

    fn apply_movement_event(&mut self, event: &MovementEvent) {
        match event {
            MovementEvent::MovementPlanned { ship_id, target_position, target_rotation, .. } => {
                // Update ship's planned movement
                if let Ok(mut transform) = self.world.get_mut::<PlannedTransform>(*ship_id) {
                    transform.target_position = *target_position;
                    transform.target_rotation = *target_rotation;
                }
            }
            MovementEvent::ShipPositionUpdated { ship_id, position, rotation, .. } => {
                // Update ship's actual position
                if let Ok(mut pos) = self.world.get_mut::<Position>(*ship_id) {
                    pos.0 = *position;
                }
                if let Ok(mut rot) = self.world.get_mut::<Rotation>(*ship_id) {
                    rot.0 = *rotation;
                }
            }
            _ => {}
        }
    }

    // ... similar methods for other event types
}
```

---

## Event Validation

### Deterministic Event Application
```rust
impl GameState {
    /// Verify that events are valid and deterministic
    pub fn validate_event(&self, event: &GameEvent) -> Result<(), ValidationError> {
        match &event.event_type {
            EventType::Movement(mov) => self.validate_movement(mov),
            EventType::Combat(combat) => self.validate_combat(combat),
            // ... other validations
        }
    }

    fn validate_movement(&self, event: &MovementEvent) -> Result<(), ValidationError> {
        match event {
            MovementEvent::MovementPlanned { ship_id, target_position, .. } => {
                // Verify ship exists
                if !self.world.contains(*ship_id) {
                    return Err(ValidationError::EntityNotFound(*ship_id));
                }

                // Verify movement is within range
                let current_pos = self.world.get::<Position>(*ship_id)?;
                let distance = (*target_position - current_pos.0).length();
                let max_range = self.get_ship_max_range(*ship_id)?;

                if distance > max_range {
                    return Err(ValidationError::MovementOutOfRange {
                        distance,
                        max_range,
                    });
                }

                Ok(())
            }
            _ => Ok(())
        }
    }
}
```

---

## File Format

### Event Log File Structure
```json
{
  "session_id": "game_2024_01_15_001",
  "scenario": "Asteroid Belt Ambush",
  "start_time": 1705334400.0,
  "events": [
    {
      "id": 0,
      "turn": 0,
      "simulation_time": null,
      "timestamp": 1705334400.0,
      "event_type": {
        "System": {
          "GameStarted": {
            "session_id": "game_2024_01_15_001",
            "scenario_name": "Asteroid Belt Ambush",
            "players": ["player1", "player2"],
            "timestamp": 1705334400.0
          }
        }
      }
    },
    {
      "id": 1,
      "turn": 1,
      "simulation_time": null,
      "timestamp": 1705334410.0,
      "event_type": {
        "Movement": {
          "MovementPlanned": {
            "turn": 1,
            "ship_id": 42,
            "movement_mode": "MoveAndTurn",
            "target_position": { "x": 10.0, "y": 5.0, "z": 0.0 },
            "target_rotation": { "x": 0.0, "y": 0.707, "z": 0.0, "w": 0.707 },
            "timestamp": 1705334410.0
          }
        }
      }
    }
  ]
}
```

### Compression
For large games, compress the event log:
```rust
use flate2::Compression;
use flate2::write::GzEncoder;

impl EventStore {
    pub fn save_compressed(&self, path: &str) -> Result<(), Error> {
        let json = serde_json::to_string(self)?;
        let file = File::create(path)?;
        let mut encoder = GzEncoder::new(file, Compression::default());
        encoder.write_all(json.as_bytes())?;
        encoder.finish()?;
        Ok(())
    }
}
```

---

## Integration with Game Loop

### Recording Events
```rust
impl Game {
    /// Player plans movement
    pub fn plan_movement(
        &mut self,
        ship_id: EntityId,
        target_position: DVec3,
        target_rotation: DQuat,
    ) {
        // Create event
        let event = GameEvent {
            id: self.event_store.next_event_id,
            turn: self.current_turn,
            simulation_time: None,
            timestamp: self.time(),
            event_type: EventType::Movement(MovementEvent::MovementPlanned {
                turn: self.current_turn,
                ship_id,
                movement_mode: self.get_ship_movement_mode(ship_id),
                target_position,
                target_rotation,
                timestamp: self.time(),
            }),
        };

        // Append to event store
        self.event_store.append(event);

        // Apply to current state
        self.apply_movement(ship_id, target_position, target_rotation);
    }

    /// Simulation update
    pub fn update_simulation(&mut self, delta_time: f32) {
        // ... simulation logic ...

        // Record position updates
        for (entity, (pos, rot, vel)) in self.world.query::<(&Position, &Rotation, &Velocity)>() {
            let event = GameEvent {
                id: self.event_store.next_event_id,
                turn: self.current_turn,
                simulation_time: Some(self.simulation_time),
                timestamp: self.time(),
                event_type: EventType::Movement(MovementEvent::ShipPositionUpdated {
                    turn: self.current_turn,
                    ship_id: entity,
                    position: pos.0,
                    rotation: rot.0,
                    velocity: vel.0,
                    simulation_time: self.simulation_time,
                }),
            };

            self.event_store.append(event);
        }
    }
}
```

---

## Performance Considerations

### Event Store Size
- **10-turn game**: ~1000-5000 events (50-250 KB uncompressed)
- **100-turn game**: ~10,000-50,000 events (500 KB - 2.5 MB uncompressed)
- **With compression**: 80-90% size reduction

### Memory Usage
- Keep full event log in memory during gameplay
- Write to disk periodically (every turn or every 10 turns)
- For replay, load events on-demand or use snapshots

### Optimization Strategies
1. **Event Batching**: Group similar events together
2. **Delta Encoding**: Store position changes instead of absolute positions
3. **Lazy Loading**: Load events only when needed for replay
4. **Indexing**: Maintain indices for fast queries
5. **Snapshots**: Combine with snapshot system (see SNAPSHOTS.md)

---

## Testing & Debugging

### Event Verification
```rust
#[test]
fn test_event_replay_determinism() {
    // Create initial state
    let mut state1 = GameState::new();

    // Apply events in real-time
    for event in &events {
        state1.apply_event(event);
    }

    // Rebuild state from events
    let state2 = GameState::from_events(&event_store, final_turn);

    // States should be identical
    assert_eq!(state1, state2);
}
```

### Event Inspection
```rust
impl EventStore {
    /// Print all events for debugging
    pub fn dump_events(&self, turn: u32) {
        println!("=== Events for Turn {} ===", turn);
        for event in self.get_turn_events(turn) {
            println!("{}: {:?}", event.id, event.event_type);
        }
    }
}
```

---

## Future Enhancements

- **Event Compression**: Custom binary format instead of JSON
- **Event Streaming**: Stream events over network for multiplayer
- **Event Filtering**: Query DSL for complex event searches
- **Event Transformation**: Modify historical events for "what-if" scenarios
- **Event Aggregation**: Combine multiple events into higher-level events
- **Conflict Detection**: Detect and resolve conflicting events in async multiplayer
