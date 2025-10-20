# Turn Movement Implementation

## Overview
This document describes the Phase 1 implementation of the turn-based movement system, focusing on basic movement commands and event sourcing. Physics, particles, and snapshots will be added in later phases.

---

## Phase 1 Scope

### What We're Implementing
- ✅ Basic ship movement on a movement plane
- ✅ Event sourcing for movement commands
- ✅ 3D widget controls (arrows, cones, cubes)
- ✅ Movement range visualization
- ✅ Rotation constraints (90-degree max turn)
- ✅ Timeline UI for turn planning
- ✅ Ship component with movement parameters
- ✅ Bezier curve movement for smooth, momentum-based motion

### What We're NOT Implementing Yet
- ❌ Physics simulation (comes later with snapshots)
- ❌ Particle effects (comes later with snapshots)
- ❌ Collision detection (Phase 2)
- ❌ Multiple movement modes (Phase 2)

---

## Movement System Design

### Movement Constraints

#### Range Constraint
- Ships have a maximum movement range per turn (default: 20 units)
- Movement is constrained to a **cylindrical volume**:
  - **XZ Radius**: 20 units from starting position
  - **Y Height**: ±10 units from starting position (slices along Y-axis)
- Visual representation: Wireframe cylinder with horizontal slices

#### Rotation Constraint
- **Maximum Turn**: 90 degrees per turn
- Ships cannot rotate more than 90 degrees from their starting orientation
- Visual representation: Arc showing available rotation range

#### Movement Plane
- Movement happens on a horizontal plane at the widget's Y position
- Dragging the widget in XZ moves the ship on this plane
- Ship automatically rotates to face the movement direction

### Bezier Curve Movement

Ships use **quadratic Bezier curves** for smooth, momentum-based movement during the 10-second turn execution.

#### Why Bezier Curves?

- **Smooth Acceleration**: Ships don't instantly jump to target position
- **Momentum**: Previous turn's velocity influences current turn's path
- **Predictable**: Deterministic - same inputs always produce same path
- **Realistic Feel**: Simulates inertia without full physics simulation

#### Mathematical Formula

```
P(t) = (1-t)² × P0 + 2(1-t)t × P1 + t² × P2

Where:
- P0 = Start position (ship's current position)
- P1 = Control point (influenced by previous velocity)
- P2 = End position (widget position)
- t = time progress [0.0 to 1.0] (0 to 10 seconds)
```

#### Control Point Calculation

The control point determines the curve's shape and creates momentum:

```rust
pub struct MovementCurve {
    pub start_position: DVec3,
    pub end_position: DVec3,
    pub control_point: DVec3,
    pub last_velocity: DVec3,
}

impl MovementCurve {
    /// Calculate control point for current turn
    pub fn calculate_control_point(
        start: DVec3,
        end: DVec3,
        last_velocity: DVec3,
    ) -> DVec3 {
        if last_velocity.length() < 0.001 {
            // First move - no previous velocity
            // Control point is 1/2.5 along the path
            let offset = end - start;
            start + offset / 2.5
        } else {
            // Use previous velocity for momentum
            // Control point is ahead of start in direction of last velocity
            start + last_velocity / 2.5
        }
    }

    /// Evaluate bezier curve at time t [0.0 to 1.0]
    pub fn evaluate(&self, t: f64) -> DVec3 {
        let t2 = t * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;

        // Quadratic bezier formula
        self.start_position * mt2
            + self.control_point * (2.0 * mt * t)
            + self.end_position * t2
    }

    /// Get velocity at time t (derivative of position)
    pub fn velocity_at(&self, t: f64) -> DVec3 {
        let mt = 1.0 - t;

        // Derivative of quadratic bezier
        2.0 * mt * (self.control_point - self.start_position)
            + 2.0 * t * (self.end_position - self.control_point)
    }

    /// Get ending velocity (for next turn's momentum)
    pub fn ending_velocity(&self) -> DVec3 {
        self.end_position - self.control_point
    }
}
```

#### Visualization During Planning

Show the curved path the ship will take:

```rust
/// Draw the bezier curve as a wireframe path
fn draw_movement_path(curve: &MovementCurve, color: Vec4) {
    const SEGMENTS: usize = 16;
    let mut last_point = curve.start_position;

    for i in 1..=SEGMENTS {
        let t = i as f64 / SEGMENTS as f64;
        let point = curve.evaluate(t);

        // Draw line segment (convert to camera-relative for rendering)
        draw_line(
            world_to_camera(last_point),
            world_to_camera(point),
            color,
            0.05
        );

        last_point = point;
    }

    // Draw control point for debugging
    draw_point(world_to_camera(curve.control_point), Color::YELLOW, 0.2);
}
```

#### Movement Execution

During the 10-second turn simulation:

