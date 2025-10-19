use glam::{Mat4, Quat, Vec3};
use crate::nebula::NebulaConfig;
use crate::core::Camera;
use crate::scene::{SceneGraph, ObjectType};
use crate::gizmo::{GizmoState, ObjectPicker};

/// Skybox configuration
#[derive(Clone)]
pub struct SkyboxConfig {
    /// Star density (0.0 to 1.0)
    pub star_density: f32,
    /// Star brightness multiplier
    pub star_brightness: f32,
    /// Nebula color primary
    pub nebula_primary_color: Vec3,
    /// Nebula color secondary
    pub nebula_secondary_color: Vec3,
    /// Nebula intensity
    pub nebula_intensity: f32,
    /// Background darkness (0.0 = black, 1.0 = lighter)
    pub background_brightness: f32,
}

impl From<crate::config::SkyboxConfigData> for SkyboxConfig {
    fn from(data: crate::config::SkyboxConfigData) -> Self {
        Self {
            star_density: data.star_density,
            star_brightness: data.star_brightness,
            nebula_primary_color: data.nebula_primary_color,
            nebula_secondary_color: data.nebula_secondary_color,
            nebula_intensity: data.nebula_intensity,
            background_brightness: data.background_brightness,
        }
    }
}

impl From<&SkyboxConfig> for crate::config::SkyboxConfigData {
    fn from(config: &SkyboxConfig) -> Self {
        Self {
            star_density: config.star_density,
            star_brightness: config.star_brightness,
            nebula_primary_color: config.nebula_primary_color,
            nebula_secondary_color: config.nebula_secondary_color,
            nebula_intensity: config.nebula_intensity,
            background_brightness: config.background_brightness,
        }
    }
}

impl Default for SkyboxConfig {
    fn default() -> Self {
        Self {
            star_density: 2.0,
            star_brightness: 3.0,
            nebula_primary_color: Vec3::new(0.1, 0.2, 0.4),
            nebula_secondary_color: Vec3::new(0.6, 0.3, 0.8),
            nebula_intensity: 1.0,
            background_brightness: 0.00,
        }
    }
}

/// Game state and logic
pub struct Game {
    /// Time accumulator for animations
    time: f32,
    /// Camera
    pub camera: Camera,
    /// Scene graph with all objects
    pub scene: SceneGraph,
    /// Gizmo state for 3D manipulation
    pub gizmo_state: GizmoState,
    /// Object picker for mouse selection
    pub object_picker: ObjectPicker,
    /// Spaceship velocity (for physics simulation)
    ship_velocity: Vec3,
    /// Spaceship angular velocity (for physics simulation)
    ship_angular_velocity: Vec3,
    /// Movement speed
    move_speed: f32,
    /// Rotation speed
    rotation_speed: f32,
    /// Skybox configuration
    pub skybox_config: SkyboxConfig,
    /// Nebula configuration
    pub nebula_config: NebulaConfig,
}

impl Game {
    pub fn new() -> Self {
        let mut scene = SceneGraph::new();

        // Add default scene objects
        scene.add_object("Cube".to_string(), ObjectType::Cube);
        scene.add_object("Nebula".to_string(), ObjectType::Nebula);
        scene.add_object("Skybox".to_string(), ObjectType::Skybox);

        Self {
            time: 0.0,
            camera: Camera::default(),
            scene,
            gizmo_state: GizmoState::new(),
            object_picker: ObjectPicker::new(),
            ship_velocity: Vec3::ZERO,
            ship_angular_velocity: Vec3::ZERO,
            move_speed: 5.0,
            rotation_speed: 2.0,
            skybox_config: SkyboxConfig::default(),
            nebula_config: NebulaConfig::default(),
        }
    }

    /// Handle mouse hover for object picking
    pub fn handle_mouse_hover(&mut self, mouse_x: f32, mouse_y: f32, viewport_width: f32, viewport_height: f32) {
        self.object_picker.pick_object(
            mouse_x,
            mouse_y,
            viewport_width,
            viewport_height,
            &self.scene,
            &self.camera,
        );
    }

