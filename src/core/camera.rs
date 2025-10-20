use glam::{Mat4, Quat, Vec3};

/// Free-flying camera with 6 degrees of freedom
pub struct Camera {
    /// Camera position in world space
    position: Vec3,
    /// Camera rotation (pitch, yaw, roll in radians)
    pitch: f32,
    yaw: f32,
    roll: f32,
    /// Field of view in radians
    fov: f32,
    /// Near clipping plane distance
    near_plane: f32,
    /// Far clipping plane distance
    far_plane: f32,
}

impl Camera {
    /// Create a new camera at the given position with default projection settings
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            pitch: 0.0,
            yaw: 0.0,
            roll: 0.0,
            fov: 45.0_f32.to_radians(),
            near_plane: 0.1,
            far_plane: 50000.0,  // Balanced far plane for both near precision and distant objects
        }
    }
    
    /// Create a camera with custom projection parameters
    pub fn with_projection(position: Vec3, fov: f32, near_plane: f32, far_plane: f32) -> Self {
        Self {
            position,
            pitch: 0.0,
            yaw: 0.0,
            roll: 0.0,
            fov,
            near_plane,
            far_plane,
        }
    }
    
    /// Get the camera's position
    pub fn position(&self) -> Vec3 {
        self.position
    }
    
    /// Get the camera's rotation as quaternion
    pub fn rotation(&self) -> Quat {
        Quat::from_euler(glam::EulerRot::YXZ, self.yaw, self.pitch, self.roll)
    }
    
    /// Get the view matrix for rendering
    pub fn view_matrix(&self) -> Mat4 {
        let rotation = self.rotation();
        let forward = rotation * Vec3::NEG_Z;
        let target = self.position + forward;
        let up = rotation * Vec3::Y;
        
        Mat4::look_at_rh(self.position, target, up)
    }
    
    /// Get the projection matrix for rendering (near-range for regular objects)
    pub fn projection_matrix(&self, aspect_ratio: f32) -> Mat4 {
        let mut proj = Mat4::perspective_rh(self.fov, aspect_ratio, self.near_plane, self.far_plane);
        // Flip Y for Vulkan coordinate system
        proj.y_axis.y *= -1.0;
        proj
    }

    /// Get the far-range projection matrix for celestial objects
    /// Uses extended far plane to render distant objects without z-fighting on close objects
    /// The far pass starts exactly where the near pass ends to maintain depth buffer compatibility
    pub fn far_projection_matrix(&self, aspect_ratio: f32) -> Mat4 {
        // Far pass: starts exactly where near pass ends and extends much further
        let far_near = self.far_plane;  // Start exactly at near far plane
        let far_far = self.far_plane * 10.0;   // Extend 10x beyond

        let mut proj = Mat4::perspective_rh(self.fov, aspect_ratio, far_near, far_far);
        // Flip Y for Vulkan coordinate system
        proj.y_axis.y *= -1.0;
        proj
    }

    /// Get field of view in radians
    pub fn fov(&self) -> f32 {
        self.fov
    }
    
    /// Get near clipping plane distance
    pub fn near_plane(&self) -> f32 {
        self.near_plane
    }
    
    /// Get far clipping plane distance
    pub fn far_plane(&self) -> f32 {
        self.far_plane
    }
    
    /// Set field of view in radians
    pub fn set_fov(&mut self, fov: f32) {
        self.fov = fov;
    }
    
    /// Set near clipping plane distance
    pub fn set_near_plane(&mut self, near: f32) {
        self.near_plane = near;
    }
    
    /// Set far clipping plane distance
    pub fn set_far_plane(&mut self, far: f32) {
        self.far_plane = far;
    }
    
    /// Set all projection parameters at once
    pub fn set_projection(&mut self, fov: f32, near_plane: f32, far_plane: f32) {
        self.fov = fov;
        self.near_plane = near_plane;
        self.far_plane = far_plane;
    }
    
    /// Move the camera forward/backward in the direction it's facing
    pub fn move_forward(&mut self, amount: f32) {
        let rotation = self.rotation();
        let forward = rotation * Vec3::NEG_Z;
        self.position += forward * amount;
    }
    
    /// Move the camera right/left (strafe)
    pub fn move_right(&mut self, amount: f32) {
        let rotation = self.rotation();
        let right = rotation * Vec3::X;
        self.position += right * amount;
    }
    
    /// Move the camera up/down in world space
    pub fn move_up(&mut self, amount: f32) {
        self.position.y += amount;
    }
    
    /// Rotate the camera (pitch and yaw) respecting current roll
    pub fn rotate(&mut self, pitch_delta: f32, yaw_delta: f32) {
        // Get current camera rotation as quaternion
        let current_rotation = self.rotation();
        
        // Get local axes (right and up) that respect the current roll
        let right = current_rotation * Vec3::X;
        let up = current_rotation * Vec3::Y;
        
        // Apply pitch rotation around the local right axis
        let pitch_rotation = Quat::from_axis_angle(right, pitch_delta);
        
        // Apply yaw rotation around the local up axis
        let yaw_rotation = Quat::from_axis_angle(up, yaw_delta);
        
        // Combine rotations and apply to camera
        let new_rotation = yaw_rotation * pitch_rotation * current_rotation;
        
        // Extract back to Euler angles
        let (yaw, pitch, roll) = new_rotation.to_euler(glam::EulerRot::YXZ);
        self.yaw = yaw;
        self.pitch = pitch;
        self.roll = roll;
    }
    
    /// Roll the camera
    pub fn roll(&mut self, amount: f32) {
        self.roll += amount;
    }
    
    /// Set the camera position
    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }
    
    /// Set the camera rotation
    pub fn set_rotation(&mut self, pitch: f32, yaw: f32, roll: f32) {
        self.pitch = pitch;
        self.yaw = yaw;
        self.roll = roll;
    }
}

impl Default for Camera {
    fn default() -> Self {
        // Start at the center (origin) of the system
        Self::new(Vec3::new(0.0, 0.0, 0.0))
    }
}

impl From<crate::config::CameraConfigData> for Camera {
    fn from(data: crate::config::CameraConfigData) -> Self {
        let mut camera = Self::new(data.position);
        camera.set_rotation(data.pitch, data.yaw, data.roll);
        camera.set_fov(data.fov.to_radians());
        camera
    }
}

impl From<&Camera> for crate::config::CameraConfigData {
    fn from(camera: &Camera) -> Self {
        Self {
            position: camera.position,
            pitch: camera.pitch,
            yaw: camera.yaw,
            roll: camera.roll,
            move_speed: 5.0, // Default, would need to be stored in Camera if configurable
            mouse_sensitivity: 0.003, // Default
            fov: camera.fov.to_degrees(),
        }
    }
}
