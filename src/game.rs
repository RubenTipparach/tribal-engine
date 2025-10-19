use glam::{Mat4, Quat, Vec3};
use crate::nebula::NebulaConfig;
use crate::core::Camera;
use crate::scene::{SceneGraph, ObjectType, ObjectId};
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

/// Camera focus animation state
struct CameraFocusAnimation {
    active: bool,
    start_position: Vec3,
    start_pitch: f32,
    start_yaw: f32,
    start_roll: f32,
    target_position: Vec3,
    target_pitch: f32,
    target_yaw: f32,
    target_roll: f32,
    progress: f32, // 0.0 to 1.0
    duration: f32, // Total animation duration in seconds
}

impl CameraFocusAnimation {
    fn new() -> Self {
        Self {
            active: false,
            start_position: Vec3::ZERO,
            start_pitch: 0.0,
            start_yaw: 0.0,
            start_roll: 0.0,
            target_position: Vec3::ZERO,
            target_pitch: 0.0,
            target_yaw: 0.0,
            target_roll: 0.0,
            progress: 0.0,
            duration: 0.5, // Half second animation
        }
    }
}

/// Notification message for UI
#[derive(Clone)]
pub struct Notification {
    pub message: String,
    pub time_remaining: f32, // seconds
}

impl Notification {
    pub fn new(message: String, duration: f32) -> Self {
        Self {
            message,
            time_remaining: duration,
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
    /// Camera focus animation state
    focus_animation: CameraFocusAnimation,
    /// Lock camera up vector to world Y axis
    pub lock_camera_up: bool,
    /// Scene dirty flag (needs save)
    pub scene_dirty: bool,
    /// Config dirty flag (needs save)
    pub config_dirty: bool,
    /// Active notifications
    pub notifications: Vec<Notification>,
    /// Material properties for mesh rendering
    pub material: crate::material::MaterialProperties,
    /// Material library
    pub material_library: crate::material_library::MaterialLibrary,
    /// Current material name being edited
    pub current_material_name: String,
    /// Material editor visibility
    pub material_editor_open: bool,
}

impl Game {
    pub fn new() -> Self {
        let mut scene = SceneGraph::new();

        // Add default scene objects
        let cube1_id = scene.add_object("Cube 1".to_string(), ObjectType::Cube);
        let cube2_id = scene.add_object("Cube 2".to_string(), ObjectType::Cube);
        scene.add_object("Nebula".to_string(), ObjectType::Nebula);
        scene.add_object("Skybox".to_string(), ObjectType::Skybox);

        // Position the second cube offset from the first
        if let Some(cube2) = scene.get_object_mut(cube2_id) {
            cube2.transform.position = glam::Vec3::new(3.0, 0.0, 0.0);
        }

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
            focus_animation: CameraFocusAnimation::new(),
            lock_camera_up: true, // Default to locked (world Y up)
            scene_dirty: false,
            config_dirty: false,
            notifications: Vec::new(),
            material: crate::material::MaterialProperties::default(),
            material_library: crate::material_library::MaterialLibrary::default(),
            current_material_name: "New Material".to_string(),
            material_editor_open: false,
        }
    }

    /// Handle mouse hover for object picking
    pub fn handle_mouse_hover(&mut self, mouse_x: f32, mouse_y: f32, viewport_width: f32, viewport_height: f32) {
        // Check gizmo hover first if enabled and object selected
        if self.gizmo_state.enabled && self.scene.selected_object().is_some() {
            let obj = self.scene.selected_object().unwrap();
            let object_pos = obj.transform.position;
            let object_rotation = obj.transform.rotation;
            self.gizmo_state.pick_axis(
                mouse_x,
                mouse_y,
                viewport_width,
                viewport_height,
                object_pos,
                object_rotation,
                &self.camera,
            );
        } else {
            // Otherwise check object hover
            self.object_picker.pick_object(
                mouse_x,
                mouse_y,
                viewport_width,
                viewport_height,
                &self.scene,
                &self.camera,
            );
        }
    }

    /// Handle mouse click for object selection or gizmo drag start
    pub fn handle_mouse_click(&mut self, mouse_x: f32, mouse_y: f32, viewport_width: f32, viewport_height: f32) {
        // Check if clicking on gizmo first
        if self.gizmo_state.enabled && self.scene.selected_object().is_some() {
            let obj = self.scene.selected_object().unwrap();
            let object_pos = obj.transform.position;
            let object_rotation = obj.transform.rotation;
            let axis = self.gizmo_state.pick_axis(
                mouse_x,
                mouse_y,
                viewport_width,
                viewport_height,
                object_pos,
                object_rotation,
                &self.camera,
            );

            if axis != crate::gizmo::GizmoAxis::None {
                // Start dragging gizmo
                self.gizmo_state.start_drag(axis);
                return;
            }
        }

        // Otherwise try to select an object
        if let Some(object_id) = self.object_picker.pick_object(
            mouse_x,
            mouse_y,
            viewport_width,
            viewport_height,
            &self.scene,
            &self.camera,
        ) {
            // If clicking already selected object, focus on it
            if self.scene.selected_object_id() == Some(object_id) {
                self.focus_on_object(object_id);
            } else {
                self.scene.select_object(object_id);
            }
        }
    }

    /// Handle mouse drag for gizmo manipulation
    pub fn handle_mouse_drag(&mut self, old_mouse: (f32, f32), new_mouse: (f32, f32), viewport_width: f32, viewport_height: f32) {
        if !self.gizmo_state.using_gizmo {
            return;
        }

        if let Some(obj) = self.scene.selected_object_mut() {
            let mut transform_changed = false;

            match self.gizmo_state.mode {
                crate::gizmo::GizmoMode::Translate => {
                    let old_pos = obj.transform.position;
                    let new_pos = self.gizmo_state.apply_drag_translate(
                        old_mouse,
                        new_mouse,
                        viewport_width,
                        viewport_height,
                        obj.transform.position,
                        &self.camera,
                    );
                    obj.transform.position = new_pos;
                    transform_changed = old_pos != new_pos;
                }
                crate::gizmo::GizmoMode::Rotate => {
                    let old_rot = obj.transform.rotation;
                    let new_rot = self.gizmo_state.apply_drag_rotate(
                        old_mouse,
                        new_mouse,
                        viewport_width,
                        viewport_height,
                        obj.transform.position,
                        obj.transform.rotation,
                        &self.camera,
                    );
                    obj.transform.rotation = new_rot;
                    transform_changed = old_rot != new_rot;
                }
                crate::gizmo::GizmoMode::Scale => {
                    let old_scale = obj.transform.scale;
                    let new_scale = self.gizmo_state.apply_drag_scale(
                        old_mouse,
                        new_mouse,
                        viewport_width,
                        viewport_height,
                        obj.transform.position,
                        obj.transform.rotation,
                        obj.transform.scale,
                        &self.camera,
                    );
                    obj.transform.scale = new_scale;
                    transform_changed = old_scale != new_scale;
                }
            }

            // Mark scene dirty if transform changed
            if transform_changed {
                self.mark_scene_dirty();
            }
        }
    }

    /// Handle mouse release
    pub fn handle_mouse_release(&mut self) {
        self.gizmo_state.end_drag();
    }
    
    /// Update game logic
    pub fn update(&mut self, delta_time: f32) {
        self.time += delta_time;

        // Update camera focus animation
        if self.focus_animation.active {
            self.focus_animation.progress += delta_time / self.focus_animation.duration;

            if self.focus_animation.progress >= 1.0 {
                // Animation complete
                self.focus_animation.progress = 1.0;
                self.focus_animation.active = false;
            }

            // Smooth interpolation using ease-in-out cubic
            let t = self.focus_animation.progress;
            let eased_t = if t < 0.5 {
                4.0 * t * t * t
            } else {
                1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
            };

            // Lerp camera position
            let position = self.focus_animation.start_position.lerp(self.focus_animation.target_position, eased_t);

            // Lerp camera rotation
            let pitch = self.focus_animation.start_pitch + (self.focus_animation.target_pitch - self.focus_animation.start_pitch) * eased_t;
            let yaw = self.focus_animation.start_yaw + (self.focus_animation.target_yaw - self.focus_animation.start_yaw) * eased_t;
            let roll = self.focus_animation.start_roll + (self.focus_animation.target_roll - self.focus_animation.start_roll) * eased_t;

            self.camera.set_position(position);
            self.camera.set_rotation(pitch, yaw, roll);
        }

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

        // Update notifications
        self.notifications.retain_mut(|notif| {
            notif.time_remaining -= delta_time;
            notif.time_remaining > 0.0
        });
    }

    /// Add a notification message
    pub fn add_notification(&mut self, message: String, duration: f32) {
        self.notifications.push(Notification::new(message, duration));
    }

    /// Mark scene as dirty (needs save)
    pub fn mark_scene_dirty(&mut self) {
        self.scene_dirty = true;
    }

    /// Mark config as dirty (needs save)
    pub fn mark_config_dirty(&mut self) {
        self.config_dirty = true;
    }

    /// Check if anything needs saving
    pub fn is_dirty(&self) -> bool {
        self.scene_dirty || self.config_dirty
    }

    /// Get all visible cubes with their model matrices
    pub fn get_visible_cubes(&self) -> Vec<Mat4> {
        self.scene
            .get_by_type(ObjectType::Cube)
            .iter()
            .filter_map(|&id| self.scene.get_object(id))
            .filter(|obj| obj.visible)
            .map(|obj| obj.transform.model_matrix())
            .collect()
    }

    /// Get all visible mesh objects (returns path and model matrix)
    pub fn get_visible_meshes(&self) -> Vec<(String, Mat4)> {
        self.scene
            .objects_sorted()
            .iter()
            .filter(|obj| obj.visible)
            .filter_map(|obj| {
                if let ObjectType::Mesh(path) = &obj.object_type {
                    Some((path.clone(), obj.transform.model_matrix()))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Check if nebula is visible
    pub fn is_nebula_visible(&self) -> bool {
        if let Some(nebula_id) = self.scene.find_by_type(ObjectType::Nebula) {
            if let Some(nebula) = self.scene.get_object(nebula_id) {
                return nebula.visible;
            }
        }
        false
    }

    /// Check if skybox is visible
    pub fn is_skybox_visible(&self) -> bool {
        if let Some(skybox_id) = self.scene.find_by_type(ObjectType::Skybox) {
            if let Some(skybox) = self.scene.get_object(skybox_id) {
                return skybox.visible;
            }
        }
        false
    }

    /// Get directional light transform if visible
    pub fn get_directional_light(&self) -> Option<Mat4> {
        if let Some(light_id) = self.scene.find_by_type(ObjectType::DirectionalLight) {
            if let Some(light) = self.scene.get_object(light_id) {
                if light.visible {
                    return Some(light.transform.model_matrix());
                }
            }
        }
        None
    }

    /// Get the current model matrix for the cube (first cube for backwards compatibility)
    pub fn get_cube_model_matrix(&self) -> Mat4 {
        if let Some(cube_id) = self.scene.find_by_type(ObjectType::Cube) {
            if let Some(cube) = self.scene.get_object(cube_id) {
                return cube.transform.model_matrix();
            }
        }
        Mat4::IDENTITY
    }

    /// Check if any cube is visible
    pub fn is_cube_visible(&self) -> bool {
        self.scene
            .get_by_type(ObjectType::Cube)
            .iter()
            .any(|&id| {
                self.scene
                    .get_object(id)
                    .map(|obj| obj.visible)
                    .unwrap_or(false)
            })
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

    /// Focus camera on a specific object with smooth animation
    pub fn focus_on_object(&mut self, object_id: ObjectId) {
        if let Some(obj) = self.scene.get_object(object_id) {
            // Calculate the distance to place camera (at least 2x the bounding box size)
            let bbox_size = obj.bounding_box_size();
            let distance = (bbox_size * 2.5).max(5.0); // At least 5 units away

            // Calculate target position (camera looking at object from a nice angle)
            let object_pos = obj.transform.position;

            // Determine the up vector based on lock state
            let up = if self.lock_camera_up {
                // Always use world Y up
                Vec3::Y
            } else {
                // Use the object's local up vector
                obj.transform.rotation * Vec3::Y
            };

            // Calculate right vector for positioning camera
            let target_forward = Vec3::NEG_Z; // Arbitrary initial forward
            let right = target_forward.cross(up).normalize();
            let corrected_forward = up.cross(right).normalize();

            // Position camera at 45-degree angle above and to the side, respecting up vector
            let offset = right * (distance * 0.7) + up * (distance * 0.5) + corrected_forward * (distance * 0.7);
            let target_camera_pos = object_pos + offset;

            // Calculate rotation using quaternion look_at for proper centering
            let forward = (object_pos - target_camera_pos).normalize();

            // Build look-at matrix to get proper rotation
            // We need to construct a rotation that:
            // 1. Points camera forward (-Z) at the object
            // 2. Keeps the camera up (Y) aligned with the desired up vector
            let right = forward.cross(up).normalize();
            let corrected_up = right.cross(forward).normalize();

            // Create rotation matrix from basis vectors
            // Camera looks down -Z, so forward maps to -Z axis
            let rotation_matrix = Mat4::from_cols(
                right.extend(0.0),
                corrected_up.extend(0.0),
                (-forward).extend(0.0),
                Vec3::ZERO.extend(1.0),
            );

            let target_rotation = Quat::from_mat4(&rotation_matrix);

            // Extract euler angles from the target rotation
            let (target_yaw, target_pitch, target_roll) = target_rotation.to_euler(glam::EulerRot::YXZ);

            // Get current camera state
            let current_pos = self.camera.position();
            let current_rotation = self.camera.rotation();
            let (current_yaw, current_pitch, current_roll) = current_rotation.to_euler(glam::EulerRot::YXZ);

            // Start the animation
            self.focus_animation = CameraFocusAnimation {
                active: true,
                start_position: current_pos,
                start_pitch: current_pitch,
                start_yaw: current_yaw,
                start_roll: current_roll,
                target_position: target_camera_pos,
                target_pitch,
                target_yaw,
                target_roll,
                progress: 0.0,
                duration: 0.6, // 0.6 seconds for smooth animation
            };
        }
    }

    /// Reset camera up vector to world Y axis
    pub fn reset_camera_up(&mut self) {
        // Set camera roll to 0 to align with world Y up
        let current_rotation = self.camera.rotation();
        let (yaw, pitch, _roll) = current_rotation.to_euler(glam::EulerRot::YXZ);
        self.camera.set_rotation(pitch, yaw, 0.0);
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

    /// Orbit camera around the currently selected object
    pub fn orbit_camera_around_selected(&mut self, pitch_delta: f32, yaw_delta: f32) {
        // Get the selected object's position
        let target_pos = if let Some(obj) = self.scene.selected_object() {
            obj.transform.position
        } else {
            // If no object selected, just do regular camera rotation
            self.camera.rotate(pitch_delta, yaw_delta);
            return;
        };

        // Get current camera position and calculate distance to target
        let camera_pos = self.camera.position();
        let to_camera = camera_pos - target_pos;
        let distance = to_camera.length();

        // Convert to spherical coordinates relative to target
        let horizontal_dist = (to_camera.x * to_camera.x + to_camera.z * to_camera.z).sqrt();
        let mut pitch = (-to_camera.y).atan2(horizontal_dist);
        let mut yaw = to_camera.z.atan2(to_camera.x);

        // Apply rotation deltas
        pitch += pitch_delta;
        yaw += yaw_delta;

        // Clamp pitch to avoid gimbal lock
        pitch = pitch.clamp(-std::f32::consts::FRAC_PI_2 + 0.01, std::f32::consts::FRAC_PI_2 - 0.01);

        // Convert back to Cartesian coordinates
        let new_camera_pos = target_pos + Vec3::new(
            distance * pitch.cos() * yaw.cos(),
            -distance * pitch.sin(),
            distance * pitch.cos() * yaw.sin(),
        );

        // Update camera position and make it look at the target
        self.camera.set_position(new_camera_pos);

        // Calculate look direction
        let forward = (target_pos - new_camera_pos).normalize();

        // Create rotation from look direction
        let up = Vec3::Y;
        let right = forward.cross(up).normalize();
        let corrected_up = right.cross(forward).normalize();

        // Build rotation matrix
        let rotation_matrix = Mat4::from_cols(
            right.extend(0.0),
            corrected_up.extend(0.0),
            (-forward).extend(0.0),
            Vec3::ZERO.extend(1.0),
        );

        let rotation = Quat::from_mat4(&rotation_matrix);
        let (new_yaw, new_pitch, new_roll) = rotation.to_euler(glam::EulerRot::YXZ);

        self.camera.set_rotation(new_pitch, new_yaw, new_roll);
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
