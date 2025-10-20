# Replay System

## Overview
The replay system allows players to rewatch and analyze combat encounters from any angle and any point in time. Combines event sourcing and snapshots to provide frame-perfect replay with full camera control, timeline scrubbing, and tactical analysis tools.

---

## Core Features

### Replay Capabilities

1. **Session Replay**: Rewatch current battle from any point
2. **Saved Replay**: Load and analyze past encounters
3. **Timeline Scrubbing**: Jump to any moment in the battle
4. **Free Camera**: Full 3D camera control during replay
5. **Speed Control**: Play at 0.25x, 0.5x, 1x, 2x, 4x speed
6. **Pause/Resume**: Freeze action at any frame
7. **Frame-by-Frame**: Step through one frame at a time
8. **Tactical Analysis**: Study enemy behavior and improve strategies
9. **Multiple Viewpoints**: Follow specific ships or units
10. **Event Timeline**: See all events visually on timeline

---

## Replay Types

### Live Replay (Current Session)
```rust
pub struct LiveReplay {
    /// Reference to current game state
    game: Arc<Mutex<Game>>,

    /// Event store being built in real-time
    event_store: Arc<Mutex<EventStore>>,

    /// Snapshot manager
    snapshot_manager: Arc<Mutex<SnapshotManager>>,

    /// Current replay position
    replay_turn: u32,
    replay_time: f32, // Time within turn (0-10s)

    /// Playback state
    is_playing: bool,
    playback_speed: f32,
}

impl LiveReplay {
    /// Jump back to earlier turn while game continues
    pub fn rewind_to_turn(&mut self, turn: u32) {
        // Use snapshot + events to reconstruct state
        let snapshot = self.snapshot_manager.lock()
            .unwrap()
            .get_snapshot(turn)
            .cloned();

        if let Some(snapshot) = snapshot {
            self.restore_from_snapshot(&snapshot);
            self.replay_events_to_turn(turn);
        }

        self.replay_turn = turn;
        self.replay_time = 0.0;
    }

    /// Return to live game state
    pub fn jump_to_live(&mut self) {
        let game = self.game.lock().unwrap();
        self.replay_turn = game.current_turn;
        self.replay_time = game.simulation_time;
    }
}
```

### Saved Replay (Past Session)
```rust
pub struct SavedReplay {
    /// Session metadata
    pub session_id: String,
    pub scenario_name: String,
    pub players: Vec<PlayerId>,
    pub start_time: f64,
    pub end_time: f64,
    pub total_turns: u32,

    /// Complete event history
    pub event_store: EventStore,

    /// Snapshots for fast seeking
    pub snapshot_manager: SnapshotManager,

    /// Current replay position
    pub current_turn: u32,
    pub current_time: f32,

    /// Replay world (separate from game world)
    pub replay_world: EcsWorld,
    pub particle_systems: Vec<ParticleSystem>,
    pub physics_world: PhysicsWorld,
}

impl SavedReplay {
    /// Load replay from disk
    pub fn load(path: &str) -> Result<Self, Error> {
        let manifest = ReplayManifest::load(&format!("{}/manifest.json", path))?;

        Ok(Self {
            session_id: manifest.session_id,
            scenario_name: manifest.scenario_name,
            players: manifest.players,
            start_time: manifest.start_time,
            end_time: manifest.end_time,
            total_turns: manifest.total_turns,
            event_store: EventStore::load(&format!("{}/events.json.gz", path))?,
            snapshot_manager: SnapshotManager::load_from_disk(&format!("{}/snapshots", path))?,
            current_turn: 0,
            current_time: 0.0,
            replay_world: EcsWorld::new(),
            particle_systems: Vec::new(),
            physics_world: PhysicsWorld::new(),
        })
    }

    /// Save replay to disk
    pub fn save(&self, path: &str) -> Result<(), Error> {
        std::fs::create_dir_all(path)?;

        // Save manifest
        let manifest = ReplayManifest {
            session_id: self.session_id.clone(),
            scenario_name: self.scenario_name.clone(),
            players: self.players.clone(),
            start_time: self.start_time,
            end_time: self.end_time,
            total_turns: self.total_turns,
        };
        manifest.save(&format!("{}/manifest.json", path))?;

        // Save events
        self.event_store.save_compressed(&format!("{}/events.json.gz", path))?;

        // Save snapshots
        let snapshot_dir = format!("{}/snapshots", path);
        std::fs::create_dir_all(&snapshot_dir)?;
        self.snapshot_manager.save_to_disk(&snapshot_dir)?;

        Ok(())
    }
}
```