```rust
pub struct TurnSimulation {
    pub curve: MovementCurve,
    pub start_rotation: DQuat,
    pub end_rotation: DQuat,
    pub duration: f32, // 10.0 seconds
}

impl TurnSimulation {
    /// Update ship position during turn simulation
    pub fn update(&self, elapsed_time: f32, ship: &mut Ship, position: &mut Position) {
        // Calculate progress through turn (0.0 to 1.0)
        let t = (elapsed_time / self.duration).clamp(0.0, 1.0);

        // Update position along bezier curve
        position.0 = self.curve.evaluate(t as f64);

        // Update rotation (spherical interpolation)
        let rotation = self.start_rotation.slerp(self.end_rotation, t as f64);

        // Store rotation in ship component or separate component
        ship.current_rotation = rotation;

        // At end of turn, store velocity for next turn
        if t >= 1.0 {
            ship.last_velocity = self.curve.ending_velocity();
        }
    }
}
```

#### Velocity Inheritance

Each turn's movement is influenced by the previous turn:

```
Turn 1: Ship moves 15 units forward
  → Ending velocity: (0, 0, 15)

Turn 2: Player wants to move right
  → Control point influenced by (0, 0, 15)
  → Ship curves from forward momentum into rightward movement
  → Smooth, realistic path

Turn 3: Player wants to stop
  → Set end position = current position
  → Control point uses previous velocity
  → Ship smoothly decelerates to a stop
```

#### Example Scenarios

**Scenario 1: First Move (No Momentum)**
```rust
start = DVec3::new(0.0, 0.0, 0.0)
end = DVec3::new(10.0, 0.0, 0.0)
last_velocity = DVec3::ZERO

control_point = start + (end - start) / 2.5
              = DVec3::new(4.0, 0.0, 0.0)

// Ship accelerates smoothly to target
```

**Scenario 2: Continuing Momentum**
```rust
start = DVec3::new(10.0, 0.0, 0.0)
end = DVec3::new(20.0, 5.0, 0.0)
last_velocity = DVec3::new(10.0, 0.0, 0.0) // From turn 1

control_point = start + last_velocity / 2.5
              = DVec3::new(14.0, 0.0, 0.0)

// Ship curves from previous momentum toward new target
```

**Scenario 3: Sharp Turn**
```rust
start = DVec3::new(20.0, 5.0, 0.0)
end = DVec3::new(20.0, 5.0, 15.0) // 90-degree turn
last_velocity = DVec3::new(10.0, 5.0, 0.0)

control_point = start + last_velocity / 2.5
              = DVec3::new(24.0, 7.0, 0.0)

// Ship overshoots in old direction before curving to target
// Creates realistic turning behavior
```

#### Arc Length Calculation

For displaying "distance traveled" on UI:

```rust
impl MovementCurve {
    /// Approximate arc length using numerical integration
    pub fn arc_length(&self) -> f64 {
        const STEPS: usize = 100;
        let mut length = 0.0;
        let mut last_point = self.evaluate(0.0);

        for i in 1..=STEPS {
            let t = i as f64 / STEPS as f64;
            let point = self.evaluate(t);
            length += (point - last_point).length();
            last_point = point;
        }

        length
    }
}
```

#### Event Recording

Bezier curve parameters are stored in events:

```rust
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum MovementEvent {
    MovementConfirmed {
        turn: u32,
        ship_id: EntityId,
        start_position: DVec3,
        end_position: DVec3,
        control_point: DVec3,
        last_velocity: DVec3,
        start_rotation: DQuat,
        end_rotation: DQuat,
        timestamp: f64,
    },
}
```

This allows perfect replay of the curve during event playback.

---

## 3D Widget System

### Widget Components

The movement widget consists of 5 interactive elements:

```
         ↑ Up Arrow (Y+)
         |
    [Pitch Cube]      ← 45° angled from back, allows pitch rotation
         |
    [Ship Icon]       ← Ship position preview
         |
    [Yaw Cube]        ← Back cube, allows yaw rotation
         |
         ↓ Down Arrow (Y-)

    Cone (Front)      ← Roll control around forward axis
```

#### 1. Up/Down Arrows (Elevation Control)
```rust
pub struct ElevationArrow {
    pub direction: Vec3,        // UP or DOWN
    pub position: Vec3,         // Relative to ship
    pub length: f32,            // Arrow shaft length
    pub color: Vec4,            // Green for up, red for down
    pub hovered: bool,
}
```

**Interaction**:
- Click and drag arrow vertically
- Raycast to vertical plane facing camera
- Update widget Y position
- Clamp to ±10 units from starting Y

**Visual**:
- Solid arrow when available
- Greyed out when at min/max height
- Glows on hover

#### 2. Front Cone (Roll Control)
```rust
pub struct RollCone {
    pub position: Vec3,         // In front of ship
    pub forward: Vec3,          // Ship forward direction
    pub radius: f32,            // Cone base radius
    pub height: f32,            // Cone height
    pub color: Vec4,            // Yellow
    pub hovered: bool,
}
```

**Interaction**:
- Click and drag cone
- Mouse Y delta rotates ship around forward (Z) axis
- Roll angle unrestricted (full 360°)

