mod gui_builder;

pub use gui_builder::{GuiPanelBuilder, GuiContentBuilder, SkyboxFxBuilder};

use imgui::{Context, Ui};
use crate::game::{Game, SkyboxConfig};
use crate::nebula::NebulaConfig;
use crate::config::EngineConfig;
use crate::scene::{SceneData, ObjectType};
use crate::gizmo::GizmoMode;
use glam::Quat;

const CONFIG_PATH: &str = "config/default.json";
const SCENE_PATH: &str = "config/scene.json";

/// Manages all UI rendering and interactions
pub struct UiManager;

impl UiManager {
    pub fn new() -> Self {
        Self
    }

    /// Build the skybox settings UI
    pub fn build_skybox_settings(ui: &Ui, game: &mut Game) {
        let mut save_clicked = false;
        let mut load_clicked = false;
        let mut reset_clicked = false;

        // Store original config to detect changes
        let orig_config = game.skybox_config.clone();

        GuiPanelBuilder::new(ui, "Skybox Settings")
            .size(350.0, 500.0)
            .position(270.0, 10.0)
            .build(|content| {
                content.text("Adjust skybox appearance in real-time");

                let config = &mut game.skybox_config;

                content
                    .header("Stars")
                    .slider_f32("Star Density", &mut config.star_density, 0.1, 10.0)
                    .slider_f32("Star Brightness", &mut config.star_brightness, 0.0, 10.0)
                    .header("Nebula")
                    .slider_f32("Nebula Intensity", &mut config.nebula_intensity, 0.0, 2.0)
                    .color_picker("Primary Color", &mut config.nebula_primary_color)
                    .color_picker("Secondary Color", &mut config.nebula_secondary_color)
                    .header("Background")
                    .slider_f32("Brightness", &mut config.background_brightness, 0.0, 0.5);

                let (s, l, r) = content.config_buttons();
                save_clicked = s;
                load_clicked = l;
                reset_clicked = r;
            });

        // Check if config changed
        if orig_config.star_density != game.skybox_config.star_density
            || orig_config.star_brightness != game.skybox_config.star_brightness
            || orig_config.nebula_intensity != game.skybox_config.nebula_intensity
            || orig_config.nebula_primary_color != game.skybox_config.nebula_primary_color
            || orig_config.nebula_secondary_color != game.skybox_config.nebula_secondary_color
            || orig_config.background_brightness != game.skybox_config.background_brightness
        {
            game.mark_config_dirty();
        }

        if save_clicked {
            Self::save_skybox_config(game);
        }
        if load_clicked {
            Self::load_skybox_config(game);
        }
        if reset_clicked {
            game.skybox_config = SkyboxConfig::default();
            game.mark_config_dirty();
        }
    }

    /// Build the nebula settings UI
    pub fn build_nebula_settings(ui: &Ui, game: &mut Game) {
        let mut save_clicked = false;
        let mut load_clicked = false;
        let mut reset_clicked = false;

        // Store original config to detect changes
        let orig_config = game.nebula_config.clone();

        GuiPanelBuilder::new(ui, "Nebula Settings")
            .size(380.0, 450.0)
            .position(270.0, 10.0)
            .build(|content| {
                content.text("Volumetric nebula raymarch shader");

                let config = &mut game.nebula_config;

                content
                    .header("Basic Controls")
                    .slider_f32("Scale", &mut config.scale, 0.1, 10.0)
                    .slider_f32("Zoom", &mut config.zoom, -2.0, 5.0)
                    .slider_f32("Density", &mut config.density, 0.0, 2.0)
                    .slider_f32("Brightness", &mut config.brightness, 0.1, 3.0)

                    .header("Colors - Center/Edge")
                    .color_picker("Center Color", &mut config.color_center)
                    .color_picker("Edge Color", &mut config.color_edge)

                    .header("Colors - Density")
                    .color_picker("Low Density", &mut config.color_density_low)
                    .color_picker("High Density", &mut config.color_density_high)

                    .header("Light")
                    .color_picker("Light Color", &mut config.light_color)
                    .slider_f32("Light Intensity", &mut config.light_intensity, 0.0, 0.1)

                    .header("Distance")
                    .slider_f32("Max Distance", &mut config.max_distance, 1.0, 50.0);

                let (s, l, r) = content.config_buttons();
                save_clicked = s;
                load_clicked = l;
                reset_clicked = r;
            });

        // Check if config changed
        if orig_config.scale != game.nebula_config.scale
            || orig_config.zoom != game.nebula_config.zoom
            || orig_config.density != game.nebula_config.density
            || orig_config.brightness != game.nebula_config.brightness
            || orig_config.color_center != game.nebula_config.color_center
            || orig_config.color_edge != game.nebula_config.color_edge
            || orig_config.color_density_low != game.nebula_config.color_density_low
            || orig_config.color_density_high != game.nebula_config.color_density_high
            || orig_config.light_color != game.nebula_config.light_color
            || orig_config.light_intensity != game.nebula_config.light_intensity
            || orig_config.max_distance != game.nebula_config.max_distance
        {
            game.mark_config_dirty();
        }

        if save_clicked {
            Self::save_nebula_config(game);
        }
        if load_clicked {
            Self::load_nebula_config(game);
        }
        if reset_clicked {
            game.nebula_config = NebulaConfig::default();
            game.mark_config_dirty();
        }
    }

