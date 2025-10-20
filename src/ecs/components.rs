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

/// Ship-specific component
#[derive(Debug, Clone)]
pub struct Ship {
    pub name: String,
    pub faction: String,
    pub thrust_force: f64,      // Newtons
    pub rotation_torque: f64,   // Newton-meters
}

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