**Visual**:
- Wireframe cone pointing forward
- Rotates with ship
- Highlights on hover

#### 3. Back Cube (Yaw Control)
```rust
pub struct YawCube {
    pub position: Vec3,         // Behind ship
    pub size: f32,              // Cube dimensions
    pub color: Vec4,            // Blue
    pub hovered: bool,
}
```

**Interaction**:
- Click and drag cube left/right
- Mouse X delta rotates ship around up (Y) axis
- Constrained to ±45° from starting yaw (90° total range)

**Visual**:
- Wireframe cube
- Fixed position behind ship
- Highlights on hover

#### 4. Pitch Cube (Pitch Control)
```rust
pub struct PitchCube {
    pub position: Vec3,         // Behind and above ship (45° angle)
    pub size: f32,              // Cube dimensions
    pub color: Vec4,            // Cyan
    pub hovered: bool,
}
```

**Interaction**:
- Click and drag cube up/down
- Mouse Y delta rotates ship around right (X) axis
- Constrained to ±45° from starting pitch (90° total range)

**Visual**:
- Wireframe cube positioned at 45° angle
- Highlights on hover

#### 5. Ship Preview Icon
```rust
pub struct ShipPreview {
    pub position: Vec3,         // Widget center
    pub rotation: Quat,         // Planned ship rotation
    pub scale: f32,             // Preview size
    pub color: Vec4,            // Semi-transparent cyan
}
```

**Visual**:
- Translucent hologram of ship model
- Shows planned position and rotation
- Fades in/out when widget is shown/hidden

---

## Movement Range Visualization

### Cylindrical Volume

The movement range is visualized as a wireframe cylinder with horizontal slices:

```
     Y-axis (height)
     ↑
     |    ___________  ← Top slice (Y + 10)
     |   /           \
     |  |   ~~~~~~    |  ← Mid slices (Y + 5, Y + 0, Y - 5)
     |   \___________/  ← Bottom slice (Y - 10)
     |
     +------------------→ X/Z plane (radius 20)
```

#### Cylinder Components
```rust
pub struct MovementRangeCylinder {
    pub center: DVec3,              // Ship starting position
    pub radius: f32,                // Max XZ movement (20 units)
    pub height_min: f32,            // Y - 10
    pub height_max: f32,            // Y + 10
    pub num_slices: u32,            // Number of horizontal rings (5)
    pub num_segments: u32,          // Segments per ring (32)
    pub color: Vec4,                // White with alpha 0.3
}
```

#### Rendering
```rust
fn draw_movement_range_cylinder(cylinder: &MovementRangeCylinder) {
    // Draw vertical edges
    for i in 0..cylinder.num_segments {
        let angle = (i as f32 / cylinder.num_segments as f32) * 2.0 * PI;
        let x = cylinder.radius * angle.cos();
        let z = cylinder.radius * angle.sin();

        let top = cylinder.center + Vec3::new(x, cylinder.height_max, z);
        let bottom = cylinder.center + Vec3::new(x, cylinder.height_min, z);

        draw_line(bottom, top, cylinder.color, 0.02);
    }

    // Draw horizontal slices
    let slice_height = (cylinder.height_max - cylinder.height_min) / (cylinder.num_slices - 1) as f32;

    for slice in 0..cylinder.num_slices {
        let y = cylinder.height_min + slice as f32 * slice_height;

        draw_circle_xz(
            cylinder.center + Vec3::new(0.0, y, 0.0),
            cylinder.radius,
            cylinder.num_segments,
            cylinder.color,
            0.02
        );
    }
}

fn draw_circle_xz(center: Vec3, radius: f32, segments: u32, color: Vec4, width: f32) {
    for i in 0..segments {
        let angle1 = (i as f32 / segments as f32) * 2.0 * PI;
        let angle2 = ((i + 1) as f32 / segments as f32) * 2.0 * PI;

        let p1 = center + Vec3::new(radius * angle1.cos(), 0.0, radius * angle1.sin());
        let p2 = center + Vec3::new(radius * angle2.cos(), 0.0, radius * angle2.sin());

        draw_line(p1, p2, color, width);
    }
}
```

---

## Rotation Constraint Visualization

### 90-Degree Arc

Show the available rotation range as a cone/arc:

```
         Ship Starting Forward
              ↑
             / \     ← 90° arc
            /   \
           /     \
          /_______\   ← Current facing
```

