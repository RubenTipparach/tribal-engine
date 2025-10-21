use glam::{Mat4, Quat, Vec3, DVec3, DQuat};
use crate::nebula::NebulaConfig;
use crate::core::Camera;
use crate::scene::{SceneGraph, ObjectType, ObjectId};
use crate::gizmo::{GizmoState, ObjectPicker};
use crate::game_manager::GameManager;
use serde::{Serialize, Deserialize};

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

/// Star shader configuration
#[derive(Clone)]
pub struct StarConfig {
    /// Star color tint (RGB)
    pub color: Vec3,
    /// Gamma correction value
    pub gamma: f32,
    /// Exposure multiplier
    pub exposure: f32,
    /// Animation speed for high-frequency details
    pub speed_hi: f32,
    /// Animation speed for low-frequency details
    pub speed_low: f32,
    /// Zoom/scale of noise patterns
    pub zoom: f32,
}

impl Default for StarConfig {
    fn default() -> Self {
        Self {
            color: Vec3::new(1.0, 0.9, 0.7), // Yellowish
            gamma: 2.2,
            exposure: 40.2,
            speed_hi: 2.0,
            speed_low: 0.5,
            zoom: 0.5,
        }
    }
}

// Star config conversions
impl From<crate::config::StarConfigData> for StarConfig {
    fn from(data: crate::config::StarConfigData) -> Self {
        Self {
            color: data.color,
            gamma: data.gamma,
            exposure: data.exposure,
            speed_hi: data.speed_hi,
            speed_low: data.speed_low,
            zoom: data.zoom,
        }
    }
}

impl From<&StarConfig> for crate::config::StarConfigData {
    fn from(config: &StarConfig) -> Self {
        Self {
            color: config.color,
            gamma: config.gamma,
            exposure: config.exposure,
            speed_hi: config.speed_hi,
            speed_low: config.speed_low,
            zoom: config.zoom,
        }
    }
}

// SSAO config conversions
impl From<crate::config::SSAOConfigData> for SSAOConfig {
    fn from(data: crate::config::SSAOConfigData) -> Self {
        Self {
            enabled: data.enabled,
            radius: data.radius,
            bias: data.bias,
            power: data.power,
            kernel_size: data.kernel_size,
        }
    }
}

impl From<&SSAOConfig> for crate::config::SSAOConfigData {
    fn from(config: &SSAOConfig) -> Self {
        Self {
            enabled: config.enabled,
            radius: config.radius,
            bias: config.bias,
            power: config.power,
            kernel_size: config.kernel_size,
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

/// SSAO configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SSAOConfig {
    pub enabled: bool,
    pub radius: f32,
    pub bias: f32,
    pub power: f32,
    pub kernel_size: u32,
}

impl Default for SSAOConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            radius: 1.0,
            bias: 0.1,
            power: 2.0,
            kernel_size: 64,
        }
    }
}

/// Game state and logic
pub struct Game {
    /// Time accumulator for animations
    time: f32,
    /// Camera
    pub camera: Camera,
    /// Scene graph with all objects (legacy - being migrated to ECS)
    pub scene: SceneGraph,
    /// ECS World for space entities (nebula, star, ships, asteroids)
    pub ecs_world: crate::ecs::EcsWorld,
    /// Nebula entity ID in ECS
    pub nebula_entity: Option<hecs::Entity>,
    /// Star entity ID in ECS
    pub star_entity: Option<hecs::Entity>,
    /// Federation Cruiser entity ID in ECS
    pub fed_cruiser_entity: Option<hecs::Entity>,
    /// Hologram ship for turn-based movement planning
    pub hologram_ship_position: Option<DVec3>,
    /// Is the player currently dragging the hologram ship?
    pub dragging_hologram: bool,
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
    /// SSAO configuration
    pub ssao_config: SSAOConfig,
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
    /// Directional light settings
    pub directional_light: crate::core::lighting::DirectionalLight,
    /// Game Manager - play/pause state and scenario parameters
    pub game_manager: GameManager,
    /// Star configuration for shader parameters
    pub star_config: StarConfig,
}

