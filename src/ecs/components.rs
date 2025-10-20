/// Core ECS Components for space entities
///
/// All positions use 64-bit coordinates (DVec3) for planetary-scale precision
/// Rotations use double-precision quaternions (DQuat)

use glam::{DVec3, DQuat, Vec3};
use serde::{Deserialize, Serialize};

/// 64-bit position component (world space)
/// Provides ~10^15 meter precision near camera origin
#[derive(Debug, Clone, Copy)]
pub struct Position(pub DVec3);

/// 64-bit rotation component (quaternion)
#[derive(Debug, Clone, Copy)]
pub struct Rotation(pub DQuat);

/// 64-bit scale component
#[derive(Debug, Clone, Copy)]
pub struct Scale(pub DVec3);

impl Default for Scale {
    fn default() -> Self {
        Self(DVec3::ONE)
    }
}

/// Velocity component (meters per second)
#[derive(Debug, Clone, Copy, Default)]
pub struct Velocity(pub DVec3);

/// Angular velocity component (radians per second)
#[derive(Debug, Clone, Copy, Default)]
pub struct AngularVelocity(pub DVec3);

/// Mass component (kilograms)
#[derive(Debug, Clone, Copy)]
pub struct Mass(pub f64);

/// Tag component for different entity types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityType {
    Ship,
    Asteroid,
    Planet,
    Star,
    Projectile,
    Nebula,
    Camera,
}

/// Visual representation component
#[derive(Debug, Clone)]
pub struct Visual {
    pub mesh_name: String,
    pub material_name: String,
}

/// Health component for destructible entities
#[derive(Debug, Clone, Copy)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}

impl Health {
    pub fn new(max: f32) -> Self {
        Self { current: max, max }
    }

    pub fn is_alive(&self) -> bool {
        self.current > 0.0
    }

    pub fn damage(&mut self, amount: f32) {
        self.current = (self.current - amount).max(0.0);
    }
}

// Ship component moved below after Star component with tactical movement capabilities

/// Asteroid component
#[derive(Debug, Clone)]
pub struct Asteroid {
    pub radius: f64,  // meters
}

/// Planet component
#[derive(Debug, Clone)]
pub struct Planet {
    pub name: String,
    pub radius: f64,      // meters
    pub mass: f64,        // kilograms
}

/// Nebula component (visual effect at massive scale)
#[derive(Debug, Clone)]
pub struct Nebula {
    pub scale: f64,           // size in meters (can be 1000x larger now!)
    pub density: f32,
    pub color: Vec3,
}

/// Star component (procedural sun with limb darkening)
#[derive(Debug, Clone)]
pub struct Star {
    pub name: String,
    pub radius: f64,          // meters (e.g., Sun = 695,700,000 m)
    pub mass: f64,            // kilograms
    pub temperature: f32,     // Kelvin (affects color)
    pub color: Vec3,          // RGB color
    pub gamma: f32,           // Gamma correction (default 2.2)
    pub exposure: f32,        // Exposure multiplier (default 40.2)
}

impl Default for Star {
    fn default() -> Self {
        // Sun-like defaults
        Self {
            name: "Star".to_string(),
            radius: 695_700_000.0,         // Sun radius in meters
            mass: 1.989e30,                // Sun mass in kg
            temperature: 5778.0,           // Sun surface temperature
            color: Vec3::new(1.0, 0.14, 0.01), // Artistic sun color
            gamma: 2.2,
            exposure: 40.2,
        }
    }
}

/// Ship component for turn-based tactical movement
#[derive(Debug, Clone)]
pub struct Ship {
    pub name: String,

    /// Movement capabilities
    pub max_movement_range: f32,        // Maximum XZ distance per turn (default: 20 units)
    pub max_rotation_angle: f32,        // Maximum rotation per turn in radians (default: π/2 = 90°)
    pub max_elevation_change: f32,      // Maximum Y-axis change per turn (default: 10 units)

    /// Current turn state
    pub confirmed_move: bool,           // Has player confirmed this turn's movement?
    pub movement_locked: bool,          // Is movement disabled (engines damaged)?

    /// Planned movement (what the widget shows)
    pub planned_position: DVec3,        // Target position for this turn
    pub planned_rotation: DQuat,        // Target rotation for this turn

    /// Turn start state (for constraint validation)
    pub turn_start_position: DVec3,     // Position at start of turn
    pub turn_start_rotation: DQuat,     // Rotation at start of turn

    /// Movement curve (Bezier curve parameters)
    pub last_velocity: DVec3,           // Velocity from previous turn (for momentum)
    pub control_point: DVec3,           // Bezier control point for current turn