---

## Replay Controller

### Main Controller
```rust
pub struct ReplayController {
    /// Replay type
    replay: ReplayType,

    /// Playback state
    state: PlaybackState,

    /// Playback speed (1.0 = real-time)
    speed: f32,

    /// Camera controller (independent of game camera)
    camera: ReplayCamera,

    /// UI state
    ui_state: ReplayUIState,

    /// Analysis tools
    analysis: TacticalAnalysis,
}

pub enum ReplayType {
    Live(LiveReplay),
    Saved(SavedReplay),
}

pub enum PlaybackState {
    Playing,
    Paused,
    Seeking,
    FrameStepping,
}

impl ReplayController {
    /// Start replay from beginning
    pub fn start_replay(&mut self) {
        self.seek_to_turn(0);
        self.state = PlaybackState::Playing;
    }

    /// Update replay (called each frame)
    pub fn update(&mut self, delta_time: f32) {
        match self.state {
            PlaybackState::Playing => {
                self.advance_replay(delta_time * self.speed);
            }
            PlaybackState::FrameStepping => {
                // Wait for manual step command
            }
            PlaybackState::Paused => {
                // Do nothing
            }
            PlaybackState::Seeking => {
                // Seeking happens instantly, return to paused
                self.state = PlaybackState::Paused;
            }
        }

        // Update camera
        self.camera.update(delta_time);

        // Update analysis tools
        self.analysis.update(self.get_current_state());
    }

    /// Advance replay by delta time
    fn advance_replay(&mut self, delta_time: f32) {
        match &mut self.replay {
            ReplayType::Live(live) => {
                live.replay_time += delta_time;

                // Advance to next turn if needed
                while live.replay_time >= 10.0 {
                    live.replay_time -= 10.0;
                    live.replay_turn += 1;

                    // Check if we've caught up to live game
                    let game = live.game.lock().unwrap();
                    if live.replay_turn >= game.current_turn {
                        live.replay_turn = game.current_turn;
                        live.replay_time = game.simulation_time;
                        self.state = PlaybackState::Paused;
                        break;
                    }
                }

                // Update simulation to current time
                self.simulate_to_time(live.replay_turn, live.replay_time);
            }

            ReplayType::Saved(saved) => {
                saved.current_time += delta_time;

                // Advance to next turn if needed
                while saved.current_time >= 10.0 {
                    saved.current_time -= 10.0;
                    saved.current_turn += 1;

                    // Check if replay is finished
                    if saved.current_turn >= saved.total_turns {
                        saved.current_turn = saved.total_turns;
                        saved.current_time = 10.0;
                        self.state = PlaybackState::Paused;
                        break;
                    }
                }

                // Update simulation to current time
                self.simulate_to_time(saved.current_turn, saved.current_time);
            }
        }
    }

    /// Simulate world state to specific time
    fn simulate_to_time(&mut self, turn: u32, time: f32) {
        // This is where we use snapshots + events to reconstruct state
        // at the exact requested time

        // 1. Get closest snapshot before this time
        let snapshot = match &self.replay {
            ReplayType::Live(live) => {
                live.snapshot_manager.lock().unwrap()
                    .get_snapshot(turn)
                    .cloned()
            }
            ReplayType::Saved(saved) => {
                saved.snapshot_manager
                    .get_snapshot(turn)
                    .cloned()
            }
        };

        // 2. Restore from snapshot
        if let Some(snapshot) = snapshot {
            self.restore_from_snapshot(&snapshot);

            // 3. Replay events from snapshot to current time
            let events = self.get_events_in_range(snapshot.turn, turn);

            for event in events {
                // Only apply events that occurred before current time
                if event.turn < turn ||
                   (event.turn == turn && event.simulation_time.unwrap_or(0.0) <= time) {
                    self.apply_event_to_replay(&event);
                }
            }

            // 4. Simulate to exact time within turn
            if snapshot.turn == turn && snapshot.simulation_time < time {
                self.simulate_turn_to_time(time);
            }
        }
    }

    /// Seek to specific turn
    pub fn seek_to_turn(&mut self, turn: u32) {
        match &mut self.replay {
            ReplayType::Live(live) => {
                live.replay_turn = turn;
                live.replay_time = 0.0;
            }
            ReplayType::Saved(saved) => {
                saved.current_turn = turn;
                saved.current_time = 0.0;
            }
        }

        self.simulate_to_time(turn, 0.0);
        self.state = PlaybackState::Seeking;
    }

    /// Seek to specific time (in seconds from start)
    pub fn seek_to_time(&mut self, total_seconds: f32) {
        let turn = (total_seconds / 10.0) as u32;
        let time_in_turn = total_seconds % 10.0;

        match &mut self.replay {
            ReplayType::Live(live) => {
                live.replay_turn = turn;
                live.replay_time = time_in_turn;
            }
            ReplayType::Saved(saved) => {
                saved.current_turn = turn;
                saved.current_time = time_in_turn;
            }
        }

        self.simulate_to_time(turn, time_in_turn);
        self.state = PlaybackState::Seeking;
    }
}
```

