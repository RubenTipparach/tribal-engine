/// ECS World initialization helpers
///
/// Provides functions to create common entities (nebula, star, ships, etc.)

use glam::{DVec3, DQuat, Vec3};
use hecs::{Entity, World};
use crate::ecs::components::*;
use crate::ecs::hierarchy::TransformHierarchy;

/// Initialize a nebula with 1000x scale
///
/// Default nebula scale is typically 20.0 units in the old system
/// With 1000x scaling and 64-bit coordinates, this becomes 20,000 meters
pub fn create_nebula_entity(world: &mut World, position: DVec3, scale_multiplier: f64) -> Entity {
    let scale = 20_000.0 * scale_multiplier; // Base scale * multiplier (e.g., 1000x)

    world.spawn((
        Position(position),
        Rotation(DQuat::IDENTITY),
        Scale(DVec3::splat(scale)),
        Nebula {
            scale,
            density: 2.0,
            color: Vec3::new(0.1, 0.2, 0.4),
        },
        EntityType::Nebula,
        Visual {
            mesh_name: "nebula".to_string(),
            material_name: "nebula_material".to_string(),
        },
    ))
}

/// Create a star entity at the center of a nebula
///
/// The star will be parented to the nebula so it follows the nebula's position
/// local_offset allows positioning the star relative to the nebula's local space
pub fn create_star_entity(
    world: &mut World,
    parent_nebula: Entity,
    radius: f64,
    local_offset: DVec3,
) -> Entity {
    let star = world.spawn((
        Position(local_offset), // Local position relative to parent
        Rotation(DQuat::IDENTITY),
        Scale(DVec3::splat(radius)),
        Star::default(),
        EntityType::Star,
        Visual {
            mesh_name: "sphere".to_string(),
            material_name: "star_material".to_string(),
        },
    ));

    // Parent the star to the nebula
    TransformHierarchy::add_child(world, parent_nebula, star);

    star
}

/// Create a ship entity
pub fn create_ship_entity(
    world: &mut World,
    name: String,
    position: DVec3,
    faction: String,
) -> Entity {
    world.spawn((
        Position(position),
        Rotation(DQuat::IDENTITY),
        Scale(DVec3::ONE),
        Velocity(DVec3::ZERO),
        AngularVelocity(DVec3::ZERO),
        Mass(50_000.0), // 50 tons
        Health::new(100.0),
        Ship {
            name: name.clone(),
            faction,
            thrust_force: 100_000.0,    // 100kN
            rotation_torque: 50_000.0,  // 50kNm
        },
        EntityType::Ship,
        Visual {
            mesh_name: "ship".to_string(),
            material_name: "ship_material".to_string(),
        },
        TurnState {
            pending_orders: Vec::new(),
            completed_orders: Vec::new(),
            action_points: 10,
            max_action_points: 10,
        },
    ))
}

/// Create an asteroid entity
pub fn create_asteroid_entity(
    world: &mut World,
    position: DVec3,
    radius: f64,
) -> Entity {
    let mass = 4.0 / 3.0 * std::f64::consts::PI * radius.powi(3) * 2_500.0; // Density ~2500 kg/m³

    world.spawn((
        Position(position),
        Rotation(DQuat::IDENTITY),
        Scale(DVec3::splat(radius)),
        Velocity(DVec3::ZERO),
        AngularVelocity(DVec3::ZERO),
        Mass(mass),
        Asteroid { radius },
        EntityType::Asteroid,
        Visual {
            mesh_name: "asteroid".to_string(),
            material_name: "asteroid_material".to_string(),
        },
    ))
}

/// Create a planet entity
pub fn create_planet_entity(
    world: &mut World,
    name: String,
    position: DVec3,
    radius: f64,
    mass: f64,
) -> Entity {
    world.spawn((
        Position(position),
        Rotation(DQuat::IDENTITY),
        Scale(DVec3::splat(radius)),
        Planet {
            name,
            radius,
            mass,
        },
        EntityType::Planet,
        Visual {
            mesh_name: "planet".to_string(),
            material_name: "planet_material".to_string(),
        },
    ))
}

/// Initialize a complete scene with nebula, star, and some asteroids
pub fn init_default_scene(world: &mut World) -> (Entity, Entity) {
    // Position nebula so system origin (0,0,0) is at the center with star
    // Nebula scale is 20,000 units, so radius ~10,000 units
    // Move nebula back so camera at +Z edge looks into the nebula toward origin
    let nebula_offset = DVec3::new(0.0, 0.0, -10_000.0);
    let nebula = create_nebula_entity(world, nebula_offset, 1.0);

    // Create star at system origin (0,0,0) - Sun-sized
    // Star is parented to nebula, so offset it by opposite of nebula position
    // to place it at world origin
    let star_local_offset = -nebula_offset; // (0, 0, +10,000)
    let star = create_star_entity(world, nebula, 695_700_000.0, star_local_offset);

    // Add some asteroids around the star/origin
    for i in 0..10 {
        let angle = (i as f64) * std::f64::consts::TAU / 10.0;
        let distance = 5_000.0; // 5000 units from center
        let position = DVec3::new(
            distance * angle.cos(),
            (i as f64 - 5.0) * 100.0, // Vary height
            distance * angle.sin(),
        );

        create_asteroid_entity(world, position, 100.0 + (i as f64) * 10.0);
    }

    (nebula, star)
}