    /// Mesh bounds (min, max) for widget positioning
    pub bounds_min: Vec3,               // AABB min in local space
    pub bounds_max: Vec3,               // AABB max in local space
}

impl Ship {
    pub fn new(name: String) -> Self {
        Self {
            name,
            max_movement_range: 20.0,
            max_rotation_angle: std::f32::consts::FRAC_PI_2, // 90 degrees
            max_elevation_change: 10.0,
            confirmed_move: false,
            movement_locked: false,
            planned_position: DVec3::ZERO,
            planned_rotation: DQuat::IDENTITY,
            turn_start_position: DVec3::ZERO,
            turn_start_rotation: DQuat::IDENTITY,
            last_velocity: DVec3::ZERO,
            control_point: DVec3::ZERO,
            bounds_min: Vec3::new(-1.0, -1.0, -1.0), // Default unit cube bounds
            bounds_max: Vec3::new(1.0, 1.0, 1.0),
        }
    }

    /// Initialize turn - save starting position/rotation
    pub fn start_turn(&mut self, current_position: DVec3, current_rotation: DQuat) {
        self.turn_start_position = current_position;
        self.turn_start_rotation = current_rotation;
        self.planned_position = current_position;
        self.planned_rotation = current_rotation;
        self.confirmed_move = false;
    }

    /// Check if planned position is within movement range
    pub fn is_position_valid(&self, position: DVec3) -> bool {
        let offset = position - self.turn_start_position;

        // Check XZ distance (horizontal radius)
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

    /// Clamp position to valid movement range
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

    /// Calculate Bezier control point for smooth movement
    pub fn calculate_control_point(&self, end_position: DVec3) -> DVec3 {
        if self.last_velocity.length() < 0.001 {
            // First move - no previous velocity
            // Control point is 1/2.5 along the path
            let offset = end_position - self.turn_start_position;
            self.turn_start_position + offset / 2.5
        } else {
            // Use previous velocity for momentum
            self.turn_start_position + self.last_velocity / 2.5
        }
    }
}

/// Movement curve for Bezier-based ship movement
#[derive(Debug, Clone, Copy)]
pub struct MovementCurve {
    pub start_position: DVec3,
    pub end_position: DVec3,
    pub control_point: DVec3,
}

impl MovementCurve {
    pub fn new(start: DVec3, end: DVec3, control: DVec3) -> Self {
        Self {
            start_position: start,
            end_position: end,
            control_point: control,
        }
    }

    /// Evaluate Bezier curve at time t [0.0 to 1.0]
    pub fn evaluate(&self, t: f64) -> DVec3 {
        let t2 = t * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;

        // Quadratic Bezier formula: P(t) = (1-t)² * P0 + 2(1-t)t * P1 + t² * P2
        self.start_position * mt2
            + self.control_point * (2.0 * mt * t)
            + self.end_position * t2
    }

    /// Get velocity at time t (derivative of position)
    pub fn velocity_at(&self, t: f64) -> DVec3 {
        let mt = 1.0 - t;

        // Derivative of quadratic Bezier
        2.0 * mt * (self.control_point - self.start_position)
            + 2.0 * t * (self.end_position - self.control_point)
    }

    /// Get ending velocity (for next turn's momentum)
    pub fn ending_velocity(&self) -> DVec3 {
        self.end_position - self.control_point
    }

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

/// Parent-child relationship component
/// Stores the parent entity ID for hierarchical transforms
#[derive(Debug, Clone, Copy)]
pub struct Parent(pub hecs::Entity);

/// Children component - tracks all child entities
#[derive(Debug, Clone)]
pub struct Children(pub Vec<hecs::Entity>);

/// Turn-based state component
/// Tracks orders and state for turn resolution
#[derive(Debug, Clone)]
pub struct TurnState {
    pub pending_orders: Vec<Order>,
    pub completed_orders: Vec<Order>,
    pub action_points: u32,
    pub max_action_points: u32,
}

/// Orders for turn-based gameplay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Order {
    Move { target_position: DVec3 },
    Attack { target_entity: u64 },  // Entity ID
    Defend,
    Wait,
}

/// Event component for tracking gameplay events
/// Used for replay and undo functionality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameEvent {
    pub turn: u32,
    pub timestamp: f64,
    pub event_type: EventType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    EntitySpawned { entity_id: u64, entity_type: EntityType },
    EntityDestroyed { entity_id: u64 },
    EntityMoved { entity_id: u64, from: DVec3, to: DVec3 },
    Combat { attacker: u64, defender: u64, damage: f32 },
    TurnEnded { turn: u32 },
}
