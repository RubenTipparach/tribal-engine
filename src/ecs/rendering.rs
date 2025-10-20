/// Rendering system for ECS entities
///
/// Handles conversion from 64-bit world positions to 32-bit camera-relative positions
/// for GPU rendering

use glam::{DVec3, DQuat, Vec3, Quat, Mat4};
use super::components::*;

/// Render data extracted from ECS for a frame
/// This is what gets sent to the renderer
pub struct RenderBatch {
    pub entities: Vec<RenderEntity>,
}

/// A single entity to render
pub struct RenderEntity {
    /// Model matrix (camera-relative, 32-bit)
    pub model_matrix: Mat4,

    /// Mesh name to render
    pub mesh_name: String,

    /// Material name
    pub material_name: String,

    /// Entity type (for sorting/batching)
    pub entity_type: EntityType,

    /// Distance from camera (for sorting)
    pub distance_from_camera: f32,
}

impl RenderBatch {
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
        }
    }

    /// Add an entity to the render batch
    pub fn add_entity(
        &mut self,
        position: DVec3,
        rotation: DQuat,
        scale: DVec3,
        visual: &Visual,
        entity_type: EntityType,
        camera_origin: DVec3,
    ) {
        // Convert to camera-relative 32-bit position
        let relative_pos = position - camera_origin;
        let pos_f32 = Vec3::new(
            relative_pos.x as f32,
            relative_pos.y as f32,
            relative_pos.z as f32,
        );

        // Convert rotation and scale to 32-bit
        let rot_f32 = Quat::from_xyzw(
            rotation.x as f32,
            rotation.y as f32,
            rotation.z as f32,
            rotation.w as f32,
        );
        let scale_f32 = Vec3::new(
            scale.x as f32,
            scale.y as f32,
            scale.z as f32,
        );

        // Build model matrix
        let model_matrix = Mat4::from_scale_rotation_translation(
            scale_f32,
            rot_f32,
            pos_f32,
        );

        let distance_from_camera = pos_f32.length();

        self.entities.push(RenderEntity {
            model_matrix,
            mesh_name: visual.mesh_name.clone(),
            material_name: visual.material_name.clone(),
            entity_type,
            distance_from_camera,
        });
    }

    /// Sort entities for optimal rendering
    /// Opaque objects front-to-back, transparent back-to-front
    pub fn sort(&mut self) {
        // For now, just sort by distance (front to back)
        self.entities.sort_by(|a, b| {
            a.distance_from_camera
                .partial_cmp(&b.distance_from_camera)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    /// Get number of entities to render
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    /// Check if batch is empty
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }
}

impl Default for RenderBatch {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract renderable entities from ECS world
pub fn extract_render_batch(
    world: &hecs::World,
    camera_origin: DVec3,
    max_distance: f64,
) -> RenderBatch {
    let mut batch = RenderBatch::new();
    let max_distance_sq = max_distance * max_distance;

    // Query all entities with required components
    for (_entity, (pos, rot, scale, visual, entity_type)) in world
        .query::<(&Position, &Rotation, &Scale, &Visual, &EntityType)>()
        .iter()
    {
        // Distance culling (64-bit)
        let dist_sq = pos.0.distance_squared(camera_origin);
        if dist_sq > max_distance_sq {
            continue;
        }

        batch.add_entity(
            pos.0,
            rot.0,
            scale.0,
            visual,
            *entity_type,
            camera_origin,
        );
    }

    batch.sort();
    batch
}
