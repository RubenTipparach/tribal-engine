# Turn-Based Movement System Documentation

## Overview
This document describes the turn-based tactical movement system for space combat. The system provides players with 10 seconds per turn to plan and execute ship maneuvers, with a holographic preview system showing predicted movement paths.

---

## Core Concepts

### Turn Structure
- **Turn Duration**: 10 seconds of real-time simulation
- **Planning Phase**: Player plans movement while simulation is paused
- **Execution Phase**: All planned movements are simulated simultaneously over 10 seconds
- **Movement Modes**: 4 distinct modes that affect ship behavior

### Ship Components
Ships with the `Ship` component can have different capabilities:
- **Moveable Ships** (e.g., Federation Cruiser): Can move, rotate, and have health/stats
- **Static Ships** (e.g., Cube 2): Has Ship component but cannot move (`moveable = false`)

---

## Movement System Parameters

### Ship Statistics
Each ship has the following movement-related parameters:

```rust
pub struct Ship {
    // Movement capabilities
    pub max_thruster_range: f32,        // Base movement range per turn (default: 20.0 units)
    pub max_rotation_per_turn: f32,     // Maximum rotation angle per turn (degrees)
    pub moveable: bool,                 // Whether ship can move at all

    // Combat stats
    pub health: f32,
    pub max_health: f32,

    // State flags
    pub has_boosted: bool,              // Used Full Speed this turn
    pub initiated_full_stop: bool,      // Used Full Stop this turn
    pub confirmed_move: bool,           // Player has confirmed their move
}
```

### Movement Modes

#### 1. MOVE_AND_TURN (Default)
- **Description**: Standard maneuver mode allowing both position and rotation changes
- **Movement Range**: Base `max_thruster_range` (20 units default)
- **Rotation**: Full rotation freedom
- **Velocity Inheritance**: Uses Bezier curve with previous velocity for smooth momentum
- **When Available**: Always available
- **Next Turn**: Can transition to any mode

#### 2. FULL_SPEED (Boost)
- **Description**: Double-speed thrust in current facing direction
- **Movement Range**: `max_thruster_range * 2.0` (40 units)
- **Rotation**: Ship faces movement direction, z-roll preserved
- **Velocity Inheritance**: High momentum carried into next turn
- **When Available**: Only after MOVE_AND_TURN mode, once per sequence
- **Next Turn**: Can transition to MOVE_AND_TURN or FULL_STOP
- **Restrictions**: Sets `has_boosted = true`, locks direction to facing

#### 3. FULL_STOP (Deceleration)
- **Description**: Emergency braking maneuver
- **Movement Range**: `max_thruster_range / 2.0` (10 units)
- **Rotation**: Maintains last rotation (no rotation change)
- **Velocity Inheritance**: Applies counter-thrust, reduces momentum by 50% per turn
- **When Available**: When moving (not already stopped)
- **Next Turn**: Continues with reduced momentum until velocity ~= 0
- **Countdown System**: `full_stop_countdown` decrements each turn
- **Special Behavior**:
  - If used after FULL_SPEED: Reduces to half the boosted momentum
  - Otherwise: Reduces current velocity by 50%
  - Control point set to `-lastVelocity * 2` for counter-thrust

#### 4. TURN_SLIDE (Drift)
- **Description**: Rotate without changing velocity vector (slide/strafe)
- **Movement Range**: Continues with previous velocity vector
- **Rotation**: Full rotation freedom
- **Velocity Inheritance**: Previous velocity vector maintained unchanged
- **When Available**: Always available
- **Next Turn**: Maintains momentum, can transition to any mode
- **Use Case**: Position ship facing for weapons while maintaining trajectory

---

## Movement Planning Phase

### 1. Ship Selection
```
Player clicks on ship → Ship becomes selected → Movement widget appears
```

### 2. Widget Components
The movement widget consists of:
- **Position Widget**: Translucent sphere/marker at projected end position
- **Elevation Arrows**: Up/down arrows for Y-axis adjustment
- **Rotation Ring**: Circle widget for pitch/yaw rotation
- **Roll Arrows**: Side arrows for z-axis roll

### 3. Widget Interaction

#### Position Manipulation
```
Mouse drag on empty space:
- Raycast to horizontal plane at current widget Y position
- Move widget position in XZ plane
- Auto-rotate ship to face movement direction (MOVE_AND_TURN mode)
- Clamp position to max_thruster_range sphere
```

#### Elevation Control
```
Mouse drag on elevation arrows:
- Raycast to vertical plane facing camera
- Adjust Y position of widget
- Update ship rotation to face new target
- Maintain XZ distance constraints
```

#### Rotation Control
```
Mouse drag on rotation ring:
- Free rotation around widget position
- Pitch (X-axis): Mouse Y movement
- Yaw (Y-axis): Mouse X movement
- Preserves z-roll value
```

