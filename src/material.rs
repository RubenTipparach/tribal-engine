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
    /// Ambient lighting intensity (0.0 = no ambient, 2.0 = bright ambient)
    pub ambient_strength: f32,
    /// Global illumination strength (0.0 = no GI, 1.0 = full GI)
    pub gi_strength: f32,
}

impl Default for MaterialProperties {
    fn default() -> Self {
        Self {
            albedo: Vec3::new(0.8, 0.8, 0.8),
            metallic: 0.2,
            roughness: 0.6,
            ambient_strength: 1.0,
            gi_strength: 0.5,
        }
    }
}

impl MaterialProperties {
    pub fn new(albedo: Vec3, metallic: f32, roughness: f32, ambient_strength: f32) -> Self {
        Self {
            albedo,
            metallic,
            roughness,
            ambient_strength,
            gi_strength: 0.5,
        }
    }

    /// Create a matte (non-metallic, rough) material
    pub fn matte(color: Vec3) -> Self {
        Self {
            albedo: color,
            metallic: 0.0,
            roughness: 0.9,
            ambient_strength: 1.0,
            gi_strength: 0.5,
        }
    }

    /// Create a metallic material
    pub fn metallic(color: Vec3, roughness: f32) -> Self {
        Self {
            albedo: color,
            metallic: 1.0,
            roughness,
            ambient_strength: 1.0,
            gi_strength: 0.3,
        }
    }

    /// Create a plastic-like material
    pub fn plastic(color: Vec3) -> Self {
        Self {
            albedo: color,
            metallic: 0.0,
            roughness: 0.3,
            ambient_strength: 1.0,
            gi_strength: 0.5,
        }
    }
}