impl Game {
    pub fn new() -> Self {
        let mut scene = SceneGraph::new();

        // Add default scene objects (legacy scene graph - for star, nebula, SSAO, Skybox)
        let star_id = scene.add_object("Star (Sun)".to_string(), ObjectType::Sphere);
        // Nebula is in ECS, but also in scene graph for transform editing via gizmo
        let nebula_id = scene.add_object("Nebula".to_string(), ObjectType::Nebula);
        scene.add_object("Skybox".to_string(), ObjectType::Skybox);
        scene.add_object("SSAO".to_string(), ObjectType::SSAO);
        scene.add_object("Game Manager".to_string(), ObjectType::GameManager);

        // Nebula positioned behind origin for space battles in the foreground
        let nebula_position = Vec3::new(0.0, 0.0, -10000.0);
        if let Some(nebula) = scene.get_object_mut(nebula_id) {
            nebula.transform.position = nebula_position;
            nebula.transform.scale = Vec3::splat(20000.0); // 1000x scale
        }

        // Star at center of nebula - procedural sphere with emissive material
        if let Some(star) = scene.get_object_mut(star_id) {
            star.transform.position = nebula_position; // Same as nebula center
            star.transform.scale = Vec3::splat(50.0); // Large visible star
        }

        // Initialize ECS world with nebula (1000x scale) and star
        let mut ecs_world = crate::ecs::EcsWorld::new();
        let (nebula_entity, star_entity) = crate::ecs::init::init_default_scene(&mut ecs_world.world);

        // Add Federation Cruiser at origin
        use glam::{DVec3, DQuat};
        let fed_cruiser_entity = crate::ecs::init::create_ship_entity(
            &mut ecs_world.world,
            "Federation Cruiser".to_string(),
            DVec3::new(0.0, 0.0, 0.0), // At origin
            DQuat::IDENTITY,
        );

        let mut game = Self {
            time: 0.0,
            camera: Camera::default(),
            scene,
            ecs_world,
            nebula_entity: Some(nebula_entity),
            star_entity: Some(star_entity),
            fed_cruiser_entity: Some(fed_cruiser_entity),
            hologram_ship_position: None,
            dragging_hologram: false,
            gizmo_state: GizmoState::new(),
            object_picker: ObjectPicker::new(),
            ship_velocity: Vec3::ZERO,
            ship_angular_velocity: Vec3::ZERO,
            move_speed: 5.0,
            rotation_speed: 2.0,
            skybox_config: SkyboxConfig::default(),
            nebula_config: NebulaConfig::default(),
            ssao_config: SSAOConfig::default(),
            focus_animation: CameraFocusAnimation::new(),
            lock_camera_up: true, // Default to locked (world Y up)
            scene_dirty: false,
            config_dirty: false,
            notifications: Vec::new(),
            material: crate::material::MaterialProperties::default(),
            material_library: crate::material_library::MaterialLibrary::default(),
            current_material_name: "New Material".to_string(),
            material_editor_open: false,
            directional_light: crate::core::lighting::DirectionalLight::default(),
            game_manager: GameManager::default(),
            star_config: StarConfig::default(),
        };

        // Sync nebula transform from scene to ECS
        game.sync_nebula_transform();

        game
    }

    /// Handle mouse hover for object picking
    pub fn handle_mouse_hover(&mut self, mouse_x: f32, mouse_y: f32, viewport_width: f32, viewport_height: f32) {
        // Check gizmo hover if enabled and object selected (edit mode)
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
        // In play mode, check for hologram ship interaction first
        if self.game_manager.mode == crate::game_manager::GameMode::Play {
            if self.handle_hologram_click(mouse_x, mouse_y, viewport_width, viewport_height) {
                return;
            }
        }

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
        // In play mode, check for hologram dragging first
        if self.game_manager.mode == crate::game_manager::GameMode::Play && self.dragging_hologram {
            self.handle_hologram_drag(new_mouse.0, new_mouse.1, viewport_width, viewport_height);
            return;
        }

        if !self.gizmo_state.using_gizmo {
            return;
        }

        if let Some(obj) = self.scene.selected_object_mut() {
            let mut transform_changed = false;
            let obj_type = obj.object_type.clone(); // Store for later check

            // Don't allow moving the star - it's always locked to nebula center
            if obj_type == ObjectType::Sphere {
                return;
            }

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

            // End the mutable borrow before calling other methods
            let _ = obj;

            // Mark scene dirty if transform changed
            if transform_changed {
                self.mark_scene_dirty();

                // If nebula was transformed, sync to ECS entity and update star position
                if obj_type == ObjectType::Nebula {
                    self.sync_nebula_transform();
                    self.sync_star_to_nebula();
                }
            }
        }
    }