---

## Replay Camera

### Free Camera Control
```rust
pub struct ReplayCamera {
    /// Camera position (64-bit for large-scale support)
    pub position: DVec3,
    pub rotation: Quat,

    /// Camera settings
    pub fov: f32,
    pub move_speed: f32,
    pub rotation_speed: f32,

    /// Camera mode
    pub mode: CameraMode,

    /// Follow target
    pub follow_target: Option<EntityId>,
    pub follow_offset: Vec3,
    pub follow_distance: f32,
}

pub enum CameraMode {
    /// Free camera, user controls position/rotation
    Free,

    /// Follow a specific entity
    Follow { entity_id: EntityId },

    /// Orbit around a point
    Orbit { center: DVec3, distance: f32 },

    /// Cinematic camera path
    Cinematic { path: CameraPath, time: f32 },
}

impl ReplayCamera {
    pub fn update(&mut self, delta_time: f32) {
        match self.mode {
            CameraMode::Free => {
                // User controls camera with WASD + mouse
            }

            CameraMode::Follow { entity_id } => {
                // Update position to follow entity
                if let Some(entity_pos) = self.get_entity_position(entity_id) {
                    let target_pos = entity_pos + self.follow_offset;
                    self.position = self.position.lerp(target_pos, 0.1);

                    // Look at entity
                    let direction = (entity_pos - self.position).normalize();
                    self.rotation = Quat::look_at(direction, Vec3::Y);
                }
            }

            CameraMode::Orbit { center, distance } => {
                // Rotate around center point
                let offset = self.rotation * Vec3::new(0.0, 0.0, distance);
                self.position = center + offset.as_dvec3();
            }

            CameraMode::Cinematic { ref mut path, ref mut time } => {
                // Follow predefined camera path
                *time += delta_time;
                let (pos, rot) = path.sample(*time);
                self.position = pos;
                self.rotation = rot;
            }
        }
    }

    /// Switch to follow mode for entity
    pub fn follow_entity(&mut self, entity_id: EntityId) {
        self.mode = CameraMode::Follow { entity_id };
    }

    /// Switch to free camera
    pub fn free_mode(&mut self) {
        self.mode = CameraMode::Free;
    }
}
```

---

## Timeline UI

