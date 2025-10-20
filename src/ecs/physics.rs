/// Physics integration with Rapier
///
/// Provides:
/// - Deterministic physics simulation (fixed timestep)
/// - Collision detection for ships, asteroids, projectiles
/// - Integration with 64-bit coordinate system

use rapier3d::prelude::*;
use glam::{DVec3, DQuat, Vec3};
use nalgebra as na;

/// Physics world wrapper
/// Handles collision detection and deterministic simulation
pub struct PhysicsWorld {
    /// Rapier rigid body set
    pub rigid_body_set: RigidBodySet,

    /// Rapier collider set
    pub collider_set: ColliderSet,

    /// Gravity configuration (optional - space combat may not need gravity)
    pub gravity: Vector<Real>,

    /// Integration parameters for deterministic simulation
    pub integration_params: IntegrationParameters,

    /// Physics pipeline
    pub physics_pipeline: PhysicsPipeline,

    /// Island manager
    pub island_manager: IslandManager,

    /// Broad phase
    pub broad_phase: DefaultBroadPhase,

    /// Narrow phase
    pub narrow_phase: NarrowPhase,

    /// Impulse joint set
    pub impulse_joint_set: ImpulseJointSet,

    /// Multibody joint set
    pub multibody_joint_set: MultibodyJointSet,

    /// CCD solver
    pub ccd_solver: CCDSolver,

    /// Query pipeline for raycasts
    pub query_pipeline: QueryPipeline,
}

impl PhysicsWorld {
    /// Create a new physics world with deterministic settings
    pub fn new() -> Self {
        let mut integration_params = IntegrationParameters::default();

        // Fixed timestep for determinism
        integration_params.dt = 1.0 / 60.0;  // 60Hz physics

        Self {
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            gravity: vector![0.0, 0.0, 0.0],  // No gravity in space
            integration_params,
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            query_pipeline: QueryPipeline::new(),
        }
    }

    /// Step the physics simulation (deterministic)
    pub fn step(&mut self) {
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_params,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            None,  // No query pipeline modifications
            &(),   // No hooks
            &(),   // No events
        );

        self.query_pipeline.update(&self.collider_set);
    }

    /// Add a dynamic ship collider
    pub fn add_ship_collider(
        &mut self,
        position: DVec3,
        rotation: DQuat,
        half_extents: Vec3,
    ) -> RigidBodyHandle {
        // Convert 64-bit position to Rapier's coordinate system
        // Note: For large-scale battles, we'll use sectors/zones
        let pos = dvec3_to_isometry(position, rotation);

        let rigid_body = RigidBodyBuilder::dynamic()
            .position(pos)
            .build();
        let rb_handle = self.rigid_body_set.insert(rigid_body);

        // Box collider for ship
        let collider = ColliderBuilder::cuboid(
            half_extents.x,
            half_extents.y,
            half_extents.z,
        ).build();
        self.collider_set.insert_with_parent(collider, rb_handle, &mut self.rigid_body_set);

        rb_handle
    }

    /// Add a static asteroid collider
    pub fn add_asteroid_collider(
        &mut self,
        position: DVec3,
        radius: f64,
    ) -> RigidBodyHandle {
        let pos = dvec3_to_isometry(position, DQuat::IDENTITY);

        let rigid_body = RigidBodyBuilder::fixed()
            .position(pos)
            .build();
        let rb_handle = self.rigid_body_set.insert(rigid_body);

        // Sphere collider for asteroid
        let collider = ColliderBuilder::ball(radius as f32).build();
        self.collider_set.insert_with_parent(collider, rb_handle, &mut self.rigid_body_set);

        rb_handle
    }

    /// Raycast for targeting/line-of-sight checks
    pub fn raycast(
        &self,
        origin: DVec3,
        direction: DVec3,
        max_distance: f64,
    ) -> Option<(RigidBodyHandle, f32)> {
        let ray = Ray::new(
            dvec3_to_point(origin),
            dvec3_to_vector(direction.normalize()),
        );

        let hit = self.query_pipeline.cast_ray(
            &self.rigid_body_set,
            &self.collider_set,
            &ray,
            max_distance as f32,
            true,
            QueryFilter::default(),
        )?;

        let collider_handle = hit.0;
        let collider = self.collider_set.get(collider_handle)?;
        let rb_handle = collider.parent()?;

        Some((rb_handle, hit.1))
    }
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert DVec3 to Rapier Isometry (position + rotation)
fn dvec3_to_isometry(pos: DVec3, rot: DQuat) -> Isometry<Real> {
    Isometry::from_parts(
        dvec3_to_translation(pos),
        dquat_to_unit_quat(rot),
    )
}

/// Convert DVec3 to Rapier Translation
fn dvec3_to_translation(v: DVec3) -> Translation<Real> {
    Translation::new(v.x as f32, v.y as f32, v.z as f32)
}

/// Convert DVec3 to Rapier Point
fn dvec3_to_point(v: DVec3) -> Point<Real> {
    Point::new(v.x as f32, v.y as f32, v.z as f32)
}

/// Convert DVec3 to Rapier Vector
fn dvec3_to_vector(v: DVec3) -> Vector<Real> {
    Vector::new(v.x as f32, v.y as f32, v.z as f32)
}

/// Convert DQuat to Rapier UnitQuaternion
fn dquat_to_unit_quat(q: DQuat) -> na::UnitQuaternion<Real> {
    na::UnitQuaternion::from_quaternion(na::Quaternion::new(
        q.w as f32,
        q.x as f32,
        q.y as f32,
        q.z as f32,
    ))
}