#### Arc Visualization
```rust
pub struct RotationConstraintArc {
    pub center: DVec3,              // Ship position
    pub start_forward: Vec3,        // Starting forward direction
    pub current_forward: Vec3,      // Current planned forward
    pub max_angle: f32,             // 90 degrees (PI/2)
    pub arc_radius: f32,            // Visual radius (5 units)
    pub arc_color: Vec4,            // Yellow with alpha 0.5
}

fn draw_rotation_constraint_arc(arc: &RotationConstraintArc) {
    let num_segments = 32;
    let half_angle = arc.max_angle / 2.0; // 45 degrees each side

    // Calculate right vector perpendicular to start_forward
    let right = arc.start_forward.cross(Vec3::Y).normalize();
    let up = right.cross(arc.start_forward).normalize();

    // Draw arc from -45° to +45°
    for i in 0..num_segments {
        let angle1 = -half_angle + (i as f32 / num_segments as f32) * arc.max_angle;
        let angle2 = -half_angle + ((i + 1) as f32 / num_segments as f32) * arc.max_angle;

        // Rotate start_forward by angle around up axis
        let rot1 = Quat::from_axis_angle(up, angle1);
        let rot2 = Quat::from_axis_angle(up, angle2);

        let dir1 = rot1 * arc.start_forward;
        let dir2 = rot2 * arc.start_forward;

        let p1 = arc.center + dir1 * arc.arc_radius;
        let p2 = arc.center + dir2 * arc.arc_radius;

        draw_line(p1, p2, arc.arc_color, 0.03);
    }

    // Draw lines from center to arc endpoints
    let left_rot = Quat::from_axis_angle(up, -half_angle);
    let right_rot = Quat::from_axis_angle(up, half_angle);

    let left_dir = left_rot * arc.start_forward;
    let right_dir = right_rot * arc.start_forward;

    draw_line(arc.center, arc.center + left_dir * arc.arc_radius, arc.arc_color, 0.03);
    draw_line(arc.center, arc.center + right_dir * arc.arc_radius, arc.arc_color, 0.03);

    // Draw current facing direction
    let current_pos = arc.center + arc.current_forward * arc.arc_radius;
    draw_line(arc.center, current_pos, Color::GREEN, 0.05);
}
```

---

## Event Sourcing

### Movement Event Types

```rust
#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum MovementEvent {
    /// Player started planning movement
    MovementPlanningStarted {
        turn: u32,
        ship_id: EntityId,
        timestamp: f64,
    },

    /// Widget position updated
    WidgetPositionChanged {
        turn: u32,
        ship_id: EntityId,
        new_position: DVec3,
        timestamp: f64,
    },

    /// Widget rotation updated
    WidgetRotationChanged {
        turn: u32,
        ship_id: EntityId,
        new_rotation: DQuat,
        timestamp: f64,
    },

    /// Player confirmed movement
    MovementConfirmed {
        turn: u32,
        ship_id: EntityId,
        final_position: DVec3,
        final_rotation: DQuat,
        timestamp: f64,
    },

    /// Movement cancelled/reset
    MovementCancelled {
        turn: u32,
        ship_id: EntityId,
        timestamp: f64,
    },
}
```

### Event Recording

```rust
pub struct MovementEventRecorder {
    events: Vec<MovementEvent>,
    current_turn: u32,
}

impl MovementEventRecorder {
    pub fn record_widget_position_change(
        &mut self,
        ship_id: EntityId,
        new_position: DVec3,
    ) {
        let event = MovementEvent::WidgetPositionChanged {
            turn: self.current_turn,
            ship_id,
            new_position,
            timestamp: self.get_timestamp(),
        };

        self.events.push(event);
    }

    pub fn record_widget_rotation_change(
        &mut self,
        ship_id: EntityId,
        new_rotation: DQuat,
    ) {
        let event = MovementEvent::WidgetRotationChanged {
            turn: self.current_turn,
            ship_id,
            new_rotation,
            timestamp: self.get_timestamp(),
        };

        self.events.push(event);
    }

    pub fn record_movement_confirmed(
        &mut self,
        ship_id: EntityId,
        final_position: DVec3,
        final_rotation: DQuat,
    ) {
        let event = MovementEvent::MovementConfirmed {
            turn: self.current_turn,
            ship_id,
            final_position,
            final_rotation,
            timestamp: self.get_timestamp(),
        };

        self.events.push(event);
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), Error> {
        let json = serde_json::to_string_pretty(&self.events)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn load_from_file(path: &str) -> Result<Self, Error> {
        let json = std::fs::read_to_string(path)?;
        let events = serde_json::from_str(&json)?;
        Ok(Self {
            events,
            current_turn: 0,
        })
    }
}
```

---

## Timeline UI

### Timeline Structure

```
Turn 1 |=====[Widget Move]==[Rotation]==[Confirm]====| 10s
Turn 2 |=====[Widget Move]==============[Confirm]====| 10s
Turn 3 |                                             | (planning)
       0s    2s      4s      6s      8s      10s
```

### Timeline Components