    /// Handle mouse release
    pub fn handle_mouse_release(&mut self) {
        // In play mode, stop hologram dragging
        if self.game_manager.mode == crate::game_manager::GameMode::Play {
            self.handle_hologram_release();
        }

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

    /// Sync nebula scene object transform to ECS entity
    /// Called when the nebula transform is changed via gizmo or loaded from scene
    pub fn sync_nebula_transform(&mut self) {
        use glam::{DVec3, DQuat};
        use crate::ecs::components::{Position, Rotation};

        // Find nebula scene object
        if let Some(nebula_id) = self.scene.find_by_type(ObjectType::Nebula) {
            if let Some(nebula_obj) = self.scene.get_object(nebula_id) {
                if let Some(entity) = self.nebula_entity {
                    // Update ECS position from scene object
                    if let Ok(mut pos) = self.ecs_world.world.get::<&mut Position>(entity) {
                        pos.0 = DVec3::new(
                            nebula_obj.transform.position.x as f64,
                            nebula_obj.transform.position.y as f64,
                            nebula_obj.transform.position.z as f64,
                        );
                    }

                    // Update ECS rotation from scene object (quaternion)
                    if let Ok(mut rot) = self.ecs_world.world.get::<&mut Rotation>(entity) {
                        let quat = nebula_obj.transform.rotation;
                        rot.0 = DQuat::from_xyzw(quat.x as f64, quat.y as f64, quat.z as f64, quat.w as f64);
                    }
                }
            }
        }
    }

    /// Sync star position to nebula center
    /// Called whenever the nebula is moved to keep the star at its center
    pub fn sync_star_to_nebula(&mut self) {
        // Get nebula position
        if let Some(nebula_id) = self.scene.find_by_type(ObjectType::Nebula) {
            if let Some(nebula_obj) = self.scene.get_object(nebula_id) {
                let nebula_pos = nebula_obj.transform.position;

                // Update star position to match nebula center
                if let Some(star_id) = self.scene.find_by_type(ObjectType::Sphere) {
                    if let Some(star_obj) = self.scene.get_object_mut(star_id) {
                        star_obj.transform.position = nebula_pos;
                    }
                }
            }
        }
    }

    /// Get nebula model matrix from ECS entity
    pub fn get_nebula_model_matrix(&self) -> Mat4 {
        use crate::ecs::components::{Position, Rotation};

        if let Some(entity) = self.nebula_entity {
            if let Ok(pos) = self.ecs_world.world.get::<&Position>(entity) {
                if let Ok(rot) = self.ecs_world.world.get::<&Rotation>(entity) {
                    // Get scale from scene object (scale isn't in ECS yet)
                    let scale = if let Some(nebula_id) = self.scene.find_by_type(ObjectType::Nebula) {
                        if let Some(nebula_obj) = self.scene.get_object(nebula_id) {
                            nebula_obj.transform.scale
                        } else {
                            Vec3::ONE
                        }
                    } else {
                        Vec3::ONE
                    };

                    // Convert from f64 to f32 for rendering
                    let position = Vec3::new(pos.0.x as f32, pos.0.y as f32, pos.0.z as f32);
                    let rotation = Quat::from_xyzw(
                        rot.0.x as f32,
                        rot.0.y as f32,
                        rot.0.z as f32,
                        rot.0.w as f32,
                    );

                    return Mat4::from_scale_rotation_translation(scale, rotation, position);
                }
            }
        }

        Mat4::IDENTITY
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

    /// Get all visible cubes with their model matrices (includes nebula and spheres for picking)
    pub fn get_visible_cubes(&self) -> Vec<Mat4> {
        let in_edit_mode = self.game_manager.mode == crate::game_manager::GameMode::Edit;
        self.scene
            .objects_sorted()
            .iter()
            .filter(|obj| obj.visible)
            .filter(|obj| !obj.editor_only || in_edit_mode)
            .filter(|obj| matches!(obj.object_type, ObjectType::Cube))
            .map(|obj| obj.transform.model_matrix())
            .collect()
    }

    /// Get all visible sphere objects (returns model matrix)
    pub fn get_visible_spheres(&self) -> Vec<Mat4> {
        let in_edit_mode = self.game_manager.mode == crate::game_manager::GameMode::Edit;
        self.scene
            .objects_sorted()
            .iter()
            .filter(|obj| obj.visible)
            .filter(|obj| !obj.editor_only || in_edit_mode)
            .filter(|obj| matches!(obj.object_type, ObjectType::Sphere))
            .map(|obj| obj.transform.model_matrix())
            .collect()
    }

    /// Get all visible mesh objects (returns path and model matrix)
    pub fn get_visible_meshes(&self) -> Vec<(String, Mat4)> {
        let in_edit_mode = self.game_manager.mode == crate::game_manager::GameMode::Edit;
        self.scene
            .objects_sorted()
            .iter()
            .filter(|obj| obj.visible)
            .filter(|obj| !obj.editor_only || in_edit_mode)
            .filter_map(|obj| {
                if let ObjectType::Mesh(path) = &obj.object_type {
                    Some((path.clone(), obj.transform.model_matrix()))
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get outlined objects (selected or highlighted objects)
    /// Returns: Vec<(mesh_path, model_matrix, outline_color, outline_width)>
    pub fn get_outlined_objects(&self) -> Vec<(String, Mat4, glam::Vec4, f32)> {
        let in_edit_mode = self.game_manager.mode == crate::game_manager::GameMode::Edit;

        // In edit mode, outline the selected object
        if in_edit_mode {
            if let Some(selected_obj) = self.scene.selected_object() {
                if let ObjectType::Mesh(ref mesh_path) = selected_obj.object_type {
                    if selected_obj.visible {
                        let model_matrix = selected_obj.transform.model_matrix();
                        let outline_color = glam::Vec4::new(1.0, 0.5, 0.0, 1.0); // Orange outline
                        let outline_width = 0.02; // 2cm outline
                        return vec![(mesh_path.clone(), model_matrix, outline_color, outline_width)];
                    }
                }
            }
        }

        Vec::new()
    }

    /// Update ship bounds when mesh is loaded
    /// This is called from the renderer after loading a mesh
    pub fn update_ship_bounds(&mut self, mesh_path: &str, bounds_min: Vec3, bounds_max: Vec3) {
        // Check if this is the Fed Cruiser mesh
        if mesh_path.contains("Fed_cruiser") {
            if let Some(fed_entity) = self.fed_cruiser_entity {
                if let Ok(ship) = self.ecs_world.world.query_one_mut::<&mut crate::ecs::components::Ship>(fed_entity) {
                    ship.bounds_min = bounds_min;
                    ship.bounds_max = bounds_max;
                    // Updated Fed Cruiser bounds
                }
            }
        }
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

    /// Get current game time
    pub fn time(&self) -> f32 {
        self.time
    }

    /// Enter play mode - save editor state and reload fresh game instance from disk
    pub fn enter_play_mode(&mut self) {
        // 1. Save current editor state (scene + all configs)
        if let Err(e) = crate::ui::UiManager::save_scene_and_configs(self) {
            eprintln!("Failed to save editor state: {}", e);
            self.add_notification("Failed to save editor state!".to_string(), 3.0);
            return;
        }

        // 2. Load fresh game instance from saved files
        let fresh_game = Self::load_fresh_instance();

        // 3. Replace self with fresh instance (preserve only runtime state)
        *self = fresh_game;

        // 4. Set game mode to Play
        self.game_manager.mode = crate::game_manager::GameMode::Play;
        self.game_manager.pause_state = crate::game_manager::PauseState::Running;

        // 5. Clear any editor selections and disable gizmo
        self.scene.deselect();
        self.gizmo_state.enabled = false;

        // 6. Initialize hologram at ship position for movement planning
        self.spawn_hologram_ship();

        self.add_notification("Play mode started".to_string(), 2.0);
    }

    /// Exit play mode - discard game instance and reload editor state from disk
    pub fn exit_play_mode(&mut self) {
        // 1. Discard current play mode state (don't save)
        // 2. Load fresh editor state from disk
        let editor_game = Self::load_fresh_instance();

        // 3. Replace self with editor instance
        *self = editor_game;

        // 4. Set game mode back to Edit
        self.game_manager.mode = crate::game_manager::GameMode::Edit;

        self.add_notification("Returned to editor".to_string(), 2.0);
    }

    /// Load a fresh game instance from saved scene and config files
    fn load_fresh_instance() -> Self {
        let mut game = Self::new();

        // Load scene from disk
        crate::ui::UiManager::load_scene_on_startup(&mut game);

        // Load all configs from disk
        crate::ui::UiManager::load_all_configs(&mut game);

        // Clear dirty flags since we just loaded from disk
        game.scene_dirty = false;
        game.config_dirty = false;

        game
    }

    // ===== HOLOGRAM SHIP TURN-BASED MOVEMENT =====

    /// Initialize hologram ship at current ship position when entering play mode or starting turn
    pub fn spawn_hologram_ship(&mut self) {
        if let Some(fed_entity) = self.fed_cruiser_entity {
            if let Ok(mut query) = self.ecs_world.world.query_one::<(&crate::ecs::components::Position, &crate::ecs::components::Rotation)>(fed_entity) {
                if let Some((position, rotation)) = query.get() {
                    // Start hologram slightly offset forward from ship so it's visible
                    let forward = (rotation.0 * DVec3::Z).normalize();
                    let offset_distance = 15.0; // 15 meters forward
                    self.hologram_ship_position = Some(position.0 + forward * offset_distance);
                }
            }
        }
    }

    /// Ray-plane intersection for dragging hologram ship on its local XZ plane
    /// Returns intersection point in world space, or None if no intersection
    fn intersect_ray_with_ship_plane(
        &self,
        ray_origin: DVec3,
        ray_direction: DVec3,
    ) -> Option<DVec3> {
        if let Some(fed_entity) = self.fed_cruiser_entity {
            if let Ok(mut query) = self.ecs_world.world.query_one::<(&crate::ecs::components::Position, &crate::ecs::components::Rotation)>(fed_entity) {
                if let Some((position, rotation)) = query.get() {
                    // The ship's local XZ plane is its local coordinate system
                    // We want to constrain movement to this plane (ship's "deck")

                    // Plane normal is ship's local Y axis (up vector)
                    let plane_normal = rotation.0 * DVec3::Y; // Transform local up to world space

                    // Plane point is ship's current position
                    let plane_point = position.0;

                    // Ray-plane intersection: t = (plane_point - ray_origin) · plane_normal / (ray_direction · plane_normal)
                    let denominator = ray_direction.dot(plane_normal);

                    // Check if ray is parallel to plane (or pointing away)
                    if denominator.abs() < 0.0001 {
                        return None;
                    }

                    let t = (plane_point - ray_origin).dot(plane_normal) / denominator;

                    // Check if intersection is in front of camera
                    if t < 0.0 {
                        return None;
                    }

                    // Calculate intersection point
                    let intersection = ray_origin + ray_direction * t;
                    return Some(intersection);
                }
            }
        }
        None
    }

    /// Handle mouse click for hologram ship interaction (in play mode)
    pub fn handle_hologram_click(&mut self, mouse_x: f32, mouse_y: f32, viewport_width: f32, viewport_height: f32) -> bool {
        if self.game_manager.mode != crate::game_manager::GameMode::Play {
            return false;
        }

        // Create ray from camera through mouse position
        let (ray_origin, ray_direction) = self.camera.screen_to_ray(
            mouse_x,
            mouse_y,
            viewport_width,
            viewport_height,
        );

        // Check if clicking on hologram ship
        if let Some(hologram_pos) = self.hologram_ship_position {
            // Simple sphere intersection test (using ship bounds)
            if let Some(fed_entity) = self.fed_cruiser_entity {
                if let Ok(mut query) = self.ecs_world.world.query_one::<&crate::ecs::components::Ship>(fed_entity) {
                    if let Some(ship) = query.get() {
                        let bounds_radius = ((ship.bounds_max - ship.bounds_min).length() * 0.5) as f64;
                        let to_hologram = hologram_pos - ray_origin;
                        let projection = to_hologram.dot(ray_direction);

                        if projection > 0.0 {
                            let closest_point = ray_origin + ray_direction * projection;
                            let distance = (closest_point - hologram_pos).length();

                            if distance < bounds_radius {
                                // Start dragging hologram
                                self.dragging_hologram = true;
                                return true;
                            }
                        }
                    }
                }
            }
        } else {
            // If no hologram exists, spawn it at ship position
            self.spawn_hologram_ship();
        }

        false
    }

    /// Handle mouse drag for hologram ship (in play mode)
    pub fn handle_hologram_drag(&mut self, mouse_x: f32, mouse_y: f32, viewport_width: f32, viewport_height: f32) {
        if !self.dragging_hologram {
            return;
        }

        // Create ray from camera through mouse position
        let (ray_origin, ray_direction) = self.camera.screen_to_ray(
            mouse_x,
            mouse_y,
            viewport_width,
            viewport_height,
        );

        // Intersect with ship's local XZ plane
        if let Some(intersection) = self.intersect_ray_with_ship_plane(ray_origin, ray_direction) {
            // Constrain to movement range
            if let Some(fed_entity) = self.fed_cruiser_entity {
                if let Ok(mut query) = self.ecs_world.world.query_one::<(&crate::ecs::components::Position, &crate::ecs::components::Ship)>(fed_entity) {
                    if let Some((position, ship)) = query.get() {
                        let ship_pos = position.0;
                        let offset = intersection - ship_pos;
                        let distance = offset.length();

                        // Clamp to max movement range
                        let clamped_offset = if distance > ship.max_movement_range as f64 {
                            offset.normalize() * ship.max_movement_range as f64
                        } else {
                            offset
                        };

                        let new_hologram_pos = ship_pos + clamped_offset;

                        // Calculate rotation toward hologram
                        if clamped_offset.length() > 0.001 {
                            // Direction from ship to hologram on XZ plane
                            let direction = DVec3::new(clamped_offset.x, 0.0, clamped_offset.z).normalize();

                            // Calculate target rotation (yaw toward hologram)
                            let target_rotation = DQuat::from_rotation_arc(DVec3::Z, direction);

                            // Constrain rotation to 90 degrees from turn start
                            let angle_diff = ship.turn_start_rotation.angle_between(target_rotation);
                            let max_rotation = ship.max_rotation_angle as f64;

                            let final_rotation = if angle_diff > max_rotation {
                                // Slerp to max allowed angle
                                ship.turn_start_rotation.slerp(target_rotation, max_rotation / angle_diff)
                            } else {
                                target_rotation
                            };

                            // Update ship's planned rotation
                            if let Ok(mut rotation_query) = self.ecs_world.world.query_one::<&mut crate::ecs::components::Rotation>(fed_entity) {
                                if let Some(rotation) = rotation_query.get() {
                                    rotation.0 = final_rotation;
                                }
                            }
                        }

                        self.hologram_ship_position = Some(new_hologram_pos);
                    }
                }
            }
        }
    }

    /// Stop dragging hologram ship
    pub fn handle_hologram_release(&mut self) {
        self.dragging_hologram = false;
    }

    /// Generate line segments for the 90-degree rotation arc boundary
    /// Returns Vec<Vec3> of line segment endpoints (pairs of vertices)
    pub fn generate_rotation_arc_boundary(&self) -> Vec<Vec3> {
        let mut vertices = Vec::new();

        if self.game_manager.mode != crate::game_manager::GameMode::Play {
            return vertices;
        }

        if let Some(fed_entity) = self.fed_cruiser_entity {
            if let Ok(mut query) = self.ecs_world.world.query_one::<(&crate::ecs::components::Position, &crate::ecs::components::Ship)>(fed_entity) {
                if let Some((position, ship)) = query.get() {
                    let ship_pos = position.0.as_vec3();
                    let start_rotation = ship.turn_start_rotation;

                    // Get the ship's forward direction at turn start
                    let start_forward = (start_rotation * DVec3::Z).as_vec3();

                    // Maximum rotation angle (90 degrees = PI/2)
                    let max_angle = ship.max_rotation_angle;
                    let half_angle = max_angle / 2.0;

                    // Arc radius for visualization
                    let arc_radius = ship.max_movement_range * 0.75; // 75% of movement range

                    // Number of segments for the arc
                    const ARC_SEGMENTS: usize = 32;

                    // Calculate right vector perpendicular to start_forward
                    let up = Vec3::Y;
                    let right = start_forward.cross(up).normalize();

                    // Generate arc points from -45° to +45° around start_forward
                    let mut arc_points = Vec::with_capacity(ARC_SEGMENTS + 1);

                    for i in 0..=ARC_SEGMENTS {
                        let t = i as f32 / ARC_SEGMENTS as f32;
                        let angle = -half_angle + t * max_angle;

                        // Rotate start_forward around Y axis
                        let rotation = Quat::from_axis_angle(up, angle);
                        let direction = rotation * start_forward;
                        let point = ship_pos + direction * arc_radius;

                        arc_points.push(point);
                    }

                    // Convert arc points to line segments
                    for i in 0..arc_points.len() - 1 {
                        vertices.push(arc_points[i]);
                        vertices.push(arc_points[i + 1]);
                    }

                    // Draw lines from ship center to arc endpoints (forming a cone shape)
                    if let Some(first) = arc_points.first() {
                        vertices.push(ship_pos);
                        vertices.push(*first);
                    }
                    if let Some(last) = arc_points.last() {
                        vertices.push(ship_pos);
                        vertices.push(*last);
                    }
                }
            }
        }

        vertices
    }

    /// Confirm movement and execute ship to hologram position
    pub fn execute_ship_movement(&mut self) {
        if let Some(hologram_pos) = self.hologram_ship_position {
            if let Some(fed_entity) = self.fed_cruiser_entity {
                // Get current position first
                let current_pos = if let Ok(mut query) = self.ecs_world.world.query_one::<&crate::ecs::components::Position>(fed_entity) {
                    query.get().map(|pos| pos.0)
                } else {
                    None
                };

                // Update ship's planned position and control point
                if let Ok(mut query) = self.ecs_world.world.query_one::<&mut crate::ecs::components::Ship>(fed_entity) {
                    if let Some(ship) = query.get() {
                        ship.planned_position = hologram_pos;

                        // Calculate bezier control point based on ship velocity/momentum
                        // For now, simple: midpoint between current and target
                        if let Some(curr_pos) = current_pos {
                            ship.control_point = (curr_pos + hologram_pos) * 0.5;
                        }
                    }
                }

                // Actually move the ship to hologram position
                if let Ok(mut query) = self.ecs_world.world.query_one::<&mut crate::ecs::components::Position>(fed_entity) {
                    if let Some(position) = query.get() {
                        position.0 = hologram_pos;
                    }
                }

                // Clear hologram after movement
                self.hologram_ship_position = None;
                self.dragging_hologram = false;

                self.add_notification("Ship moved!".to_string(), 2.0);
            }
        }
    }
}