#### Roll Control
```
Mouse drag on roll arrows:
- Rotate around forward axis (Z-axis)
- Mouse Y movement = roll angle
- Roll sensitivity multiplier applied
```

### 4. Movement Mode Selection UI
Four buttons displayed:
- **Move & Turn**: Default mode button
- **Full Speed**: Only enabled after using Move & Turn
- **Turn & Slide**: Always enabled
- **Full Stop**: Only enabled when ship has velocity
- **Reset**: Reverts to last confirmed move

Button states update based on ship's current mode and flags.

---

## Visualization System

### 1. Movement Arc (Trajectory Line)
```
Planning Phase:
- Draw bezier curve from current position to widget position
- Use 8-16 iterations for smooth curve
- Color: Cyan/Blue for player ships
- Includes velocity inheritance control point

Simulation Phase:
- Draw remaining trajectory based on simulation progress
- Fade/remove traveled portion of arc
- Show "future path" from current position to end position
```

### 2. Holographic Ship Preview
```
Conditions for display:
- Timeline scrubber > 0.5 seconds
- Timeline scrubber < 9.5 seconds (for player ships)
- Planning phase active

Display:
- Translucent/wireframe version of ship model
- Position: Interpolate along movement bezier curve
- Rotation: Slerp between start and end rotation
- Updates in real-time as timeline scrubber moves
```

### 3. Forward Cone Indicator
```
At end position:
- Draw cone/arrow showing ship facing direction
- Color coded (cyan for player, red for enemy)
- Size: proportional to ship size
- Dash pattern for non-confirmed moves
```

### 4. Velocity Vector Lines
```
Draw wireframe lines showing:
- Previous velocity vector (dashed, grey)
- Current planned velocity vector (solid, white)
- Difference vector when mode changes (yellow)
```

---

## Bezier Curve Movement

### Mathematical Implementation

The system uses quadratic Bezier curves for smooth, momentum-based movement:

```
P(t) = (1-t)² * P0 + 2(1-t)t * P1 + t² * P2

Where:
- P0 = Start position (current ship position)
- P1 = Control point (momentum influence)
- P2 = End position (widget/target position)
- t = time progress [0.0 to 1.0]
```

### Control Point Calculation

#### Standard Movement (MOVE_AND_TURN, TURN_SLIDE)
```
lastVelocity = previous turn's (endPosition - startPosition)
controlPoint = startPosition + (lastVelocity / 2.5)

Effect: Curves toward previous velocity direction before reaching target
```

#### First Move (No Previous Velocity)
```
lastVelocity = targetPosition - startPosition
controlPoint = startPosition + (lastVelocity / 2.5)

Effect: Smooth acceleration toward target
```

#### Full Stop
```
controlPoint = startPosition - (lastVelocity * 2)

Effect: Counter-thrust against momentum, strong deceleration curve
```

#### Drifting (Engine Failure)
```
driftDirection = (lastVelocity * 0.25)
controlPoint = startPosition + (driftDirection * 0.5)

Effect: Continues in last direction with 25% momentum, no control
```

### Velocity Inheritance
```
At turn end:
lastVelocity = P2 - P1 (target position - control point)

This becomes the momentum for next turn's control point calculation
```

---

## Simulation Execution

### 1. Turn Start (`OnStartSim`)
```rust
fn on_start_sim() {
    // Capture starting state
    position_start = current_position;
    position_target = widget_position;
    rotation_start = current_rotation;
    rotation_target = widget_rotation;

    // Calculate control point based on movement mode
    calculate_control_point();

    // Store for reset capability
    last_move = (position_offset, rotation, movement_mode);

    // Check engine status
    if thrusters_disabled {
        initiate_drift_mode();
    }
}
```

### 2. Simulation Update (Each Frame)
```rust
fn update_sim(time_percent: f32) {
    // time_percent goes from 0.0 to 1.0 over 10 seconds

    // Position update (Bezier curve)
    current_position = bezier_curve(
        position_start,
        control_point,
        position_target,
        time_percent
    );

    // Rotation update (Spherical interpolation)
    if !drifting {
        current_rotation = slerp(
            rotation_start,
            rotation_target,
            time_percent
        );
    }

    // Visual effects (thruster particles, etc.)
    update_thruster_fx(time_percent * 10.0); // Convert to seconds
}
```

### 3. Turn End (`OnStopSim`)
```rust
fn on_stop_sim() {
    // Finalize position/rotation
    snap_to_target_position();
    snap_to_target_rotation();

    // Calculate momentum for next turn
    update_velocity_vector();

    // Handle special modes
    if initiated_full_stop {
        reduce_momentum_by_half();
        decrement_stop_counter();
    }

    // Reset widget to new position
    widget_position = current_position + position_offset;

    // Reset confirmation state
    confirmed_move = true;  // Auto-confirm for next turn planning
}
```