```rust
pub struct MovementTimeline {
    /// Current turn being planned
    pub current_turn: u32,

    /// All recorded events
    pub events: Vec<MovementEvent>,

    /// Playback position (0.0 to total_duration)
    pub playback_position: f32,

    /// Total duration in seconds
    pub total_duration: f32,

    /// Is timeline playing?
    pub is_playing: bool,
}

impl MovementTimeline {
    pub fn render(&self, ui: &Ui) {
        ui.window("Movement Timeline")
            .size([800.0, 120.0], imgui::Condition::FirstUseEver)
            .build(|| {
                // Current turn indicator
                ui.text(format!("Turn {} - Planning Phase", self.current_turn));

                // Playback position slider
                let mut pos = self.playback_position;
                if ui.slider("##timeline", 0.0, self.total_duration, &mut pos) {
                    // User scrubbed timeline - update playback position
                    self.on_scrub(pos);
                }

                // Event markers
                self.render_event_markers(ui);

                // Playback controls
                ui.spacing();
                if self.is_playing {
                    if ui.button("⏸ Pause") {
                        self.pause();
                    }
                } else {
                    if ui.button("▶ Replay") {
                        self.play();
                    }
                }

                ui.same_line();
                if ui.button("Confirm Move") {
                    self.confirm_movement();
                }
            });
    }

    fn render_event_markers(&self, ui: &Ui) {
        let window_width = ui.window_size()[0];
        let draw_list = ui.get_window_draw_list();
        let cursor_pos = ui.cursor_screen_pos();

        for event in &self.events {
            // Only show events for current turn
            if event.turn() != self.current_turn {
                continue;
            }

            let time = event.timestamp() - self.get_turn_start_time();
            let x = (time / 10.0) * window_width;

            let (color, icon) = match event {
                MovementEvent::WidgetPositionChanged { .. } => ([0.0, 1.0, 1.0, 1.0], "●"),
                MovementEvent::WidgetRotationChanged { .. } => ([1.0, 1.0, 0.0, 1.0], "◆"),
                MovementEvent::MovementConfirmed { .. } => ([0.0, 1.0, 0.0, 1.0], "✓"),
                _ => ([1.0, 1.0, 1.0, 1.0], "○"),
            };

            // Draw marker
            draw_list.add_circle(
                [cursor_pos[0] + x, cursor_pos[1] + 30.0],
                4.0,
                color,
            ).filled(true).build();

            // Tooltip on hover
            if self.is_mouse_over_marker(cursor_pos, x, 30.0) {
                ui.tooltip(|| {
                    ui.text(format!("{:?}", event));
                    ui.text(format!("Time: {:.2}s", time));
                });
            }
        }
    }
}
```

---

## Ship Component

### ECS Component Definition

```rust
#[derive(Clone, Debug)]
pub struct Ship {
    /// Movement capabilities
    pub max_movement_range: f32,        // 20 units default
    pub max_rotation_angle: f32,        // 90 degrees (PI/2) default
    pub max_elevation_change: f32,      // 10 units default

    /// Current state
    pub confirmed_move: bool,
    pub movement_locked: bool,

    /// Planned movement (what widget shows)
    pub planned_position: DVec3,
    pub planned_rotation: DQuat,

    /// Starting position (for constraint checks)
    pub turn_start_position: DVec3,
    pub turn_start_rotation: DQuat,
}

impl Ship {
    pub fn new() -> Self {
        Self {
            max_movement_range: 20.0,
            max_rotation_angle: std::f32::consts::FRAC_PI_2, // 90 degrees
            max_elevation_change: 10.0,
            confirmed_move: false,
            movement_locked: false,
            planned_position: DVec3::ZERO,
            planned_rotation: DQuat::IDENTITY,
            turn_start_position: DVec3::ZERO,
            turn_start_rotation: DQuat::IDENTITY,
        }
    }

    /// Check if planned position is within movement range
    pub fn is_position_valid(&self, position: DVec3) -> bool {
        let offset = position - self.turn_start_position;

        // Check XZ distance (radius)
        let xz_distance = (offset.x * offset.x + offset.z * offset.z).sqrt();
        if xz_distance > self.max_movement_range as f64 {
            return false;
        }

        // Check Y distance (elevation)
        if offset.y.abs() > self.max_elevation_change as f64 {
            return false;
        }

        true
    }

    /// Check if planned rotation is within rotation constraint
    pub fn is_rotation_valid(&self, rotation: DQuat) -> bool {
        // Calculate angle difference between start and planned rotation
        let angle_diff = self.turn_start_rotation.angle_between(rotation);

        angle_diff <= self.max_rotation_angle as f64
    }

    /// Clamp position to valid range
    pub fn clamp_position(&self, position: DVec3) -> DVec3 {
        let mut offset = position - self.turn_start_position;

        // Clamp XZ to radius
        let xz_distance = (offset.x * offset.x + offset.z * offset.z).sqrt();
        if xz_distance > self.max_movement_range as f64 {
            let scale = self.max_movement_range as f64 / xz_distance;
            offset.x *= scale;
            offset.z *= scale;
        }

        // Clamp Y to elevation
        offset.y = offset.y.clamp(
            -self.max_elevation_change as f64,
            self.max_elevation_change as f64,
        );

        self.turn_start_position + offset
    }

    /// Clamp rotation to valid range
    pub fn clamp_rotation(&self, rotation: DQuat) -> DQuat {
        let angle_diff = self.turn_start_rotation.angle_between(rotation);

        if angle_diff <= self.max_rotation_angle as f64 {
            return rotation;
        }

        // Slerp to max allowed angle
        let t = self.max_rotation_angle as f64 / angle_diff;
        self.turn_start_rotation.slerp(rotation, t)
    }
}
```

