mod gui_builder;

pub use gui_builder::{GuiPanelBuilder, GuiContentBuilder, SkyboxFxBuilder};

use imgui::{Context, Ui};
use crate::game::{Game, SkyboxConfig};
use crate::nebula::NebulaConfig;

/// Manages all UI rendering and interactions
pub struct UiManager;

impl UiManager {
    pub fn new() -> Self {
        Self
    }

    /// Build the skybox settings UI
    pub fn build_skybox_settings(ui: &Ui, game: &mut Game) {
        GuiPanelBuilder::new(ui, "Skybox Settings")
            .size(350.0, 450.0)
            .position(10.0, 10.0)
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
                    .slider_f32("Brightness", &mut config.background_brightness, 0.0, 0.5)
                    .separator()
                    .button("Reset to Default", || {
                        game.skybox_config = SkyboxConfig::default();
                    });
            });
    }

    /// Build the nebula settings UI
    pub fn build_nebula_settings(ui: &Ui, game: &mut Game) {
        GuiPanelBuilder::new(ui, "Nebula Settings")
            .size(380.0, 400.0)
            .position(370.0, 10.0)
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

                    .separator()
                    .button("Reset to Default", || {
                        game.nebula_config = NebulaConfig::default();
                    });
            });
    }

    /// Build the visibility settings UI
    pub fn build_visibility_settings(ui: &Ui, game: &mut Game) {
        GuiPanelBuilder::new(ui, "Visibility")
            .size(200.0, 100.0)
            .position(760.0, 10.0)
            .build(|content| {
                content
                    .text("Toggle objects")
                    .separator()
                    .checkbox("Show Cube", &mut game.show_cube);
            });
    }

    /// Build all UI panels
    pub fn build_ui(context: &mut Context, game: &mut Game) {
        let ui = context.frame();
        Self::build_skybox_settings(&ui, game);
        Self::build_nebula_settings(&ui, game);
        Self::build_visibility_settings(&ui, game);
    }
}
