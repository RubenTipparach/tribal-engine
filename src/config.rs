use anyhow::Result;
use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Complete engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    pub nebula: NebulaConfigData,
    pub skybox: SkyboxConfigData,
    pub camera: CameraConfigData,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            nebula: NebulaConfigData::default(),
            skybox: SkyboxConfigData::default(),
            camera: CameraConfigData::default(),
        }
    }
}

impl EngineConfig {
    /// Load configuration from JSON file
    pub fn load(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: EngineConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to JSON file with pretty formatting
    pub fn save(&self, path: &str) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = Path::new(path).parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Load or create default configuration
    pub fn load_or_default(path: &str) -> Self {
        Self::load(path).unwrap_or_else(|_| {
            let config = Self::default();
            // Try to save the default config
            let _ = config.save(path);
            config
        })
    }
}

/// Nebula configuration (serializable)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NebulaConfigData {
    pub zoom: f32,
    pub density: f32,
    pub brightness: f32,
    pub scale: f32,

    #[serde(with = "vec3_serde")]
    pub color_center: Vec3,

    #[serde(with = "vec3_serde")]
    pub color_edge: Vec3,

    #[serde(with = "vec3_serde")]
    pub color_density_low: Vec3,

    #[serde(with = "vec3_serde")]
    pub color_density_high: Vec3,

    #[serde(with = "vec3_serde")]
    pub light_color: Vec3,

    pub light_intensity: f32,
    pub max_distance: f32,
}

impl Default for NebulaConfigData {
    fn default() -> Self {
        Self {
            zoom: 0.01,
            density: 2.0,
            brightness: 1.0,
            scale: 20.0,
            color_center: Vec3::new(0.8, 1.0, 1.0) * 7.0,
            color_edge: Vec3::new(0.48, 0.53, 0.5) * 1.5,
            color_density_low: Vec3::new(1.0, 0.9, 0.8),
            color_density_high: Vec3::new(0.4, 0.15, 0.1),
            light_color: Vec3::new(1.0, 0.5, 0.25),
            light_intensity: 1.0 / 30.0,
            max_distance: 10.0,
        }
    }
}

/// Skybox configuration (serializable)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkyboxConfigData {
    pub star_density: f32,
    pub star_brightness: f32,

    #[serde(with = "vec3_serde")]
    pub nebula_primary_color: Vec3,

    #[serde(with = "vec3_serde")]
    pub nebula_secondary_color: Vec3,

    pub nebula_intensity: f32,
    pub background_brightness: f32,
}

impl Default for SkyboxConfigData {
    fn default() -> Self {
        Self {
            star_density: 2.0,
            star_brightness: 3.0,
            nebula_primary_color: Vec3::new(0.1, 0.2, 0.4),
            nebula_secondary_color: Vec3::new(0.6, 0.3, 0.8),
            nebula_intensity: 1.0,
            background_brightness: 0.0,
        }
    }
}

/// Camera configuration (serializable)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraConfigData {
    #[serde(with = "vec3_serde")]
    pub position: Vec3,

    pub pitch: f32,
    pub yaw: f32,
    pub roll: f32,
    pub move_speed: f32,
    pub mouse_sensitivity: f32,
    pub fov: f32,
}

impl Default for CameraConfigData {
    fn default() -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 5.0),
            pitch: 0.0,
            yaw: 0.0,
            roll: 0.0,
            move_speed: 5.0,
            mouse_sensitivity: 0.003,
            fov: 70.0,
        }
    }
}

/// Custom serialization for Vec3
mod vec3_serde {
    use glam::Vec3;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Serialize, Deserialize)]
    struct Vec3Data {
        x: f32,
        y: f32,
        z: f32,
    }

    pub fn serialize<S>(vec: &Vec3, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Vec3Data {
            x: vec.x,
            y: vec.y,
            z: vec.z,
        }
        .serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec3, D::Error>
    where
        D: Deserializer<'de>,
    {
        let data = Vec3Data::deserialize(deserializer)?;
        Ok(Vec3::new(data.x, data.y, data.z))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = EngineConfig::default();
        assert_eq!(config.nebula.zoom, 0.01);
        assert_eq!(config.skybox.star_density, 2.0);
    }

    #[test]
    fn test_save_load() {
        let config = EngineConfig::default();
        let path = "test_config.json";

        config.save(path).unwrap();
        let loaded = EngineConfig::load(path).unwrap();

        assert_eq!(loaded.nebula.zoom, config.nebula.zoom);

        // Cleanup
        let _ = fs::remove_file(path);
    }
}