    /// Build the scene hierarchy UI
    pub fn build_scene_hierarchy(ui: &Ui, game: &mut Game) {
        let mut save_scene_clicked = false;
        let mut load_scene_clicked = false;
        let mut clicked_obj_id: Option<usize> = None;
        let mut double_clicked_obj_id: Option<usize> = None;
        let mut duplicate_object_id: Option<usize> = None;

        GuiPanelBuilder::new(ui, "Scene Hierarchy")
            .size(250.0, 480.0)
            .position(10.0, 10.0)
            .build(|content| {
                content.text("Select objects to edit");
                content.text_disabled("Click selected to focus");
                content.separator();

                // Collect object data first to avoid borrow issues
                let objects: Vec<(usize, String)> = game
                    .scene
                    .objects_sorted()
                    .iter()
                    .map(|obj| (obj.id, obj.name.clone()))
                    .collect();
                let selected_id = game.scene.selected_object_id();

                for (id, name) in objects {
                    let is_selected = selected_id == Some(id);
                    let label = if is_selected {
                        format!("> {}", name)
                    } else {
                        format!("  {}", name)
                    };

                    if ui.selectable(&label) {
                        // If clicking already selected object, focus on it
                        if is_selected {
                            double_clicked_obj_id = Some(id);
                        } else {
                            clicked_obj_id = Some(id);
                        }
                    }

                    // Check for double-click (also focuses)
                    if ui.is_item_hovered() && ui.is_mouse_double_clicked(imgui::MouseButton::Left) {
                        double_clicked_obj_id = Some(id);
                    }
                }

                // Object manipulation buttons
                content.separator();
                content.header("Object Actions");

                // Duplicate button - only enabled if an object is selected
                let selected_id = game.scene.selected_object_id();
                if let Some(id) = selected_id {
                    // Check if the selected object is duplicatable
                    let can_duplicate = game.scene.get_object(id)
                        .map(|obj| !matches!(obj.object_type, crate::scene::ObjectType::Skybox | crate::scene::ObjectType::Nebula))
                        .unwrap_or(false);

                    if can_duplicate {
                        if ui.button("Duplicate") {
                            duplicate_object_id = Some(id);
                        }
                    } else {
                        ui.text_disabled("Cannot duplicate");
                    }
                } else {
                    ui.text_disabled("Select object first");
                }

                // Gizmo controls integrated here
                content.separator();
                content.header("Transform Tools");

                if ui.button("Translate (1)") {
                    game.gizmo_state.mode = GizmoMode::Translate;
                }
                ui.same_line();
                if game.gizmo_state.mode == GizmoMode::Translate {
                    ui.text("[X]");
                } else {
                    ui.text("[ ]");
                }

                if ui.button("Rotate (2)") {
                    game.gizmo_state.mode = GizmoMode::Rotate;
                }
                ui.same_line();
                if game.gizmo_state.mode == GizmoMode::Rotate {
                    ui.text("[X]");
                } else {
                    ui.text("[ ]");
                }

                if ui.button("Scale (3)") {
                    game.gizmo_state.mode = GizmoMode::Scale;
                }
                ui.same_line();
                if game.gizmo_state.mode == GizmoMode::Scale {
                    ui.text("[X]");
                } else {
                    ui.text("[ ]");
                }

                content.checkbox("Show Gizmo", &mut game.gizmo_state.enabled);

                // Camera up vector controls
                content.separator();
                content.header("Camera Up Vector");

                content.checkbox("Lock to World Y", &mut game.lock_camera_up);

                if ui.button("Reset Up Vector") {
                    game.reset_camera_up();
                }

                content.separator();
                let (s, l, _) = content.config_buttons();
                save_scene_clicked = s;
                load_scene_clicked = l;
            });

        if let Some(id) = clicked_obj_id {
            game.scene.select_object(id);
        }

        // Handle double-click to focus on object
        if let Some(id) = double_clicked_obj_id {
            game.scene.select_object(id);
            game.focus_on_object(id);
        }

        // Handle duplicate
        if let Some(id) = duplicate_object_id {
            if let Some(new_id) = game.scene.duplicate_object(id) {
                game.scene.select_object(new_id);
                game.mark_scene_dirty();
            }
        }

        if save_scene_clicked {
            Self::save_scene(game);
        }
        if load_scene_clicked {
            Self::load_scene(game);
        }
    }

