use glam::Vec3;
use serde::{Deserialize, Serialize};

/// Material properties for PBR rendering
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MaterialProperties {
    /// Base color (albedo) of the material
    pub albedo: Vec3,
    /// Metallic factor (0.0 = dielectric, 1.0 = metal)
    pub metallic: f32,
    /// Roughness factor (0.0 = smooth/glossy, 1.0 = rough/matte)
    pub roughness: f32,
    /// Ambient occlusion intensity (0.0 = no AO, 1.0 = full AO)
    pub ao_intensity: f32,
}

impl Default for MaterialProperties {
    fn default() -> Self {
        Self {
            albedo: Vec3::new(0.8, 0.8, 0.8),
            metallic: 0.2,
            roughness: 0.6,
            ao_intensity: 1.0,
        }
    }
}

impl MaterialProperties {
    pub fn new(albedo: Vec3, metallic: f32, roughness: f32, ao_intensity: f32) -> Self {
        Self {
            albedo,
            metallic,
            roughness,
            ao_intensity,
        }
    }

    /// Create a matte (non-metallic, rough) material
    pub fn matte(color: Vec3) -> Self {
        Self {
            albedo: color,
            metallic: 0.0,
            roughness: 0.9,
            ao_intensity: 1.0,
        }
    }

    /// Create a metallic material
    pub fn metallic(color: Vec3, roughness: f32) -> Self {
        Self {
            albedo: color,
            metallic: 1.0,
            roughness,
            ao_intensity: 1.0,
        }
    }

    /// Create a plastic-like material
    pub fn plastic(color: Vec3) -> Self {
        Self {
            albedo: color,
            metallic: 0.0,
            roughness: 0.3,
            ao_intensity: 1.0,
        }
    }
}
