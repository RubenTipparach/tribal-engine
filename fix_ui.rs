    /// Render object hover/selection info
    pub fn render_object_info(ui: &Ui, game: &Game) {
        // Show hovered object
        if let Some(hovered_id) = game.object_picker.hovered_object {
            if let Some(obj) = game.scene.get_object(hovered_id) {
                ui.window("Hover Info")
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

        // Show selected object
        if let Some(selected) = game.scene.selected_object() {
            ui.window("Selected")
                .position([270.0, ui.io().display_size[1] - 80.0], imgui::Condition::Always)
                .size([200.0, 60.0], imgui::Condition::Always)
                .no_decoration()
                .bg_alpha(0.9)
                .build(|| {
                    ui.text_colored([0.2, 1.0, 0.2, 1.0], "Selected:");
                    ui.same_line();
                    ui.text(&selected.name);
                });
        }
    }

    /// Build all UI panels
    pub fn build_ui(context: &mut Context, game: &mut Game, viewport_width: f32, viewport_height: f32) {
        let ui = context.frame();

        // Show object hover/selection info
        Self::render_object_info(&ui, game);

        // Always show scene hierarchy, transform editor, and gizmo toolbar
        Self::build_scene_hierarchy(&ui, game);
        Self::build_transform_editor(&ui, game);
        Self::build_gizmo_toolbar(&ui, game);

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