---

## Timeline Scrubber System

### Purpose
Allows player to preview exact ship positions at any point during the 10-second turn.

### Implementation
```
Timeline UI: 0s [========|====] 10s
                        ^ scrubber at 5.2 seconds

When scrubber moves:
1. Calculate time_percent = scrubber_time / 10.0
2. For each ship:
   - position = bezier_curve(start, control, end, time_percent)
   - rotation = slerp(start_rot, end_rot, time_percent)
3. Update holographic preview position
4. Show collision predictions if any
```

### User Interaction
- **Click + Drag**: Scrub through timeline
- **Space Bar**: Play/Pause simulation
- **Release**: Return to real-time or stay paused
- **Visual Feedback**: Timeline shows weapon firing times, collision warnings

---

## Edge Cases & Special Behaviors

### 1. Engine Destruction
```
When thrusters reach 0 HP:
- Ship enters drift mode (autoDrift = true)
- Movement locked to: last_velocity * 0.25
- No rotation changes
- Widget disabled/greyed out
- Continues drifting each turn until repaired
```

### 2. Collision During Movement
```
If collision detected during simulation:
- Continue along bezier path (momentum maintained)
- Apply collision damage
- If collision is head-on and severe:
  - Reduce velocity by collision factor
  - Update control point for next turn
```

### 3. Momentum Chains
```
Turn 1: MOVE_AND_TURN (20 units) → velocity = V1
Turn 2: FULL_SPEED (40 units) → velocity = V2 (2x magnitude)
Turn 3: TURN_SLIDE → velocity = V2 (maintained)
Turn 4: FULL_STOP → velocity = V2 * 0.5
Turn 5: FULL_STOP → velocity = V2 * 0.25
Turn 6: MOVE_AND_TURN → velocity can change direction
```

### 4. Movement Confirmation
```
Player must explicitly confirm move by:
- Clicking "Confirm Move" button, OR
- Clicking "End Turn" button

Until confirmed:
- Widget remains interactive
- Arc preview shows "unconfirmed" visual style
- Can use "Reset" to revert to last turn's ending state

After confirmation:
- Widget becomes non-interactive
- Arc preview shows "locked" visual style
- Can only view, not modify
```

### 5. AI Ship Movement
```
AI ships:
- Plan entire move instantly when turn starts
- No gradual widget adjustment
- Widget appears at final position immediately
- confirmed_move = true by default
- No "Reset" capability
```

---

## Coordinate System Considerations

Given the 64-bit coordinate system:
- All movement calculations use **camera-relative coordinates**
- Widget positions are relative to ship's 64-bit world position
- Bezier curves operate in 32-bit camera space
- Final positions converted back to 64-bit world space
- Precision maintained by re-centering camera periodically

---

## Wireframe Rendering

### Movement Arc
```rust
// Draw bezier curve as line segments
fn draw_movement_arc(start: Vec3, control: Vec3, end: Vec3) {
    let segments = 16;
    let mut last_point = start;

    for i in 1..=segments {
        let t = i as f32 / segments as f32;
        let point = bezier(start, control, end, t);

        // Draw line segment
        draw_line(last_point, point, Color::CYAN, line_width: 0.05);
        last_point = point;
    }
}
```

### Rotation Arc
```rust
// Show rotation change as circular arc
fn draw_rotation_arc(ship_pos: Vec3, start_rot: Quat, end_rot: Quat) {
    let radius = 2.0; // Visual radius around ship
    let segments = 12;

    for i in 0..segments {
        let t1 = i as f32 / segments as f32;
        let t2 = (i + 1) as f32 / segments as f32;

        let rot1 = Quat::slerp(start_rot, end_rot, t1);
        let rot2 = Quat::slerp(start_rot, end_rot, t2);

        let p1 = ship_pos + rot1 * Vec3::FORWARD * radius;
        let p2 = ship_pos + rot2 * Vec3::FORWARD * radius;

        draw_line(p1, p2, Color::YELLOW, line_width: 0.03);
    }
}
```

### Max Range Indicator
```rust
// Draw sphere showing maximum movement range
fn draw_max_range_indicator(ship_pos: Vec3, max_range: f32) {
    draw_wireframe_sphere(ship_pos, max_range, Color::WHITE.with_alpha(0.3));
}
```

---

## UI Implementation Notes

### Movement Mode Panel
```
[Move & Turn] [Full Speed] [Turn & Slide] [Full Stop] [Reset]
     ✓             X            ✓            ✓          ✓

✓ = Available/Selected
X = Disabled (requirements not met)

Below buttons:
- Current Range: 20.0 / 20.0 units
- Movement Mode: MOVE_AND_TURN
- Velocity: 15.3 units/turn
```