---

## Widget Interaction System

### Ray Picking

```rust
pub struct WidgetInteraction {
    /// Currently hovered widget element
    pub hovered_element: Option<WidgetElement>,

    /// Currently dragging element
    pub dragging_element: Option<WidgetElement>,

    /// Drag start info
    pub drag_start_mouse: Vec2,
    pub drag_start_value: Vec3,
}

pub enum WidgetElement {
    UpArrow,
    DownArrow,
    RollCone,
    YawCube,
    PitchCube,
    MovementPlane,
}

impl WidgetInteraction {
    pub fn update(&mut self, camera: &Camera, mouse_pos: Vec2, mouse_button_down: bool) {
        let ray = camera.screen_to_ray(mouse_pos);

        // Check hover
        if !mouse_button_down {
            self.hovered_element = self.ray_pick_widget(&ray);
        }

        // Handle drag
        if mouse_button_down {
            if self.dragging_element.is_none() && self.hovered_element.is_some() {
                // Start drag
                self.dragging_element = self.hovered_element;
                self.drag_start_mouse = mouse_pos;
            }

            if let Some(element) = &self.dragging_element {
                self.handle_drag(element, &ray, mouse_pos);
            }
        } else {
            // End drag
            self.dragging_element = None;
        }
    }

    fn ray_pick_widget(&self, ray: &Ray) -> Option<WidgetElement> {
        // Test ray intersection with each widget element
        // Return closest hit

        // Test arrows
        if self.ray_intersects_arrow(ray, WidgetElement::UpArrow) {
            return Some(WidgetElement::UpArrow);
        }
        if self.ray_intersects_arrow(ray, WidgetElement::DownArrow) {
            return Some(WidgetElement::DownArrow);
        }

        // Test cone
        if self.ray_intersects_cone(ray) {
            return Some(WidgetElement::RollCone);
        }

        // Test cubes
        if self.ray_intersects_cube(ray, WidgetElement::YawCube) {
            return Some(WidgetElement::YawCube);
        }
        if self.ray_intersects_cube(ray, WidgetElement::PitchCube) {
            return Some(WidgetElement::PitchCube);
        }

        // Test movement plane
        if self.ray_intersects_plane(ray) {
            return Some(WidgetElement::MovementPlane);
        }

        None
    }

    fn handle_drag(&mut self, element: &WidgetElement, ray: &Ray, mouse_pos: Vec2) {
        match element {
            WidgetElement::UpArrow | WidgetElement::DownArrow => {
                self.handle_elevation_drag(ray);
            }
            WidgetElement::RollCone => {
                let delta_y = mouse_pos.y - self.drag_start_mouse.y;
                self.handle_roll_drag(delta_y);
            }
            WidgetElement::YawCube => {
                let delta_x = mouse_pos.x - self.drag_start_mouse.x;
                self.handle_yaw_drag(delta_x);
            }
            WidgetElement::PitchCube => {
                let delta_y = mouse_pos.y - self.drag_start_mouse.y;
                self.handle_pitch_drag(delta_y);
            }
            WidgetElement::MovementPlane => {
                self.handle_position_drag(ray);
            }
        }
    }

    fn handle_elevation_drag(&mut self, ray: &Ray) {
        // Raycast to vertical plane facing camera
        let camera_forward_xz = Vec3::new(ray.direction.x, 0.0, ray.direction.z).normalize();
        let plane_normal = camera_forward_xz;
        let plane = Plane::new(plane_normal, self.widget_position);

        if let Some(hit_point) = ray.intersect_plane(&plane) {
            let new_y = hit_point.y;
            self.update_widget_elevation(new_y);
        }
    }

    fn handle_position_drag(&mut self, ray: &Ray) {
        // Raycast to horizontal plane at widget Y
        let plane = Plane::new(Vec3::Y, Vec3::new(0.0, self.widget_position.y, 0.0));

        if let Some(hit_point) = ray.intersect_plane(&plane) {
            self.update_widget_position(hit_point);
        }
    }
}
```

---

## Rendering Pipeline

### Wireframe Shader Requirements

We'll need a simple wireframe shader for rendering the widget and constraints:

```glsl
// wireframe.vert
#version 450

layout(location = 0) in vec3 position;

layout(push_constant) uniform PushConstants {
    mat4 model;
    mat4 view;
    mat4 projection;
    vec4 color;
} push;

void main() {
    gl_Position = push.projection * push.view * push.model * vec4(position, 1.0);
}

// wireframe.frag
#version 450

layout(push_constant) uniform PushConstants {
    mat4 model;
    mat4 view;
    mat4 projection;
    vec4 color;
} push;

layout(location = 0) out vec4 outColor;

void main() {
    outColor = push.color;
}
```

---

## Implementation Order