    /// Handle mouse click for object selection
    pub fn handle_mouse_click(&mut self, mouse_x: f32, mouse_y: f32, viewport_width: f32, viewport_height: f32) {
        if let Some(object_id) = self.object_picker.pick_object(
            mouse_x,
            mouse_y,
            viewport_width,
            viewport_height,
            &self.scene,
            &self.camera,
        ) {
            self.scene.select_object(object_id);
        }
    }
    
    /// Update game logic
    pub fn update(&mut self, delta_time: f32) {
        self.time += delta_time;

        // Update cube object if it exists
        if let Some(cube_id) = self.scene.find_by_type(ObjectType::Cube) {
            if let Some(cube) = self.scene.get_object_mut(cube_id) {
                // Apply angular velocity to cube
                let delta_rotation = Quat::from_euler(
                    glam::EulerRot::XYZ,
                    self.ship_angular_velocity.x * delta_time,
                    self.ship_angular_velocity.y * delta_time,
                    self.ship_angular_velocity.z * delta_time,
                );
                cube.transform.rotation = (cube.transform.rotation * delta_rotation).normalize();

                // Apply velocity with damping
                cube.transform.position += self.ship_velocity * delta_time;
            }
        }

        self.ship_velocity *= 0.98; // Air resistance
        self.ship_angular_velocity *= 0.95; // Angular damping
    }

    /// Get the current model matrix for the cube
    pub fn get_cube_model_matrix(&self) -> Mat4 {
        if let Some(cube_id) = self.scene.find_by_type(ObjectType::Cube) {
            if let Some(cube) = self.scene.get_object(cube_id) {
                return cube.transform.model_matrix();
            }
        }
        Mat4::IDENTITY
    }

    /// Check if cube is visible
    pub fn is_cube_visible(&self) -> bool {
        self.scene
            .find_by_type(ObjectType::Cube)
            .and_then(|id| self.scene.get_object(id))
            .map(|obj| obj.visible)
            .unwrap_or(false)
    }
    
    /// Get camera view matrix
    pub fn get_view_matrix(&self) -> Mat4 {
        self.camera.view_matrix()
    }
    
    /// Get the current time for shader effects
    pub fn get_time(&self) -> f32 {
        self.time
    }
    
    /// Get camera position for shaders
    pub fn get_camera_position(&self) -> Vec3 {
        self.camera.position()
    }
    
    // Control methods
    
    pub fn set_move_speed(&mut self, speed: f32) {
        self.move_speed = speed;
    }
    
    pub fn set_rotation_speed(&mut self, speed: f32) {
        self.rotation_speed = speed;
    }
    
    pub fn add_thrust(&mut self, amount: f32) {
        if let Some(cube_id) = self.scene.find_by_type(ObjectType::Cube) {
            if let Some(cube) = self.scene.get_object(cube_id) {
                let forward = cube.transform.rotation * Vec3::NEG_Z;
                self.ship_velocity += forward * amount;
            }
        }
    }
    
    pub fn add_rotation(&mut self, pitch: f32, yaw: f32, roll: f32) {
        self.ship_angular_velocity.x += pitch;
        self.ship_angular_velocity.y += yaw;
        self.ship_angular_velocity.z += roll;
    }
    
    pub fn rotate_camera(&mut self, pitch_delta: f32, yaw_delta: f32) {
        self.camera.rotate(pitch_delta, yaw_delta);
    }
    
    pub fn move_camera_forward(&mut self, amount: f32) {
        self.camera.move_forward(amount);
    }
    
    pub fn move_camera_right(&mut self, amount: f32) {
        self.camera.move_right(amount);
    }
    
    pub fn roll_camera(&mut self, amount: f32) {
        self.camera.roll(amount);
    }
}
