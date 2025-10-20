mod gui_builder;

pub use gui_builder::{GuiPanelBuilder, GuiContentBuilder, SkyboxFxBuilder};

use imgui::{Context, Ui};
use crate::game::{Game, SkyboxConfig, SSAOConfig};
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
                content.text("Use transform gizmo to move/rotate nebula");

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

    /// Build directional light settings panel
    pub fn build_directional_light_settings(ui: &Ui, game: &mut Game) {
        GuiPanelBuilder::new(ui, "Directional Light Settings")
            .size(350.0, 300.0)
            .position(270.0, 10.0)
            .build(|content| {
                content.text("Main directional light source");
                content.separator();

                let light = &mut game.directional_light;

                content.header("Light Color & Intensity");

                // Color picker for light color
                let mut color = [light.color.x, light.color.y, light.color.z];
                content.text("Light Color");
                if ui.color_edit3("##light_color", &mut color) {
                    light.color = glam::Vec3::new(color[0], color[1], color[2]);
                }

                // Intensity slider
                content.text("Brightness");
                ui.slider("##light_intensity", 0.0, 3.0, &mut light.intensity);

                content.separator();
                content.header("Shadow/Ambient Color");

                // Shadow color picker
                let mut shadow_color = [light.shadow_color.x, light.shadow_color.y, light.shadow_color.z];
                content.text("Shadow Color");
                if ui.color_edit3("##shadow_color", &mut shadow_color) {
                    light.shadow_color = glam::Vec3::new(shadow_color[0], shadow_color[1], shadow_color[2]);
                }

                content.separator();
                content.header("Direction (via Transform)");
                content.text("Rotate the light object to");
                content.text("change light direction");
            });
    }

    pub fn build_ssao_settings(ui: &Ui, game: &mut Game) {
        // Store original config to detect changes
        let orig_config = game.ssao_config.clone();

        GuiPanelBuilder::new(ui, "SSAO Settings")
            .size(350.0, 300.0)
            .position(270.0, 10.0)
            .build(|content| {
                content.text("Screen-Space Ambient Occlusion");
                content.separator();

                let ssao = &mut game.ssao_config;

                // Enable/Disable toggle
                content.checkbox("Enable SSAO", &mut ssao.enabled);
                content.separator();

                content.header("SSAO Parameters");

                // Radius slider
                content.text("Radius");
                ui.slider("##ssao_radius", 0.1, 3.0, &mut ssao.radius);

                // Bias slider
                content.text("Bias (prevents self-occlusion)");
                ui.slider("##ssao_bias", 0.001, 0.5, &mut ssao.bias);

                // Power slider
                content.text("Power (contrast)");
                ui.slider("##ssao_power", 1.0, 4.0, &mut ssao.power);

                // Kernel size slider (must be integer)
                content.text("Kernel Size (sample count)");
                let mut kernel_f32 = ssao.kernel_size as f32;
                if ui.slider("##ssao_kernel", 8.0, 128.0, &mut kernel_f32) {
                    ssao.kernel_size = kernel_f32 as u32;
                }

                content.separator();
                content.text("Quality vs Performance:");
                content.text("Lower samples = faster");
                content.text("Higher samples = smoother");
            });

        // Detect changes
        if orig_config.enabled != game.ssao_config.enabled
            || orig_config.radius != game.ssao_config.radius
            || orig_config.bias != game.ssao_config.bias
            || orig_config.power != game.ssao_config.power
            || orig_config.kernel_size != game.ssao_config.kernel_size
        {
            game.mark_config_dirty();
        }
    }

    /// Build game mode toolbar (Play/Pause/Edit)
    fn build_game_mode_toolbar(ui: &Ui, game: &mut Game) {
        let is_editing = game.game_manager.is_editing();
        let is_playing = game.game_manager.is_playing();
        let is_paused = game.game_manager.is_paused();

        // Calculate center position (window width / 2 - toolbar width / 2)
        let toolbar_width = 280.0;
        let screen_width = ui.io().display_size[0];
        let center_x = (screen_width - toolbar_width) * 0.5;

        ui.window("Game Mode")
            .position([center_x, 5.0], imgui::Condition::Always)
            .size([toolbar_width, 70.0], imgui::Condition::Always)
            .collapsible(false)
            .title_bar(false)
            .build(|| {
            // Compact horizontal layout
            // Show mode text
            let mode_text = if is_editing {
                "EDIT"
            } else if is_paused {
                "PAUSED"
            } else {
                "PLAYING"
            };

            let mode_color = if is_editing {
                [0.5, 0.8, 1.0, 1.0]
            } else if is_paused {
                [1.0, 1.0, 0.0, 1.0]
            } else {
                [0.0, 1.0, 0.0, 1.0]
            };

            ui.text_colored(mode_color, mode_text);
            ui.same_line();

            // Small separator
            ui.text("|");
            ui.same_line();

            // Play/Stop button
            if is_editing {
                if ui.button("Play") {
                    game.game_manager.start_play_mode(game.time());
                    game.add_notification("Play mode started!".to_string(), 2.0);
                }
            } else {
                if ui.button("Stop") {
                    game.game_manager.stop_play_mode();
                    game.add_notification("Returned to Edit mode".to_string(), 2.0);
                }
            }

            // Pause/Resume button (only in play mode)
            if is_playing {
                ui.same_line();
                if is_paused {
                    if ui.button("Resume") {
                        game.game_manager.toggle_pause();
                    }
                } else {
                    if ui.button("Pause") {
                        game.game_manager.toggle_pause();
                    }
                }
            }
        });
    }

    /// Build pause menu (shown when game is paused)
    fn build_pause_menu(ui: &Ui, game: &mut Game) {
        // Semi-transparent overlay (skip for now - ImGui background is complex)

        // Center pause menu
        ui.window("PAUSED")
            .position(
                [
                    ui.io().display_size[0] * 0.5 - 200.0,
                    ui.io().display_size[1] * 0.5 - 150.0,
                ],
                imgui::Condition::Always,
            )
            .size([400.0, 300.0], imgui::Condition::Always)
            .collapsible(false)
            .build(|| {
            ui.dummy([0.0, 20.0]);

            // Title
            let title = "GAME PAUSED";
            let title_size = ui.calc_text_size(title);
            ui.set_cursor_pos([200.0 - title_size[0] / 2.0, ui.cursor_pos()[1]]);
            ui.text_colored([1.0, 1.0, 0.0, 1.0], title);

            ui.dummy([0.0, 30.0]);

            // Game info
            ui.text(format!("Scenario: {}", game.game_manager.scenario_name));
            ui.text(format!("Turn: {}", game.game_manager.current_turn));

            let elapsed = game.game_manager.get_elapsed_time(game.time());
            let minutes = (elapsed / 60.0) as u32;
            let seconds = (elapsed % 60.0) as u32;
            ui.text(format!("Elapsed Time: {}:{:02}", minutes, seconds));

            ui.dummy([0.0, 30.0]);
            ui.separator();
            ui.dummy([0.0, 10.0]);

            // Buttons
            let button_width = 360.0;
            ui.set_cursor_pos([20.0, ui.cursor_pos()[1]]);
            if ui.button_with_size("Resume", [button_width, 40.0]) {
                game.game_manager.toggle_pause();
            }

            ui.dummy([0.0, 10.0]);
            ui.set_cursor_pos([20.0, ui.cursor_pos()[1]]);
            if ui.button_with_size("Stop (Return to Edit)", [button_width, 40.0]) {
                game.game_manager.stop_play_mode();
                game.add_notification("Returned to Edit mode".to_string(), 2.0);
            }

            ui.dummy([0.0, 10.0]);
            ui.set_cursor_pos([20.0, ui.cursor_pos()[1]]);
            ui.text_colored([0.7, 0.7, 0.7, 1.0], "Press ESC to toggle pause");
        });
    }

    /// Build Game Manager settings panel
    fn build_game_manager_settings(ui: &Ui, game: &mut Game) {
        GuiPanelBuilder::new(ui, "Game Manager Settings")
            .size(400.0, 500.0)
            .position(270.0, 10.0)
            .build(|content| {
                content.text("Configure scenario parameters");

                let manager = &mut game.game_manager;

                content
                    .header("Scenario")
                    .text_input("Scenario Name", &mut manager.scenario_name)
                    .text_input("Description", &mut manager.scenario_description)
                    .header("Game Settings")
                    .slider_u32("Max Turns (0 = unlimited)", &mut manager.max_turns, 0, 100)
                    .slider_f32("Turn Time Limit (0 = none)", &mut manager.turn_time_limit, 0.0, 300.0)
                    .header("Factions")
                    .text_input("Player Faction", &mut manager.player_faction);

                // AI factions (simple display for now)
                ui.text("AI Factions:");
                for (i, faction) in manager.ai_factions.iter().enumerate() {
                    ui.bullet_text(format!("{}: {}", i + 1, faction));
                }

                content.header("Victory Conditions");
                let conditions = &mut manager.victory_conditions;
                ui.checkbox("Eliminate All Enemies", &mut conditions.eliminate_all_enemies);
                content.slider_u32("Survive N Turns", &mut conditions.survive_turns, 0, 50);

                // Mark config as dirty if changed
                game.mark_config_dirty();
            });
    }

    /// Build the scene hierarchy UI
    pub fn build_scene_hierarchy(ui: &Ui, game: &mut Game) {
        let mut save_scene_clicked = false;
        let mut load_scene_clicked = false;
        let mut clicked_obj_id: Option<usize> = None;
        let mut double_clicked_obj_id: Option<usize> = None;
        let mut duplicate_object_id: Option<usize> = None;
        let mut clicked_material: Option<String> = None;

        GuiPanelBuilder::new(ui, "Scene Hierarchy")
            .size(250.0, 550.0)
            .position(10.0, 10.0)
            .build(|content| {
                content.text("Select objects to edit");
                content.text_disabled("Click selected to focus");
                content.separator();

                // Collect objects and categorize them
                let all_objects: Vec<(usize, String, crate::scene::ObjectType)> = game
                    .scene
                    .objects_sorted()
                    .iter()
                    .map(|obj| (obj.id, obj.name.clone(), obj.object_type.clone()))
                    .collect();

                let selected_id = game.scene.selected_object_id();

                // Split into singletons and regular objects
                let singletons: Vec<_> = all_objects.iter()
                    .filter(|(_, _, obj_type)| matches!(obj_type,
                        crate::scene::ObjectType::Skybox |
                        crate::scene::ObjectType::Nebula |
                        crate::scene::ObjectType::DirectionalLight |
                        crate::scene::ObjectType::SSAO))
                    .collect();

                let objects: Vec<_> = all_objects.iter()
                    .filter(|(_, _, obj_type)| !matches!(obj_type,
                        crate::scene::ObjectType::Skybox |
                        crate::scene::ObjectType::Nebula |
                        crate::scene::ObjectType::DirectionalLight |
                        crate::scene::ObjectType::SSAO))
                    .collect();

                // Render Singletons section
                if !singletons.is_empty() {
                    content.header("Singletons");
                    for (id, name, _obj_type) in singletons {
                        let is_selected = selected_id == Some(*id);
                        let label = if is_selected {
                            format!("> {}", name)
                        } else {
                            format!("  {}", name)
                        };

                        if ui.selectable(&label) {
                            if is_selected {
                                double_clicked_obj_id = Some(*id);
                            } else {
                                clicked_obj_id = Some(*id);
                            }
                        }

                        if ui.is_item_hovered() && ui.is_mouse_double_clicked(imgui::MouseButton::Left) {
                            double_clicked_obj_id = Some(*id);
                        }
                    }
                    content.separator();
                }

                // Render Objects section
                if !objects.is_empty() {
                    content.header("Objects");
                    for (id, name, _obj_type) in objects {
                        let is_selected = selected_id == Some(*id);
                        let label = if is_selected {
                            format!("> {}", name)
                        } else {
                            format!("  {}", name)
                        };

                        if ui.selectable(&label) {
                            if is_selected {
                                double_clicked_obj_id = Some(*id);
                            } else {
                                clicked_obj_id = Some(*id);
                            }
                        }

                        if ui.is_item_hovered() && ui.is_mouse_double_clicked(imgui::MouseButton::Left) {
                            double_clicked_obj_id = Some(*id);
                        }
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
                        .map(|obj| !matches!(obj.object_type,
                            crate::scene::ObjectType::Skybox |
                            crate::scene::ObjectType::Nebula |
                            crate::scene::ObjectType::SSAO))
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

                // Materials section
                content.separator();
                content.header("Materials");

                let material_names = game.material_library.material_names();
                for mat_name in &material_names {
                    let label = format!("  {}", mat_name);
                    if ui.selectable(&label) {
                        clicked_material = Some(mat_name.clone());
                    }
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

        // Handle material click - open material editor
        if let Some(mat_name) = clicked_material {
            game.current_material_name = mat_name.clone();
            if let Some(mat) = game.material_library.get(&mat_name) {
                game.material = *mat;
            }
            game.material_editor_open = true;
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
                        ObjectType::SSAO => {
                            content.text("Select this object to see");
                            content.text("SSAO Settings panel");
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

    /// Build material editor panel
    pub fn build_material_editor(ui: &Ui, game: &mut Game) {
        // Material Editor Panel - only show when open
        if !game.material_editor_open {
            return;
        }

        ui.window("Material Editor")
            .position([990.0, 10.0], imgui::Condition::FirstUseEver)
            .size([280.0, 500.0], imgui::Condition::FirstUseEver)
            .opened(&mut game.material_editor_open)
            .build(|| {
                let content = ui;
                content.text("PBR Material Properties");
                content.separator();

                // Material name input
                ui.text("Material Name:");
                let mut name_buf = game.current_material_name.clone();
                if ui.input_text("##material_name", &mut name_buf).build() {
                    game.current_material_name = name_buf;
                }

                content.separator();

                // Material library dropdown
                ui.text("Load from Library:");
                let material_names = game.material_library.material_names();
                let current_idx = material_names.iter().position(|n| n == &game.current_material_name).unwrap_or(0);

                if let Some(_token) = ui.begin_combo("##material_library", &material_names.get(current_idx).map(|s| s.as_str()).unwrap_or("")) {
                    for name in &material_names {
                        let is_selected = name == &game.current_material_name;
                        if ui.selectable_config(name).selected(is_selected).build() {
                            if let Some(mat) = game.material_library.get(name) {
                                game.material = *mat;
                                game.current_material_name = name.clone();
                            }
                        }
                    }
                }

                content.separator();

                // Albedo color
                content.text("Albedo (Base Color)");
                let mut albedo = [game.material.albedo.x, game.material.albedo.y, game.material.albedo.z];
                if ui.color_edit3("##albedo", &mut albedo) {
                    game.material.albedo = glam::Vec3::new(albedo[0], albedo[1], albedo[2]);
                }

                content.separator();

                // Metallic slider
                ui.text("Metallic");
                ui.slider("##metallic", 0.0, 1.0, &mut game.material.metallic);
                ui.same_line();
                ui.text_disabled("(0=plastic, 1=metal)");

                // Roughness slider
                ui.text("Roughness");
                ui.slider("##roughness", 0.0, 1.0, &mut game.material.roughness);
                ui.same_line();
                ui.text_disabled("(0=smooth, 1=rough)");

                // Ambient lighting slider
                ui.text("Ambient Strength");
                ui.slider("##ambient_strength", 0.0, 2.0, &mut game.material.ambient_strength);
                ui.same_line();
                ui.text_disabled("(constant ambient light)");

                // GI strength slider
                ui.text("GI Strength");
                ui.slider("##gi_strength", 0.0, 1.0, &mut game.material.gi_strength);
                ui.same_line();
                ui.text_disabled("(environmental lighting)");

                content.separator();

                // Preset buttons
                content.text("Presets:");
                if ui.button("Plastic") {
                    game.material = crate::material::MaterialProperties::plastic(game.material.albedo);
                }
                ui.same_line();
                if ui.button("Metal") {
                    game.material = crate::material::MaterialProperties::metallic(game.material.albedo, 0.3);
                }
                ui.same_line();
                if ui.button("Matte") {
                    game.material = crate::material::MaterialProperties::matte(game.material.albedo);
                }

                content.separator();

                // Save/Delete buttons
                ui.text("Material Library:");
                if ui.button("Save Material") {
                    game.material_library.set(game.current_material_name.clone(), game.material);
                    if let Err(e) = game.material_library.save("config/materials.json") {
                        eprintln!("Failed to save material library: {}", e);
                    } else {
                        println!("Material '{}' saved to library", game.current_material_name);
                    }
                }

                ui.same_line();

                // Can't delete default materials
                let can_delete = game.current_material_name != "Default"
                    && game.current_material_name != "Metal"
                    && game.current_material_name != "Plastic"
                    && game.current_material_name != "Matte"
                    && game.material_library.contains(&game.current_material_name);

                ui.disabled(!can_delete, || {
                    if ui.button("Delete") {
                        if game.material_library.remove(&game.current_material_name).is_some() {
                            if let Err(e) = game.material_library.save("config/materials.json") {
                                eprintln!("Failed to save material library: {}", e);
                            } else {
                                println!("Material '{}' deleted from library", game.current_material_name);
                            }
                            // Switch to default material after deleting
                            game.current_material_name = "Default".to_string();
                            if let Some(mat) = game.material_library.get("Default") {
                                game.material = *mat;
                            }
                        }
                    }
                });

                content.separator();

                // Apply to selected object
                if let Some(selected_obj) = game.scene.selected_object() {
                    ui.text(format!("Selected: {}", selected_obj.name));
                    if ui.button("Apply to Selected Object") {
                        if let Some(obj) = game.scene.selected_object_mut() {
                            obj.material = Some(game.current_material_name.clone());
                            game.scene_dirty = true;
                            println!("Applied material '{}' to '{}'", game.current_material_name, obj.name);
                        }
                    }
                } else {
                    ui.text_disabled("No object selected");
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

        // Show Play/Pause/Edit mode controls at top
        Self::build_game_mode_toolbar(&ui, game);

        // Show pause menu if in play mode and paused
        if game.game_manager.is_playing() && game.game_manager.is_paused() {
            Self::build_pause_menu(&ui, game);
        }

        // Only show edit UI when in edit mode
        if game.game_manager.is_editing() {
            // Always show scene hierarchy and transform editor in edit mode
            Self::build_scene_hierarchy(&ui, game);
            Self::build_transform_editor(&ui, game);
        }

        // Show edit-mode-only panels
        if game.game_manager.is_editing() {
            // Show material editor if open
            Self::build_material_editor(&ui, game);

            // Show object-specific panels ONLY when that object is selected
            let selected_type = game.scene.selected_object().map(|obj| obj.object_type.clone());

            match selected_type {
                Some(ObjectType::Skybox) => Self::build_skybox_settings(&ui, game),
                Some(ObjectType::Nebula) => Self::build_nebula_settings(&ui, game),
                Some(ObjectType::DirectionalLight) => Self::build_directional_light_settings(&ui, game),
                Some(ObjectType::SSAO) => Self::build_ssao_settings(&ui, game),
                Some(ObjectType::GameManager) => Self::build_game_manager_settings(&ui, game),
                Some(ObjectType::Cube) | Some(ObjectType::Mesh(_)) => {
                    // Mesh/Cube objects can use materials but have no extra settings panel
                    // Material editor is accessed via Materials section in hierarchy
                }
                None => {
                    // Nothing selected - don't show any config panels
                }
                _ => {}
            }
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

    fn save_ssao_config(game: &mut Game) {
        let mut engine_config = EngineConfig::load_or_default(CONFIG_PATH);
        engine_config.ssao = (&game.ssao_config).into();
        if let Err(e) = engine_config.save(CONFIG_PATH) {
            eprintln!("Failed to save SSAO config: {}", e);
            game.add_notification("Failed to save SSAO config".to_string(), 3.0);
        } else {
            println!("SSAO config saved to {}", CONFIG_PATH);
            game.config_dirty = false;
            game.add_notification("SSAO config saved".to_string(), 2.0);
        }
    }

    fn load_ssao_config(game: &mut Game) {
        match EngineConfig::load(CONFIG_PATH) {
            Ok(config) => {
                game.ssao_config = config.ssao.into();
                println!("SSAO config loaded from {}", CONFIG_PATH);
                game.config_dirty = false;
                game.add_notification("SSAO config loaded".to_string(), 2.0);
            }
            Err(e) => {
                eprintln!("Failed to load SSAO config: {}", e);
                game.add_notification("Failed to load SSAO config".to_string(), 3.0);
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
                game.sync_nebula_transform(); // Sync position/rotation to ECS
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
                game.ssao_config = config.ssao.into();
                println!("All configs loaded from {}", CONFIG_PATH);
            }
            Err(e) => {
                eprintln!("Failed to load config file: {}, using defaults", e);
            }
        }

        // Load material library
        game.material_library = crate::material_library::MaterialLibrary::load_or_default("config/materials.json");
        println!("Material library loaded");
    }

    /// Save all current configs to file
    pub fn save_all_configs(game: &Game) {
        let engine_config = EngineConfig {
            nebula: (&game.nebula_config).into(),
            skybox: (&game.skybox_config).into(),
            camera: (&game.camera).into(),
            ssao: (&game.ssao_config).into(),
        };

        if let Err(e) = engine_config.save(CONFIG_PATH) {
            eprintln!("Failed to save all configs: {}", e);
        } else {
            println!("All configs saved to {}", CONFIG_PATH);
        }
    }

    /// Save EVERYTHING (scene + all configs) to files
    fn save_scene(game: &mut Game) {
        // Save scene (object transforms and hierarchy)
        let scene_data = SceneData::from_scene_graph(&game.scene);
        let scene_result = scene_data.save(SCENE_PATH);

        // Save all configs (skybox, nebula, camera, SSAO)
        let engine_config = EngineConfig {
            nebula: (&game.nebula_config).into(),
            skybox: (&game.skybox_config).into(),
            camera: (&game.camera).into(),
            ssao: (&game.ssao_config).into(),
        };
        let config_result = engine_config.save(CONFIG_PATH);

        // Report results
        if scene_result.is_err() || config_result.is_err() {
            if let Err(e) = scene_result {
                eprintln!("Failed to save scene: {}", e);
            }
            if let Err(e) = config_result {
                eprintln!("Failed to save configs: {}", e);
            }
            game.add_notification("Failed to save".to_string(), 3.0);
        } else {
            println!("Scene and configs saved");
            game.scene_dirty = false;
            game.config_dirty = false;
            game.add_notification("Everything saved!".to_string(), 2.0);
        }
    }

    /// Load EVERYTHING (scene + all configs) from files
    fn load_scene(game: &mut Game) {
        let mut success = true;

        // Load scene
        match SceneData::load(SCENE_PATH) {
            Ok(scene_data) => {
                game.scene = scene_data.to_scene_graph();
                game.sync_nebula_transform(); // Sync nebula transform to ECS
                println!("Scene loaded from {}", SCENE_PATH);
            }
            Err(e) => {
                eprintln!("Failed to load scene: {}", e);
                success = false;
            }
        }

        // Load all configs
        match EngineConfig::load(CONFIG_PATH) {
            Ok(config) => {
                game.skybox_config = config.skybox.into();
                game.nebula_config = config.nebula.into();
                game.camera = config.camera.into();
                game.ssao_config = config.ssao.into();
                println!("All configs loaded from {}", CONFIG_PATH);
            }
            Err(e) => {
                eprintln!("Failed to load configs: {}", e);
                success = false;
            }
        }

        if success {
            game.scene_dirty = false;
            game.config_dirty = false;
            game.add_notification("Everything loaded!".to_string(), 2.0);
        } else {
            game.add_notification("Failed to load".to_string(), 3.0);
        }
    }

    /// Load scene on startup with intelligent merging
    pub fn load_scene_on_startup(game: &mut Game) {
        let scene_data = SceneData::load_and_merge_with_default(SCENE_PATH);
        game.scene = scene_data.to_scene_graph();

        // Ensure SSAO singleton always exists (add if missing)
        if game.scene.find_by_type(crate::scene::ObjectType::SSAO).is_none() {
            game.scene.add_object("SSAO".to_string(), crate::scene::ObjectType::SSAO);
        }

        // Sync nebula transform from loaded scene to ECS
        game.sync_nebula_transform();

        println!("Scene initialized from {}", SCENE_PATH);
    }
}
