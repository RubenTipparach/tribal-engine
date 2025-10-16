use glam::{Mat4, Quat, Vec3};
use crate::nebula::NebulaConfig;
use crate::camera::Camera;

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

impl Default for SkyboxConfig {
    fn default() -> Self {
        Self {
            star_density: 2.0,
            star_brightness: 3.0,
            nebula_primary_color: Vec3::new(0.1, 0.2, 0.4),
            nebula_secondary_color: Vec3::new(0.6, 0.3, 0.8),
            nebula_intensity: 0.3,
            background_brightness: 0.02,
        }
    }
}

/// Game state and logic
pub struct Game {
    /// Time accumulator for animations
    time: f32,
    /// Camera
    pub camera: Camera,
    /// Spaceship position
    pub ship_position: Vec3,
    /// Spaceship rotation (quaternion)
    pub ship_rotation: Quat,
    /// Spaceship velocity
    ship_velocity: Vec3,
    /// Spaceship angular velocity
    ship_angular_velocity: Vec3,
    /// Ship scale
    pub ship_scale: Vec3,
    /// Movement speed
    move_speed: f32,
    /// Rotation speed
    rotation_speed: f32,
    /// Skybox configuration
    pub skybox_config: SkyboxConfig,
    /// Nebula configuration
    pub nebula_config: NebulaConfig,
    /// Show cube mesh
    pub show_cube: bool,
}

impl Game {
    pub fn new() -> Self {
        Self {
            time: 0.0,
            camera: Camera::default(),
            ship_position: Vec3::ZERO,
            ship_rotation: Quat::IDENTITY,
            ship_velocity: Vec3::ZERO,
            ship_angular_velocity: Vec3::ZERO,
            ship_scale: Vec3::ONE,
            move_speed: 5.0,
            rotation_speed: 2.0,
            skybox_config: SkyboxConfig::default(),
            nebula_config: NebulaConfig::default(),
            show_cube: true,
        }
    }
    
    /// Update game logic
    pub fn update(&mut self, delta_time: f32) {
        self.time += delta_time;
        
        // Apply angular velocity
        let delta_rotation = Quat::from_euler(
            glam::EulerRot::XYZ,
            self.ship_angular_velocity.x * delta_time,
            self.ship_angular_velocity.y * delta_time,
            self.ship_angular_velocity.z * delta_time,
        );
        self.ship_rotation = (self.ship_rotation * delta_rotation).normalize();
        
        // Apply velocity with damping
        self.ship_position += self.ship_velocity * delta_time;
        self.ship_velocity *= 0.98; // Air resistance
        self.ship_angular_velocity *= 0.95; // Angular damping
        
        // You can add more game logic here:
        // - Collision detection
        // - Physics simulation
        // - etc.
    }
    
    /// Get the current model matrix for the spaceship
    pub fn get_cube_model_matrix(&self) -> Mat4 {
        let translation = Mat4::from_translation(self.ship_position);
        let rotation = Mat4::from_quat(self.ship_rotation);
        let scale = Mat4::from_scale(self.ship_scale);
        
        translation * rotation * scale
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
        let forward = self.ship_rotation * Vec3::NEG_Z;
        self.ship_velocity += forward * amount;
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
