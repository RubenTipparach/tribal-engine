use glam::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct DirectionalLight {
    pub direction: Vec3,
    pub color: Vec3,
    pub intensity: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct PointLight {
    pub position: Vec3,
    pub color: Vec3,
    pub intensity: f32,
}

pub struct LightingData {
    pub directional_lights: Vec<DirectionalLight>,
    pub point_lights: Vec<PointLight>,
}

impl LightingData {
    pub fn new() -> Self {
        Self {
            directional_lights: Vec::new(),
            point_lights: Vec::new(),
        }
    }

    pub fn add_directional_light(&mut self, light: DirectionalLight) {
        self.directional_lights.push(light);
    }

    pub fn add_point_light(&mut self, light: PointLight) {
        self.point_lights.push(light);
    }
}