### Timeline Panel
```
Turn 3 | [==========|====================] | 10.0s
       0s         5.0s                    10.0s

       🔫 3.2s - Weapon Fire
       ⚠️ 7.1s - Collision Warning
```

### Ship Status Panel
```
Federation Cruiser
HP: 750 / 1000
Thrusters: ████████░░ 80%
Weapons: Online
Movement: Confirmed ✓
```

---

## Implementation Checklist

### Phase 1: Basic Movement
- [ ] Add Ship component to ECS
- [ ] Implement shipMovementEstimator (hologram position)
- [ ] Create movement widget (position sphere)
- [ ] Implement position clamping to max_thruster_range
- [ ] Add MOVE_AND_TURN mode

### Phase 2: Bezier Movement
- [ ] Implement quadratic bezier curve function
- [ ] Add velocity inheritance system
- [ ] Create control point calculation
- [ ] Implement smooth simulation update

### Phase 3: Movement Modes
- [ ] Add FULL_SPEED mode
- [ ] Add FULL_STOP mode with deceleration
- [ ] Add TURN_SLIDE mode
- [ ] Implement mode transition rules

### Phase 4: Visualization
- [ ] Draw movement arc (bezier curve)
- [ ] Draw holographic ship preview
- [ ] Draw forward cone indicator
- [ ] Draw velocity vector lines
- [ ] Add max range sphere indicator

### Phase 5: Widget Interaction
- [ ] Implement position dragging
- [ ] Add elevation control (up/down arrows)
- [ ] Add rotation control (rotation ring)
- [ ] Add roll control (roll arrows)
- [ ] Clamp widget to movement range

### Phase 6: UI Panels
- [ ] Create movement mode selection panel
- [ ] Add timeline scrubber
- [ ] Add ship status display
- [ ] Implement confirm/reset buttons
- [ ] Show available modes based on ship state

### Phase 7: Timeline System
- [ ] Implement 10-second timeline
- [ ] Add scrubber for time preview
- [ ] Show ship positions at scrubber time
- [ ] Display events on timeline (weapon fire, etc.)

### Phase 8: Polish
- [ ] Add wireframe shaders
- [ ] Implement hologram shader
- [ ] Add movement mode transition animations
- [ ] Sound effects for mode changes
- [ ] Tutorial tooltips

---

## Testing Scenarios

### Test 1: Basic Movement
1. Select Federation Cruiser
2. Drag movement widget 15 units forward
3. Confirm move
4. Execute turn
5. Verify ship reaches target position smoothly

### Test 2: Full Speed Boost
1. Move forward 20 units (MOVE_AND_TURN)
2. Confirm and execute
3. Select FULL_SPEED mode
4. Verify range increases to 40 units
5. Execute turn
6. Verify high momentum on next turn

### Test 3: Full Stop
1. Execute FULL_SPEED move
2. Next turn, select FULL_STOP
3. Verify range reduced to 10 units
4. Execute turn
5. Verify momentum reduced by 50%
6. Repeat until ship stops

### Test 4: Turn & Slide
1. Move forward 20 units
2. Confirm and execute
3. Select TURN_SLIDE mode
4. Rotate ship 90 degrees
5. Execute turn
6. Verify ship slides forward while facing sideways

### Test 5: Static Ship
1. Select Cube 2 (moveable = false)
2. Verify movement widget does not appear
3. Verify ship has Ship component
4. Verify ship can still be targeted/attacked

### Test 6: Engine Destruction
1. Damage cruiser's thrusters to 0 HP
2. Verify ship enters drift mode
3. Verify widget is disabled
4. Execute turn
5. Verify ship drifts with 25% previous velocity

---

## Future Enhancements

### Advanced Movement
- **Strafe Thrusters**: Lateral movement without rotation
- **Afterburner**: 3x speed boost, overheats engines
- **Reverse Thrusters**: Move backwards
- **Emergency Turn**: Rapid rotation at cost of position control

### Tactical Features
- **Formation Movement**: Group ships move together
- **Waypoints**: Multiple positions in single turn
- **Patrol Routes**: Automated movement patterns
- **Collision Avoidance**: Auto-adjust paths to avoid friendly ships

### Visual Enhancements
- **Energy Trail**: Glowing path behind ships during movement
- **Thruster Flare**: Intensity based on acceleration
- **Hologram Flicker**: Distortion effect on preview ship
- **Range Rings**: Concentric circles at 5/10/15/20 unit intervals

---

## Notes for Editor

Please review this document and verify:
1. Movement ranges are balanced (20 units base, 40 for boost)
2. Movement mode transition rules match intended gameplay
3. Bezier curve control point calculation matches Unity implementation
4. Timeline duration (10 seconds) feels right for tactical decisions
5. Widget interaction methods are intuitive
6. Any edge cases I missed from the Unity code

Feel free to edit values, add clarifications, or note inconsistencies directly in this document.
