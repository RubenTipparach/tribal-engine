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

        if save_clicked {
            Self::save_skybox_config(&game.skybox_config);
        }
        if load_clicked {
            Self::load_skybox_config(game);
        }
        if reset_clicked {
            game.skybox_config = SkyboxConfig::default();
        }
    }

    /// Build the nebula settings UI
    pub fn build_nebula_settings(ui: &Ui, game: &mut Game) {
        let mut save_clicked = false;
        let mut load_clicked = false;
        let mut reset_clicked = false;

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

        if save_clicked {
            Self::save_nebula_config(&game.nebula_config);
        }
        if load_clicked {
            Self::load_nebula_config(game);
        }
        if reset_clicked {
            game.nebula_config = NebulaConfig::default();
        }
    }

    /// Build the scene hierarchy UI
    pub fn build_scene_hierarchy(ui: &Ui, game: &mut Game) {
        let mut save_scene_clicked = false;
        let mut load_scene_clicked = false;
        let mut clicked_obj_id: Option<usize> = None;

        GuiPanelBuilder::new(ui, "Scene Hierarchy")
            .size(250.0, 400.0)
            .position(10.0, 10.0)
            .build(|content| {
                content.text("Select objects to edit");
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
                        clicked_obj_id = Some(id);
                    }
                }

                // Gizmo controls integrated here
                content.separator();
                content.header("Transform Tools");

                if ui.button("Translate (W)") {
                    game.gizmo_state.mode = GizmoMode::Translate;
                }
                ui.same_line();
                if game.gizmo_state.mode == GizmoMode::Translate {
                    ui.text("[X]");
                } else {
                    ui.text("[ ]");
                }

                if ui.button("Rotate (E)") {
                    game.gizmo_state.mode = GizmoMode::Rotate;
                }
                ui.same_line();
                if game.gizmo_state.mode == GizmoMode::Rotate {
                    ui.text("[X]");
                } else {
                    ui.text("[ ]");
                }

                if ui.button("Scale (R)") {
                    game.gizmo_state.mode = GizmoMode::Scale;
                }
                ui.same_line();
                if game.gizmo_state.mode == GizmoMode::Scale {
                    ui.text("[X]");
                } else {
                    ui.text("[ ]");
                }

                content.checkbox("Show Gizmo", &mut game.gizmo_state.enabled);

                content.separator();
                let (s, l, _) = content.config_buttons();
                save_scene_clicked = s;
                load_scene_clicked = l;
            });

        if let Some(id) = clicked_obj_id {
            game.scene.select_object(id);
        }

        if save_scene_clicked {
            Self::save_scene(&game);
        }
        if load_scene_clicked {
            Self::load_scene(game);
        }
    }

    /// Build the transform editor UI for selected object (top-right corner)
    pub fn build_transform_editor(ui: &Ui, game: &mut Game) {
        let window_width = ui.io().display_size[0];
        let panel_width = 350.0;

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
                if ui.button("Translate (W)") {
                    game.gizmo_state.mode = GizmoMode::Translate;
                }
                ui.same_line();
                if game.gizmo_state.mode == GizmoMode::Translate {
                    ui.text("[X]");
                } else {
                    ui.text("[ ]");
                }

                if ui.button("Rotate (E)") {
                    game.gizmo_state.mode = GizmoMode::Rotate;
                }
                ui.same_line();
                if game.gizmo_state.mode == GizmoMode::Rotate {
                    ui.text("[X]");
                } else {
                    ui.text("[ ]");
                }

                if ui.button("Scale (R)") {
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
        // Only show hover tooltip if no object is currently selected
        if game.scene.selected_object().is_none() {
            if let Some(hovered_id) = game.object_picker.hovered_object {
                if let Some(obj) = game.scene.get_object(hovered_id) {
                    ui.window("##hover_overlay")
                        .position([10.0, ui.io().display_size[1] - 80.0], imgui::Condition::Always)
                        .size([250.0, 60.0], imgui::Condition::Always)
                        .no_decoration()
                        .bg_alpha(0.9)
                        .build(|| {
                            ui.text_colored([1.0, 1.0, 0.0, 1.0], "Hovering:");
                            ui.same_line();
                            ui.text(&obj.name);
                            ui.text_disabled("Click to select");
                        });
                }
            }
        }
        // Selected object info is now shown in the Transform panel (top-right)
    }

    /// Build all UI panels
    pub fn build_ui(context: &mut Context, game: &mut Game, viewport_width: f32, viewport_height: f32) {
        let ui = context.frame();

        // Show object hover/selection info overlay
        Self::render_object_info(&ui, game);

        // Always show scene hierarchy and transform editor
        Self::build_scene_hierarchy(&ui, game);
        Self::build_transform_editor(&ui, game);

        // Show object-specific panels ONLY when that object is selected
        let selected_type = game.scene.selected_object().map(|obj| obj.object_type);

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

    fn save_skybox_config(config: &SkyboxConfig) {
        let mut engine_config = EngineConfig::load_or_default(CONFIG_PATH);
        engine_config.skybox = config.into();
        if let Err(e) = engine_config.save(CONFIG_PATH) {
            eprintln!("Failed to save skybox config: {}", e);
        } else {
            println!("Skybox config saved to {}", CONFIG_PATH);
        }
    }

    fn load_skybox_config(game: &mut Game) {
        match EngineConfig::load(CONFIG_PATH) {
            Ok(config) => {
                game.skybox_config = config.skybox.into();
                println!("Skybox config loaded from {}", CONFIG_PATH);
            }
            Err(e) => eprintln!("Failed to load skybox config: {}", e),
        }
    }

    fn save_nebula_config(config: &NebulaConfig) {
        let mut engine_config = EngineConfig::load_or_default(CONFIG_PATH);
        engine_config.nebula = config.into();
        if let Err(e) = engine_config.save(CONFIG_PATH) {
            eprintln!("Failed to save nebula config: {}", e);
        } else {
            println!("Nebula config saved to {}", CONFIG_PATH);
        }
    }

    fn load_nebula_config(game: &mut Game) {
        match EngineConfig::load(CONFIG_PATH) {
            Ok(config) => {
                game.nebula_config = config.nebula.into();
                println!("Nebula config loaded from {}", CONFIG_PATH);
            }
            Err(e) => eprintln!("Failed to load nebula config: {}", e),
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
    fn save_scene(game: &Game) {
        let scene_data = SceneData::from_scene_graph(&game.scene);
        if let Err(e) = scene_data.save(SCENE_PATH) {
            eprintln!("Failed to save scene: {}", e);
        } else {
            println!("Scene saved to {}", SCENE_PATH);
        }
    }

    /// Load scene from file
    fn load_scene(game: &mut Game) {
        match SceneData::load(SCENE_PATH) {
            Ok(scene_data) => {
                game.scene = scene_data.to_scene_graph();
                println!("Scene loaded from {}", SCENE_PATH);
            }
            Err(e) => eprintln!("Failed to load scene: {}", e),
        }
    }

    /// Load scene on startup
    pub fn load_scene_on_startup(game: &mut Game) {
        let scene_data = SceneData::load_or_default(SCENE_PATH);
        game.scene = scene_data.to_scene_graph();
        println!("Scene loaded from {}", SCENE_PATH);
    }
}