### Phase 1.1: Basic Components (Week 1)
1. Add `Ship` component to ECS
2. Create `MovementEvent` enum
3. Create `MovementEventRecorder`
4. Add event recording to game loop
5. Save/load events to JSON

### Phase 1.2: Widget Rendering (Week 1-2)
1. Create wireframe vertex buffers for:
   - Arrow geometry
   - Cone geometry
   - Cube geometry
   - Cylinder slices
   - Rotation arc
2. Implement wireframe shader
3. Render widget components
4. Render movement range cylinder
5. Render rotation constraint arc

### Phase 1.3: Widget Interaction (Week 2)
1. Implement ray picking for widget elements
2. Handle mouse hover states
3. Implement drag handling for each element:
   - Elevation arrows
   - Roll cone
   - Yaw cube
   - Pitch cube
   - Movement plane
4. Apply movement/rotation constraints
5. Record events during interaction

### Phase 1.4: Timeline UI (Week 2-3)
1. Create timeline window
2. Render event markers
3. Implement playback scrubbing
4. Add playback controls
5. Implement movement confirmation

### Phase 1.5: Integration & Polish (Week 3)
1. Wire up ship selection to show widget
2. Connect timeline to game turns
3. Add visual feedback (colors, highlights)
4. Test and debug interaction
5. Add keyboard shortcuts

---

## Testing Plan

### Unit Tests
- ✅ Ship.is_position_valid() with various positions
- ✅ Ship.is_rotation_valid() with various rotations
- ✅ Ship.clamp_position() edge cases
- ✅ Ship.clamp_rotation() edge cases
- ✅ Event serialization/deserialization

### Integration Tests
- ✅ Widget position updates trigger events
- ✅ Widget rotation updates trigger events
- ✅ Movement confirmation creates final event
- ✅ Timeline playback recreates widget state
- ✅ Events save/load correctly

### Manual Tests
- ✅ Widget appears when ship selected
- ✅ Cylinder visualizes movement range
- ✅ Arc visualizes rotation constraint
- ✅ Dragging elements feels responsive
- ✅ Constraints prevent invalid moves
- ✅ Timeline shows all events
- ✅ Playback matches original actions

---

## Success Criteria

Phase 1 is complete when:
- ✅ Player can select a ship and see movement widget
- ✅ Player can drag widget elements to plan movement
- ✅ Movement is constrained to cylindrical volume
- ✅ Rotation is constrained to 90 degrees
- ✅ All interactions are recorded as events
- ✅ Events are saved to/loaded from JSON
- ✅ Timeline shows event history
- ✅ Timeline playback recreates movement planning

---

## File Structure

```
src/
  ecs/
    components.rs      (add Ship component)
  movement/
    mod.rs             (movement system module)
    events.rs          (MovementEvent, MovementEventRecorder)
    widget.rs          (widget components and rendering)
    interaction.rs     (ray picking, drag handling)
    constraints.rs     (movement/rotation validation)
    timeline.rs        (timeline UI and playback)
  shaders/
    wireframe.vert     (wireframe vertex shader)
    wireframe.frag     (wireframe fragment shader)
```

---

## Notes for Implementation

1. **64-bit Coordinates**: Widget position uses DVec3, but rendering uses camera-relative Vec3
2. **Event Granularity**: Don't record every mouse move - throttle to 10-20 events per second max
3. **Undo/Redo**: Event sourcing naturally supports undo by removing events
4. **Network Sync**: Events are small and easily transmitted for multiplayer
5. **Determinism**: Keep all randomness out of movement for deterministic replay

---

## Next Phases Preview

**Phase 2**: Movement modes, bezier curves, velocity inheritance
**Phase 3**: Physics simulation, collision detection
**Phase 4**: Particles, visual effects, snapshots
**Phase 5**: Weapons, targeting, combat events

---

## Renderer Refactoring (PRIORITY)

### Problem
The current renderer.rs is **4877 lines** and violates SOLID principles:
- **Single Responsibility**: Handles skybox, nebula, meshes, gizmo, SSAO, ImGui, stars, etc.
- **Open/Closed**: Cannot add new rendering features without modifying renderer
- **Dependency Inversion**: Concrete implementations hard-coded instead of injected

### Solution: Plugin-Based Render Pass System

Create a **RenderPass trait** that each rendering system implements, then register passes as plugins.

#### Architecture

```rust
// Core trait - each feature implements this
pub trait RenderPass {
    fn initialize(&mut self, ctx: &RenderContext, render_pass: vk::RenderPass, extent: vk::Extent2D) -> Result<()>;
    fn update(&mut self, ctx: &RenderContext, frame_index: usize, game: &Game) -> Result<()>;
    fn render(&mut self, ctx: &RenderContext, command_buffer: vk::CommandBuffer, frame_index: usize, game: &Game) -> Result<()>;
    fn recreate_swapchain(&mut self, ctx: &RenderContext, render_pass: vk::RenderPass, extent: vk::Extent2D) -> Result<()>;
    fn cleanup(&mut self, device: &ash::Device);
    fn name(&self) -> &str;
    fn should_render(&self, game: &Game) -> bool { true }
}

// Registry manages all passes
pub struct RenderPassRegistry {
    passes: Vec<Box<dyn RenderPass>>,
}

impl RenderPassRegistry {
    pub fn register(&mut self, pass: Box<dyn RenderPass>);
    pub fn render_all(&mut self, ctx: &RenderContext, command_buffer: vk::CommandBuffer, frame_index: usize, game: &Game) -> Result<()>;
}
```