    /// Build the transform editor UI for selected object (top-right corner)
    pub fn build_transform_editor(ui: &Ui, game: &mut Game) {
        let window_width = ui.io().display_size[0];
        let panel_width = 350.0;
        let mut transform_changed = false;

        GuiPanelBuilder::new(ui, "Transform")
            .size(panel_width, 320.0)
            .position(window_width - panel_width - 10.0, 10.0)
            .build(|content| {
                if let Some(obj) = game.scene.selected_object_mut() {
                    // Show selected object name prominently
                    content.text_colored([0.2, 1.0, 0.2, 1.0], "Selected:");
                    ui.same_line();
                    content.text(&obj.name);
                    content.separator();

                    // Store original values to detect changes
                    let orig_visible = obj.visible;
                    let orig_position = obj.transform.position;
                    let orig_scale = obj.transform.scale;
                    let (orig_pitch, orig_yaw, orig_roll) = obj.transform.euler_angles();

                    // Visibility
                    content.checkbox("Visible", &mut obj.visible);
                    content.separator();

                    // Position - using input fields (unbounded)
                    content.header("Position");
                    content.input_vec3("Position", &mut obj.transform.position);

                    // Rotation - using input fields with wrap-around
                    content.header("Rotation (degrees)");
                    let (pitch, yaw, roll) = obj.transform.euler_angles();
                    let mut pitch_deg = pitch.to_degrees();
                    let mut yaw_deg = yaw.to_degrees();
                    let mut roll_deg = roll.to_degrees();

                    content.input_angle("Pitch", &mut pitch_deg);
                    content.input_angle("Yaw", &mut yaw_deg);
                    content.input_angle("Roll", &mut roll_deg);

                    // Update rotation quaternion
                    obj.transform.set_euler_rotation(
                        pitch_deg.to_radians(),
                        yaw_deg.to_radians(),
                        roll_deg.to_radians(),
                    );

                    // Scale - using input fields (unbounded)
                    content.header("Scale");
                    content.input_vec3("Scale", &mut obj.transform.scale);

                    // Check if anything changed
                    if orig_visible != obj.visible
                        || orig_position != obj.transform.position
                        || orig_scale != obj.transform.scale
                        || orig_pitch != pitch_deg.to_radians()
                        || orig_yaw != yaw_deg.to_radians()
                        || orig_roll != roll_deg.to_radians()
                    {
                        transform_changed = true;
                    }

                    // Show object-specific settings hint
                    content.separator();
                    match obj.object_type {
                        ObjectType::Nebula => {
                            content.text("Select this object to see");
                            content.text("Nebula Settings panel");
                        }
                        ObjectType::Skybox => {
                            content.text("Select this object to see");
                            content.text("Skybox Settings panel");
                        }
                        _ => {}
                    }
                } else {
                    content.text("No object selected");
                    content.separator();
                    content.text("Select an object from");
                    content.text("the Scene Hierarchy");
                }
            });

        // Mark scene as dirty if transform changed
        if transform_changed {
            game.mark_scene_dirty();
        }
    }

