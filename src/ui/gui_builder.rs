use imgui::Ui;
use glam::Vec3;

/// Builder for creating GUI panels with common controls
pub struct GuiPanelBuilder<'a> {
    ui: &'a Ui,
    title: &'a str,
    size: [f32; 2],
    position: [f32; 2],
}

impl<'a> GuiPanelBuilder<'a> {
    pub fn new(ui: &'a Ui, title: &'a str) -> Self {
        Self {
            ui,
            title,
            size: [350.0, 400.0],
            position: [10.0, 10.0],
        }
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.size = [width, height];
        self
    }

    pub fn position(mut self, x: f32, y: f32) -> Self {
        self.position = [x, y];
        self
    }

    pub fn build<F>(self, f: F)
    where
        F: FnOnce(&mut GuiContentBuilder),
    {
        self.ui
            .window(self.title)
            .size(self.size, imgui::Condition::FirstUseEver)
            .position(self.position, imgui::Condition::FirstUseEver)
            .build(|| {
                let mut content = GuiContentBuilder::new(self.ui);
                f(&mut content);
            });
    }
}

/// Builder for adding controls to a GUI panel
pub struct GuiContentBuilder<'a> {
    ui: &'a Ui,
}

impl<'a> GuiContentBuilder<'a> {
    fn new(ui: &'a Ui) -> Self {
        Self { ui }
    }

    pub fn text(&mut self, text: &str) -> &mut Self {
        self.ui.text(text);
        self
    }

    pub fn separator(&mut self) -> &mut Self {
        self.ui.separator();
        self
    }

    pub fn header(&mut self, text: &str) -> &mut Self {
        self.ui.separator();
        self.ui.text(text);
        self
    }

    pub fn slider_f32(
        &mut self,
        label: &str,
        value: &mut f32,
        min: f32,
        max: f32,
    ) -> &mut Self {
        self.ui.slider(label, min, max, value);
        self
    }

    pub fn slider_i32(
        &mut self,
        label: &str,
        value: &mut i32,
        min: i32,
        max: i32,
    ) -> &mut Self {
        self.ui.slider(label, min, max, value);
        self
    }

    pub fn color_picker(&mut self, label: &str, color: &mut Vec3) -> &mut Self {
        let mut color_array = [color.x, color.y, color.z];
        if self.ui.color_edit3(label, &mut color_array) {
            *color = Vec3::new(color_array[0], color_array[1], color_array[2]);
        }
        self
    }

    pub fn button<F>(&mut self, label: &str, on_click: F) -> &mut Self
    where
        F: FnOnce(),
    {
        if self.ui.button(label) {
            on_click();
        }
        self
    }

    pub fn checkbox(&mut self, label: &str, value: &mut bool) -> &mut Self {
        self.ui.checkbox(label, value);
        self
    }

    pub fn spacing(&mut self) -> &mut Self {
        self.ui.spacing();
        self
    }
}

/// Specialized builder for skybox FX controls
pub struct SkyboxFxBuilder<'a> {
    pub content: &'a mut GuiContentBuilder<'a>,
}

impl<'a> SkyboxFxBuilder<'a> {
    pub fn new(content: &'a mut GuiContentBuilder<'a>) -> Self {
        Self { content }
    }

    pub fn star_controls(
        &mut self,
        density: &mut f32,
        brightness: &mut f32,
    ) -> &mut Self {
        self.content
            .header("Stars")
            .slider_f32("Star Density", density, 0.1, 10.0)
            .slider_f32("Star Brightness", brightness, 0.0, 10.0);
        self
    }

    pub fn nebula_controls(
        &mut self,
        intensity: &mut f32,
        primary_color: &mut Vec3,
        secondary_color: &mut Vec3,
    ) -> &mut Self {
        self.content
            .header("Nebula")
            .slider_f32("Nebula Intensity", intensity, 0.0, 2.0)
            .color_picker("Primary Color", primary_color)
            .color_picker("Secondary Color", secondary_color);
        self
    }

    pub fn background_controls(&mut self, brightness: &mut f32) -> &mut Self {
        self.content
            .header("Background")
            .slider_f32("Brightness", brightness, 0.0, 0.5);
        self
    }
}