#### Example Pass Implementation

```rust
// src/core/passes/skybox.rs
pub struct SkyboxPass {
    pipeline: vk::Pipeline,
    pipeline_layout: vk::PipelineLayout,
    descriptor_sets: Vec<vk::DescriptorSet>,
    // ... skybox-specific resources
}

impl RenderPass for SkyboxPass {
    fn initialize(&mut self, ctx: &RenderContext, render_pass: vk::RenderPass, extent: vk::Extent2D) -> Result<()> {
        // Create skybox pipeline, descriptor sets, etc.
        Ok(())
    }

    fn render(&mut self, ctx: &RenderContext, command_buffer: vk::CommandBuffer, frame_index: usize, game: &Game) -> Result<()> {
        unsafe {
            ctx.device.cmd_bind_pipeline(command_buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline);
            // ... render skybox
        }
        Ok(())
    }

    fn should_render(&self, game: &Game) -> bool {
        game.is_skybox_visible()
    }

    fn name(&self) -> &str { "Skybox" }
}
```

#### Renderer Becomes Thin Coordinator

```rust
pub struct VulkanRenderer {
    // Core Vulkan resources only
    device: ash::Device,
    instance: ash::Instance,
    swapchain: vk::SwapchainKHR,
    render_pass: vk::RenderPass,
    // ...

    // Plugin registry
    render_passes: RenderPassRegistry,
}

impl VulkanRenderer {
    pub fn new(window: &Window) -> Result<Self> {
        // Initialize core Vulkan resources
        // ...

        // Register all render passes as plugins
        let mut render_passes = RenderPassRegistry::new();
        render_passes.register(Box::new(SkyboxPass::new()));
        render_passes.register(Box::new(NebulaPass::new()));
        render_passes.register(Box::new(MeshPass::new()));
        render_passes.register(Box::new(StarPass::new()));
        render_passes.register(Box::new(GizmoPass::new()));
        render_passes.register(Box::new(SSAOPass::new()));
        render_passes.register(Box::new(ImGuiPass::new()));

        // Initialize all passes
        let ctx = RenderContext { device, instance, physical_device, command_pool, graphics_queue };
        render_passes.initialize_all(&ctx, render_pass, extent)?;

        Ok(Self { device, instance, swapchain, render_pass, render_passes, ... })
    }

    pub fn render(&mut self, game: &mut Game) -> Result<()> {
        // Begin frame, acquire image, begin render pass
        // ...

        // Render all passes (automatically skips if should_render() returns false)
        let ctx = RenderContext { device, instance, physical_device, command_pool, graphics_queue };
        self.render_passes.render_all(&ctx, command_buffer, frame_index, game)?;

        // End render pass, submit, present
        // ...
        Ok(())
    }
}
```

### Benefits

1. **Separation of Concerns**: Each pass owns its own resources and logic
2. **Easy to Add Features**: Just implement RenderPass and register it
3. **Easy to Remove Features**: Delete the pass file and unregister
4. **Testable**: Can test each pass independently
5. **Maintainable**: ~200 lines per pass vs 4877 lines monolith
6. **Flexible**: Can reorder passes, enable/disable at runtime, add passes at runtime

### File Structure

```
src/core/
  renderer.rs          (~500 lines - core Vulkan only)
  render_pass.rs       (~150 lines - trait + registry)
  passes/
    mod.rs
    skybox.rs          (~200 lines)
    nebula.rs          (~300 lines)
    mesh.rs            (~250 lines)
    star.rs            (~200 lines)
    gizmo.rs           (~150 lines)
    ssao.rs            (~400 lines)
    imgui.rs           (~100 lines)
```

Total: ~2250 lines spread across 9 files vs 4877 lines in one file

### Migration Plan

1. ✅ Create RenderPass trait and registry
2. Extract skybox to SkyboxPass
3. Extract nebula to NebulaPass
4. Extract mesh rendering to MeshPass
5. Extract star rendering to StarPass
6. Extract gizmo to GizmoPass
7. Extract SSAO to SSAOPass
8. Extract ImGui to ImGuiPass
9. Slim down renderer.rs to core only
10. Test all passes work
11. Add ability to register passes at runtime

### Implementation Status

- ✅ Created `src/core/render_pass.rs` - trait and registry
- ⏳ Extracting passes...
- ⏳ Testing...

This refactoring is **critical** before implementing the widget system. The widget rendering should be its own pass, not embedded in the monolithic renderer.