    /// Build gizmo toolbar
    pub fn build_gizmo_toolbar(ui: &Ui, game: &mut Game) {
        GuiPanelBuilder::new(ui, "Gizmo")
            .size(200.0, 120.0)
            .position(630.0, 520.0)
            .build(|content| {
                content.text("Transform Tools");
                content.separator();

                // Gizmo mode buttons (using regular buttons as radio)
                if ui.button("Translate (1)") {
                    game.gizmo_state.mode = GizmoMode::Translate;
                }
                ui.same_line();
                if game.gizmo_state.mode == GizmoMode::Translate {
                    ui.text("[X]");
                } else {
                    ui.text("[ ]");
                }

                if ui.button("Rotate (2)") {
                    game.gizmo_state.mode = GizmoMode::Rotate;
                }
                ui.same_line();
                if game.gizmo_state.mode == GizmoMode::Rotate {
                    ui.text("[X]");
                } else {
                    ui.text("[ ]");
                }

                if ui.button("Scale (3)") {
                    game.gizmo_state.mode = GizmoMode::Scale;
                }
                ui.same_line();
                if game.gizmo_state.mode == GizmoMode::Scale {
                    ui.text("[X]");
                } else {
                    ui.text("[ ]");
                }

                content.separator();
                content.checkbox("Show Gizmo", &mut game.gizmo_state.enabled);
            });
    }

    /// Render object hover info overlay (only when nothing is selected)
    pub fn render_object_info(ui: &Ui, game: &Game) {
        // Show hover tooltip whenever hovering over an object
        if let Some(hovered_id) = game.object_picker.hovered_object {
            if let Some(obj) = game.scene.get_object(hovered_id) {
                let is_selected = game.scene.selected_object_id() == Some(hovered_id);
                let label = if is_selected {
                    format!("Selected: {}", obj.name)
                } else {
                    format!("Hovering: {}", obj.name)
                };

                ui.window("##hover_overlay")
                    .position([10.0, ui.io().display_size[1] - 80.0], imgui::Condition::Always)
                    .size([250.0, 60.0], imgui::Condition::Always)
                    .no_decoration()
                    .bg_alpha(0.9)
                    .build(|| {
                        if is_selected {
                            ui.text_colored([0.0, 1.0, 0.0, 1.0], &label);
                        } else {
                            ui.text_colored([1.0, 1.0, 0.0, 1.0], &label);
                            ui.text_disabled("Click to select");
                        }
                    });
            }
        }
        // Selected object info is now shown in the Transform panel (top-right)
    }

    /// Render notifications in the lower right corner
    pub fn render_notifications(ui: &Ui, game: &Game) {
        let screen_width = ui.io().display_size[0];
        let screen_height = ui.io().display_size[1];

        for (i, notification) in game.notifications.iter().enumerate() {
            let y_offset = 10.0 + (i as f32 * 70.0);
            let alpha = (notification.time_remaining / 2.0).min(1.0); // Fade out in last 2 seconds

            ui.window(&format!("##notification_{}", i))
                .position([screen_width - 260.0, screen_height - y_offset - 60.0], imgui::Condition::Always)
                .size([250.0, 50.0], imgui::Condition::Always)
                .no_decoration()
                .bg_alpha(0.9 * alpha)
                .build(|| {
                    ui.text_colored([0.2, 1.0, 0.2, alpha], &notification.message);
                });
        }
    }

    /// Build all UI panels
    pub fn build_ui(context: &mut Context, game: &mut Game, viewport_width: f32, viewport_height: f32) {
        let ui = context.frame();

        // Show object hover/selection info overlay
        Self::render_object_info(&ui, game);

        // Show notifications in lower right
        Self::render_notifications(&ui, game);

        // Always show scene hierarchy and transform editor
        Self::build_scene_hierarchy(&ui, game);
        Self::build_transform_editor(&ui, game);

        // Show object-specific panels ONLY when that object is selected
        let selected_type = game.scene.selected_object().map(|obj| obj.object_type.clone());

        match selected_type {
            Some(ObjectType::Skybox) => Self::build_skybox_settings(&ui, game),
            Some(ObjectType::Nebula) => Self::build_nebula_settings(&ui, game),
            Some(ObjectType::Cube) => {
                // Cube has no extra properties beyond transform
                // Transform editor is enough
            }
            None => {
                // Nothing selected - don't show any config panels
            }
            _ => {}
        }
    }

    // Config save/load helper functions

