/// ECS World and Component System
///
/// This module provides:
/// - 64-bit coordinate system (DVec3) for world positions
/// - Camera-relative rendering (converts to 32-bit at render time)
/// - Component definitions for space entities
/// - Integration with Rapier physics

pub mod components;
pub mod physics;
pub mod spatial;
pub mod rendering;
pub mod hierarchy;
pub mod init;

use glam::{DVec3, Vec3};
use hecs::World;

/// The main ECS world containing all entities
pub struct EcsWorld {
    /// hecs World - stores all entities and components
    pub world: World,

    /// Camera origin in world space (64-bit)
    /// All rendering is done relative to this point
    pub camera_origin: DVec3,
}

impl EcsWorld {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            camera_origin: DVec3::ZERO,
        }
    }

    /// Update camera origin for rendering
    /// This should be the camera's world position
    pub fn set_camera_origin(&mut self, origin: DVec3) {
        self.camera_origin = origin;
    }

    /// Convert world position (64-bit) to camera-relative position (32-bit)
    /// This is safe because we're only rendering things close to the camera
    pub fn world_to_camera_relative(&self, world_pos: DVec3) -> Vec3 {
        let relative = world_pos - self.camera_origin;
        Vec3::new(
            relative.x as f32,
            relative.y as f32,
            relative.z as f32,
        )
    }

    /// Get the number of entities
    pub fn entity_count(&self) -> u32 {
        self.world.len() as u32
    }
}

impl Default for EcsWorld {
    fn default() -> Self {
        Self::new()
    }
}
