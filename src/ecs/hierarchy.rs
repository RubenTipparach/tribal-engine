/// Hierarchical transform system for parent-child relationships
///
/// Ensures child transforms are updated relative to their parents
/// Critical for star-nebula parenting where star follows nebula

use glam::{DVec3, DQuat, DMat4};
use hecs::{Entity, World};
use crate::ecs::components::{Position, Rotation, Scale, Parent, Children};

/// Compute world-space transform from local transform and parent
pub struct TransformHierarchy;

impl TransformHierarchy {
    /// Update all child transforms based on parent transforms
    /// Call this once per frame after updating root entity positions
    pub fn update_hierarchy(world: &mut World) {
        // Get all root entities (entities without Parent component)
        let mut roots: Vec<Entity> = Vec::new();

        for (entity, ()) in world.query::<()>().iter() {
            // Check if entity has Position but not Parent
            if world.get::<&Position>(entity).is_ok() && world.get::<&Parent>(entity).is_err() {
                roots.push(entity);
            }
        }

        // Recursively update children of each root
        for root in roots {
            Self::update_children_recursive(world, root, None);
        }
    }

    /// Recursively update children's world transform
    fn update_children_recursive(world: &mut World, entity: Entity, parent_transform: Option<DMat4>) {
        // Get this entity's local transform
        let local_transform = Self::get_local_transform(world, entity);

        // Compute world transform
        let world_transform = if let Some(parent_mat) = parent_transform {
            parent_mat * local_transform
        } else {
            local_transform
        };

        // Extract position, rotation, scale from world transform if this has a parent
        if parent_transform.is_some() {
            Self::apply_world_transform(world, entity, world_transform);
        }

        // Collect children first to avoid borrow checker issues
        let children: Vec<Entity> = world.get::<&Children>(entity)
            .map(|c| c.0.clone())
            .unwrap_or_default();

        // Update all children
        for child in children {
            Self::update_children_recursive(world, child, Some(world_transform));
        }
    }

    /// Get local transform matrix from Position, Rotation, Scale
    fn get_local_transform(world: &World, entity: Entity) -> DMat4 {
        let pos = world.get::<&Position>(entity)
            .map(|p| p.0)
            .unwrap_or(DVec3::ZERO);

        let rot = world.get::<&Rotation>(entity)
            .map(|r| r.0)
            .unwrap_or(DQuat::IDENTITY);

        let scale = world.get::<&Scale>(entity)
            .map(|s| s.0)
            .unwrap_or(DVec3::ONE);

        DMat4::from_scale_rotation_translation(scale, rot, pos)
    }

    /// Apply world transform to entity's Position, Rotation, Scale
    fn apply_world_transform(world: &mut World, entity: Entity, transform: DMat4) {
        // Extract translation
        let translation = DVec3::new(
            transform.w_axis.x,
            transform.w_axis.y,
            transform.w_axis.z,
        );

        // Extract scale (length of each axis)
        let scale_x = DVec3::new(transform.x_axis.x, transform.x_axis.y, transform.x_axis.z).length();
        let scale_y = DVec3::new(transform.y_axis.x, transform.y_axis.y, transform.y_axis.z).length();
        let scale_z = DVec3::new(transform.z_axis.x, transform.z_axis.y, transform.z_axis.z).length();
        let scale = DVec3::new(scale_x, scale_y, scale_z);

        // Extract rotation (normalize axes and create rotation matrix)
        let rotation_mat = DMat4::from_cols(
            transform.x_axis / scale_x as f64,
            transform.y_axis / scale_y as f64,
            transform.z_axis / scale_z as f64,
            glam::DVec4::W,
        );
        let rotation = DQuat::from_mat4(&rotation_mat);

        // Update components
        if let Ok(mut pos) = world.get::<&mut Position>(entity) {
            pos.0 = translation;
        }

        if let Ok(mut rot) = world.get::<&mut Rotation>(entity) {
            rot.0 = rotation;
        }

        if let Ok(mut scl) = world.get::<&mut Scale>(entity) {
            scl.0 = scale;
        }
    }

    /// Add a child to a parent entity
    /// Automatically sets up Parent and Children components
    pub fn add_child(world: &mut World, parent: Entity, child: Entity) {
        // Check if child already has parent component
        let has_parent = world.get::<&Parent>(child).is_ok();

        if has_parent {
            if let Ok(mut parent_comp) = world.get::<&mut Parent>(child) {
                parent_comp.0 = parent;
            }
        } else {
            let _ = world.insert_one(child, Parent(parent));
        }

        // Check if parent has children component
        let has_children = world.get::<&Children>(parent).is_ok();

        if has_children {
            if let Ok(mut children) = world.get::<&mut Children>(parent) {
                if !children.0.contains(&child) {
                    children.0.push(child);
                }
            }
        } else {
            let _ = world.insert_one(parent, Children(vec![child]));
        }
    }

    /// Remove a child from its parent
    pub fn remove_child(world: &mut World, parent: Entity, child: Entity) {
        // Remove parent component from child
        let _ = world.remove_one::<Parent>(child);

        // Remove child from parent's children list
        if let Ok(mut children) = world.get::<&mut Children>(parent) {
            children.0.retain(|&e| e != child);
        }
    }

    /// Get all descendants of an entity (children, grandchildren, etc.)
    pub fn get_descendants(world: &World, entity: Entity) -> Vec<Entity> {
        let mut descendants = Vec::new();

        if let Ok(children) = world.get::<&Children>(entity) {
            for &child in &children.0 {
                descendants.push(child);
                descendants.extend(Self::get_descendants(world, child));
            }
        }

        descendants
    }
}