    fn save_skybox_config(game: &mut Game) {
        let mut engine_config = EngineConfig::load_or_default(CONFIG_PATH);
        engine_config.skybox = (&game.skybox_config).into();
        if let Err(e) = engine_config.save(CONFIG_PATH) {
            eprintln!("Failed to save skybox config: {}", e);
            game.add_notification("Failed to save skybox config".to_string(), 3.0);
        } else {
            println!("Skybox config saved to {}", CONFIG_PATH);
            game.config_dirty = false;
            game.add_notification("Skybox config saved".to_string(), 2.0);
        }
    }

    fn load_skybox_config(game: &mut Game) {
        match EngineConfig::load(CONFIG_PATH) {
            Ok(config) => {
                game.skybox_config = config.skybox.into();
                println!("Skybox config loaded from {}", CONFIG_PATH);
                game.config_dirty = false;
                game.add_notification("Skybox config loaded".to_string(), 2.0);
            }
            Err(e) => {
                eprintln!("Failed to load skybox config: {}", e);
                game.add_notification("Failed to load skybox config".to_string(), 3.0);
            }
        }
    }

    fn save_nebula_config(game: &mut Game) {
        let mut engine_config = EngineConfig::load_or_default(CONFIG_PATH);
        engine_config.nebula = (&game.nebula_config).into();
        if let Err(e) = engine_config.save(CONFIG_PATH) {
            eprintln!("Failed to save nebula config: {}", e);
            game.add_notification("Failed to save nebula config".to_string(), 3.0);
        } else {
            println!("Nebula config saved to {}", CONFIG_PATH);
            game.config_dirty = false;
            game.add_notification("Nebula config saved".to_string(), 2.0);
        }
    }

    fn load_nebula_config(game: &mut Game) {
        match EngineConfig::load(CONFIG_PATH) {
            Ok(config) => {
                game.nebula_config = config.nebula.into();
                println!("Nebula config loaded from {}", CONFIG_PATH);
                game.config_dirty = false;
                game.add_notification("Nebula config loaded".to_string(), 2.0);
            }
            Err(e) => {
                eprintln!("Failed to load nebula config: {}", e);
                game.add_notification("Failed to load nebula config".to_string(), 3.0);
            }
        }
    }

    /// Load all configs from file into the game state
    pub fn load_all_configs(game: &mut Game) {
        match EngineConfig::load(CONFIG_PATH) {
            Ok(config) => {
                game.skybox_config = config.skybox.into();
                game.nebula_config = config.nebula.into();
                game.camera = config.camera.into();
                println!("All configs loaded from {}", CONFIG_PATH);
            }
            Err(e) => {
                eprintln!("Failed to load config file: {}, using defaults", e);
            }
        }
    }

    /// Save all current configs to file
    pub fn save_all_configs(game: &Game) {
        let engine_config = EngineConfig {
            nebula: (&game.nebula_config).into(),
            skybox: (&game.skybox_config).into(),
            camera: (&game.camera).into(),
        };

        if let Err(e) = engine_config.save(CONFIG_PATH) {
            eprintln!("Failed to save all configs: {}", e);
        } else {
            println!("All configs saved to {}", CONFIG_PATH);
        }
    }

    /// Save scene to file
    fn save_scene(game: &mut Game) {
        let scene_data = SceneData::from_scene_graph(&game.scene);
        if let Err(e) = scene_data.save(SCENE_PATH) {
            eprintln!("Failed to save scene: {}", e);
            game.add_notification("Failed to save scene".to_string(), 3.0);
        } else {
            println!("Scene saved to {}", SCENE_PATH);
            game.scene_dirty = false;
            game.add_notification("Scene saved".to_string(), 2.0);
        }
    }

    /// Load scene from file
    fn load_scene(game: &mut Game) {
        match SceneData::load(SCENE_PATH) {
            Ok(scene_data) => {
                game.scene = scene_data.to_scene_graph();
                println!("Scene loaded from {}", SCENE_PATH);
                game.scene_dirty = false;
                game.add_notification("Scene loaded".to_string(), 2.0);
            }
            Err(e) => {
                eprintln!("Failed to load scene: {}", e);
                game.add_notification("Failed to load scene".to_string(), 3.0);
            }
        }
    }

    /// Load scene on startup with intelligent merging
    pub fn load_scene_on_startup(game: &mut Game) {
        let scene_data = SceneData::load_and_merge_with_default(SCENE_PATH);
        game.scene = scene_data.to_scene_graph();
        println!("Scene initialized from {}", SCENE_PATH);
    }
}