### Timeline Visualization
```rust
pub struct ReplayTimeline {
    /// Total duration in seconds
    pub total_duration: f32,

    /// Current position in seconds
    pub current_position: f32,

    /// Events to display on timeline
    pub events: Vec<TimelineEvent>,

    /// Zoom level (seconds per pixel)
    pub zoom: f32,

    /// Scroll offset
    pub scroll_offset: f32,
}

#[derive(Clone)]
pub struct TimelineEvent {
    pub time: f32,
    pub event_type: EventType,
    pub description: String,
    pub color: Vec4,
    pub icon: TimelineIcon,
}

pub enum TimelineIcon {
    Movement,
    WeaponFire,
    Collision,
    Explosion,
    ShipDestroyed,
    TurnBoundary,
}

impl ReplayTimeline {
    /// Render timeline UI
    pub fn render(&self, ui: &Ui) {
        ui.window("Replay Timeline")
            .size([800.0, 150.0], imgui::Condition::FirstUseEver)
            .build(|| {
                // Timeline header
                ui.text(format!("Turn {}/{}",
                    (self.current_position / 10.0) as u32 + 1,
                    (self.total_duration / 10.0) as u32));

                ui.same_line();
                ui.text(format!("{:.1}s / {:.1}s",
                    self.current_position,
                    self.total_duration));

                // Timeline scrubber
                let mut position = self.current_position;
                if ui.slider("##timeline", 0.0, self.total_duration, &mut position) {
                    // User scrubbed timeline
                    self.on_scrub(position);
                }

                // Event markers
                self.render_event_markers(ui);

                // Turn boundaries
                self.render_turn_boundaries(ui);
            });
    }

    fn render_event_markers(&self, ui: &Ui) {
        // Draw event markers on timeline
        let window_width = ui.window_size()[0];

        for event in &self.events {
            // Calculate X position based on event time
            let x = (event.time / self.total_duration) * window_width;

            // Draw marker
            let draw_list = ui.get_window_draw_list();
            let cursor_pos = ui.cursor_screen_pos();

            draw_list.add_circle(
                [cursor_pos[0] + x, cursor_pos[1] + 20.0],
                5.0,
                event.color,
            ).filled(true).build();

            // Tooltip on hover
            if ui.is_mouse_hovering_rect(
                [cursor_pos[0] + x - 5.0, cursor_pos[1] + 15.0],
                [cursor_pos[0] + x + 5.0, cursor_pos[1] + 25.0],
            ) {
                ui.tooltip(|| {
                    ui.text(&event.description);
                    ui.text(format!("Time: {:.2}s", event.time));
                });
            }
        }
    }

    fn render_turn_boundaries(&self, ui: &Ui) {
        let window_width = ui.window_size()[0];
        let draw_list = ui.get_window_draw_list();
        let cursor_pos = ui.cursor_screen_pos();

        // Draw vertical line at each 10-second mark
        let mut turn = 10.0;
        while turn < self.total_duration {
            let x = (turn / self.total_duration) * window_width;

            draw_list.add_line(
                [cursor_pos[0] + x, cursor_pos[1]],
                [cursor_pos[0] + x, cursor_pos[1] + 40.0],
                [0.5, 0.5, 0.5, 1.0],
            ).thickness(1.0).build();

            turn += 10.0;
        }
    }
}
```

---

## Playback Controls

### Control Panel
```rust
pub struct PlaybackControls {
    pub is_playing: bool,
    pub speed: f32,
    pub loop_replay: bool,
}

impl PlaybackControls {
    pub fn render(&mut self, ui: &Ui, replay: &mut ReplayController) {
        ui.window("Playback Controls")
            .size([300.0, 200.0], imgui::Condition::FirstUseEver)
            .build(|| {
                // Play/Pause button
                if self.is_playing {
                    if ui.button("⏸ Pause") {
                        replay.pause();
                        self.is_playing = false;
                    }
                } else {
                    if ui.button("▶ Play") {
                        replay.play();
                        self.is_playing = true;
                    }
                }

                ui.same_line();

                // Stop button
                if ui.button("⏹ Stop") {
                    replay.stop();
                    self.is_playing = false;
                }

                ui.same_line();

                // Frame step buttons
                if ui.button("◀◀ Frame") {
                    replay.step_backward();
                }

                ui.same_line();

                if ui.button("Frame ▶▶") {
                    replay.step_forward();
                }

                // Speed control
                ui.spacing();
                ui.text("Playback Speed:");

                if ui.radio_button("0.25x", &mut self.speed, 0.25) {
                    replay.set_speed(0.25);
                }
                ui.same_line();

                if ui.radio_button("0.5x", &mut self.speed, 0.5) {
                    replay.set_speed(0.5);
                }
                ui.same_line();

                if ui.radio_button("1x", &mut self.speed, 1.0) {
                    replay.set_speed(1.0);
                }
                ui.same_line();

                if ui.radio_button("2x", &mut self.speed, 2.0) {
                    replay.set_speed(2.0);
                }
                ui.same_line();

                if ui.radio_button("4x", &mut self.speed, 4.0) {
                    replay.set_speed(4.0);
                }

                // Loop checkbox
                ui.spacing();
                ui.checkbox("Loop Replay", &mut self.loop_replay);

                // Camera controls
                ui.spacing();
                ui.separator();
                ui.text("Camera:");

                if ui.button("Free Camera") {
                    replay.camera.free_mode();
                }

                if ui.button("Follow Selected Ship") {
                    if let Some(selected) = replay.get_selected_entity() {
                        replay.camera.follow_entity(selected);
                    }
                }
            });
    }
}
```

---

## Tactical Analysis

