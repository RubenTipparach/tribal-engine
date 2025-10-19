use crate::material::MaterialProperties;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Material library containing named materials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialLibrary {
    pub materials: HashMap<String, MaterialProperties>,
}

impl Default for MaterialLibrary {
    fn default() -> Self {
        let mut materials = HashMap::new();

        // Add some default materials
        materials.insert(
            "Default".to_string(),
            MaterialProperties::default(),
        );
        materials.insert(
            "Metal".to_string(),
            MaterialProperties::metallic(glam::Vec3::new(0.8, 0.8, 0.8), 0.3),
        );
        materials.insert(
            "Plastic".to_string(),
            MaterialProperties::plastic(glam::Vec3::new(0.8, 0.8, 0.8)),
        );
        materials.insert(
            "Matte".to_string(),
            MaterialProperties::matte(glam::Vec3::new(0.8, 0.8, 0.8)),
        );

        Self { materials }
    }
}

impl MaterialLibrary {
    pub fn new() -> Self {
        Self::default()
    }

    /// Load material library from JSON file
    pub fn load(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let library: MaterialLibrary = serde_json::from_str(&content)?;
        Ok(library)
    }

    /// Save material library to JSON file
    pub fn save(&self, path: &str) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = Path::new(path).parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Load or create default material library
    pub fn load_or_default(path: &str) -> Self {
        Self::load(path).unwrap_or_else(|_| {
            let library = Self::default();
            // Try to save the default library
            let _ = library.save(path);
            library
        })
    }

    /// Get a material by name
    pub fn get(&self, name: &str) -> Option<&MaterialProperties> {
        self.materials.get(name)
    }

    /// Add or update a material
    pub fn set(&mut self, name: String, material: MaterialProperties) {
        self.materials.insert(name, material);
    }

    /// Remove a material by name
    pub fn remove(&mut self, name: &str) -> Option<MaterialProperties> {
        // Don't allow removing the default material
        if name == "Default" {
            return None;
        }
        self.materials.remove(name)
    }

    /// Get all material names
    pub fn material_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.materials.keys().cloned().collect();
        names.sort();
        names
    }

    /// Check if a material exists
    pub fn contains(&self, name: &str) -> bool {
        self.materials.contains_key(name)
    }
}