### Analysis Tools
```rust
pub struct TacticalAnalysis {
    /// Ship movement paths
    pub movement_paths: HashMap<EntityId, Vec<DVec3>>,

    /// Weapon fire lines
    pub weapon_fire_lines: Vec<WeaponFireLine>,

    /// Damage dealt over time
    pub damage_timeline: Vec<DamageEvent>,

    /// Ship positions at key moments
    pub position_snapshots: HashMap<u32, Vec<(EntityId, DVec3)>>,
}

pub struct WeaponFireLine {
    pub attacker_id: EntityId,
    pub target_id: EntityId,
    pub fire_time: f32,
    pub hit_time: f32,
    pub damage: f32,
    pub hit: bool,
}

impl TacticalAnalysis {
    /// Build analysis from event store
    pub fn from_events(event_store: &EventStore) -> Self {
        let mut analysis = Self {
            movement_paths: HashMap::new(),
            weapon_fire_lines: Vec::new(),
            damage_timeline: Vec::new(),
            position_snapshots: HashMap::new(),
        };

        // Process all events
        for event in event_store.iter() {
            match &event.event_type {
                EventType::Movement(MovementEvent::ShipPositionUpdated {
                    ship_id, position, ..
                }) => {
                    analysis.movement_paths
                        .entry(*ship_id)
                        .or_insert_with(Vec::new)
                        .push(*position);
                }

                EventType::Combat(CombatEvent::WeaponFired {
                    attacker_id, target_id, fire_time, ..
                }) => {
                    analysis.weapon_fire_lines.push(WeaponFireLine {
                        attacker_id: *attacker_id,
                        target_id: *target_id,
                        fire_time: *fire_time,
                        hit_time: 0.0,
                        damage: 0.0,
                        hit: false,
                    });
                }

                EventType::Combat(CombatEvent::DamageDealt {
                    target_id, damage_amount, timestamp, ..
                }) => {
                    analysis.damage_timeline.push(DamageEvent {
                        target_id: *target_id,
                        damage: *damage_amount,
                        time: *timestamp,
                    });
                }

                _ => {}
            }
        }

        analysis
    }

    /// Render analysis visualizations
    pub fn render_visualizations(&self, replay: &ReplayController) {
        // Draw movement paths
        for (entity_id, path) in &self.movement_paths {
            self.draw_movement_path(replay, *entity_id, path);
        }

        // Draw weapon fire lines
        for fire_line in &self.weapon_fire_lines {
            if fire_line.fire_time <= replay.get_current_time() {
                self.draw_weapon_line(replay, fire_line);
            }
        }
    }

    fn draw_movement_path(
        &self,
        replay: &ReplayController,
        entity_id: EntityId,
        path: &[DVec3]
    ) {
        // Draw line showing ship's path through space
        for window in path.windows(2) {
            let start = replay.world_to_screen(window[0]);
            let end = replay.world_to_screen(window[1]);

            // Draw line (using immediate mode rendering)
            draw_line(start, end, Color::CYAN.with_alpha(0.5), 0.05);
        }
    }
}
```

---

## Saved Replay Format

### Directory Structure
```
replays/
  └── game_2024_01_15_001/
      ├── manifest.json          # Replay metadata
      ├── events.json.gz          # Compressed event log
      └── snapshots/              # Snapshot directory
          ├── snapshot_turn_0.json.gz
          ├── snapshot_turn_10.json.gz
          ├── snapshot_turn_20.json.gz
          └── ...
```

### Manifest File
```json
{
  "session_id": "game_2024_01_15_001",
  "scenario_name": "Asteroid Belt Ambush",
  "players": ["player1", "player2"],
  "start_time": 1705334400.0,
  "end_time": 1705335000.0,
  "total_turns": 50,
  "duration": 500.0,
  "winner": "player1",
  "metadata": {
    "engine_version": "0.1.0",
    "file_format_version": 1,
    "compression": "gzip"
  }
}
```

---

## Future Enhancements

- **Replay Sharing**: Upload/download replays from cloud
- **Replay Annotations**: Add comments/markers to specific moments
- **Slow Motion**: Variable slow-motion (0.1x, 0.01x)
- **Replay Editing**: Cut/splice replays together
- **Stat Tracking**: Detailed statistics and graphs
- **Heatmaps**: Visualize combat intensity, movement patterns
- **Replay Comparison**: Compare two replays side-by-side
- **AI Analysis**: Automatically identify tactical mistakes
- **Replay Highlights**: Auto-generate highlight reels
- **VR Replay**: Watch replays in virtual reality
