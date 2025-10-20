use glam::{Mat4, Quat, Vec3};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for scene objects
pub type ObjectId = usize;

/// Transform component for positioning objects in 3D space
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {
    pub fn new(position: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }

    pub fn identity() -> Self {
        Self {
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }

    pub fn from_position(position: Vec3) -> Self {
        Self {
            position,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }

    /// Get the model matrix for this transform
    pub fn model_matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.position)
    }

    /// Set rotation from Euler angles (pitch, yaw, roll in radians)
    pub fn set_euler_rotation(&mut self, pitch: f32, yaw: f32, roll: f32) {
        self.rotation = Quat::from_euler(glam::EulerRot::YXZ, yaw, pitch, roll);
    }

    /// Get Euler angles from quaternion rotation (pitch, yaw, roll)
    pub fn euler_angles(&self) -> (f32, f32, f32) {
        let (yaw, pitch, roll) = self.rotation.to_euler(glam::EulerRot::YXZ);
        (pitch, yaw, roll)
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}

/// Types of objects in the scene
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectType {
    Cube,
    Nebula,
    Skybox,
    DirectionalLight,
    SSAO, // SSAO settings singleton
    Mesh(String), // Custom mesh with path to .obj file
    Custom(u32), // For future custom mesh support
}

/// Scene object with transform and type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneObject {
    pub id: ObjectId,
    pub name: String,
    pub object_type: ObjectType,
    pub transform: Transform,
    pub visible: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub material: Option<String>, // Name of material from material library
}

impl SceneObject {
    pub fn new(id: ObjectId, name: String, object_type: ObjectType) -> Self {
        Self {
            id,
            name,
            object_type,
            transform: Transform::default(),
            visible: true,
            material: None,
        }
    }

    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
    }

    /// Get the bounding box size for this object (before scaling)
    pub fn base_bounding_box_size(&self) -> f32 {
        match self.object_type {
            ObjectType::Cube => 2.0, // Cube is 2x2x2
            ObjectType::Nebula => 10.0, // Nebula is larger
            ObjectType::Skybox => 50.0, // Skybox is very large
            ObjectType::DirectionalLight => 1.5, // Light visualization arrow
            ObjectType::SSAO => 0.0, // SSAO is a settings singleton, no visual representation
            ObjectType::Mesh(_) => 5.0, // Default size for mesh objects
            ObjectType::Custom(_) => 2.0, // Default for custom objects
        }
    }

    /// Get the actual bounding box size accounting for scale
    pub fn bounding_box_size(&self) -> f32 {
        let base_size = self.base_bounding_box_size();
        let max_scale = self.transform.scale.x.max(self.transform.scale.y).max(self.transform.scale.z);
        base_size * max_scale
    }
}

/// Scene graph managing all objects in the scene
pub struct SceneGraph {
    objects: HashMap<ObjectId, SceneObject>,
    next_id: ObjectId,
    selected_object: Option<ObjectId>,
}

impl SceneGraph {
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
            next_id: 0,
            selected_object: None,
        }
    }

    /// Add an object to the scene
    pub fn add_object(&mut self, name: String, object_type: ObjectType) -> ObjectId {
        let id = self.next_id;
        self.next_id += 1;

        let object = SceneObject::new(id, name, object_type);
        self.objects.insert(id, object);
        id
    }

    /// Add an object with a specific transform
    pub fn add_object_with_transform(
        &mut self,
        name: String,
        object_type: ObjectType,
        transform: Transform,
    ) -> ObjectId {
        let id = self.add_object(name, object_type);
        if let Some(obj) = self.objects.get_mut(&id) {
            obj.transform = transform;
        }
        id
    }

    /// Duplicate an object (returns new object ID if successful)
    pub fn duplicate_object(&mut self, id: ObjectId) -> Option<ObjectId> {
        // Get the object to duplicate
        let obj = self.objects.get(&id)?;

        // Don't allow duplicating skybox or nebula
        if matches!(obj.object_type, ObjectType::Skybox | ObjectType::Nebula) {
            return None;
        }

        // Clone the object data
        let object_type = obj.object_type.clone();
        let transform = obj.transform;
        let visible = obj.visible;

        // Create a new name with " Copy" suffix
        let new_name = format!("{} Copy", obj.name);

        // Create the new object
        let new_id = self.next_id;
        self.next_id += 1;

        let mut new_object = SceneObject::new(new_id, new_name, object_type);
        new_object.transform = transform;
        new_object.visible = visible;

        // Offset the position slightly so it's visible
        new_object.transform.position += glam::Vec3::new(0.5, 0.5, 0.5);

        self.objects.insert(new_id, new_object);
        Some(new_id)
    }

    /// Remove an object from the scene
    pub fn remove_object(&mut self, id: ObjectId) -> Option<SceneObject> {
        if self.selected_object == Some(id) {
            self.selected_object = None;
        }
        self.objects.remove(&id)
    }

    /// Get a reference to an object
    pub fn get_object(&self, id: ObjectId) -> Option<&SceneObject> {
        self.objects.get(&id)
    }

    /// Get a mutable reference to an object
    pub fn get_object_mut(&mut self, id: ObjectId) -> Option<&mut SceneObject> {
        self.objects.get_mut(&id)
    }

    /// Get all objects
    pub fn objects(&self) -> &HashMap<ObjectId, SceneObject> {
        &self.objects
    }

    /// Get all objects as a sorted vector
    pub fn objects_sorted(&self) -> Vec<&SceneObject> {
        let mut objects: Vec<&SceneObject> = self.objects.values().collect();
        objects.sort_by_key(|obj| obj.id);
        objects
    }

    /// Select an object
    pub fn select_object(&mut self, id: ObjectId) {
        if self.objects.contains_key(&id) {
            self.selected_object = Some(id);
        }
    }

    /// Deselect current object
    pub fn deselect(&mut self) {
        self.selected_object = None;
    }

    /// Get currently selected object ID
    pub fn selected_object_id(&self) -> Option<ObjectId> {
        self.selected_object
    }

    /// Get currently selected object
    pub fn selected_object(&self) -> Option<&SceneObject> {
        self.selected_object.and_then(|id| self.objects.get(&id))
    }

    /// Get currently selected object (mutable)
    pub fn selected_object_mut(&mut self) -> Option<&mut SceneObject> {
        self.selected_object.and_then(|id| self.objects.get_mut(&id))
    }

    /// Find object by type (returns first match)
    pub fn find_by_type(&self, object_type: ObjectType) -> Option<ObjectId> {
        self.objects
            .iter()
            .find(|(_, obj)| obj.object_type == object_type)
            .map(|(id, _)| *id)
    }

    /// Get all objects of a specific type
    pub fn get_by_type(&self, object_type: ObjectType) -> Vec<ObjectId> {
        self.objects
            .iter()
            .filter(|(_, obj)| obj.object_type == object_type)
            .map(|(id, _)| *id)
            .collect()
    }
}

impl Default for SceneGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Serializable scene data (just transforms and metadata)
#[derive(Debug, Serialize, Deserialize)]
pub struct SceneData {
    pub objects: Vec<SceneObject>,
}

impl SceneData {
    pub fn from_scene_graph(scene: &SceneGraph) -> Self {
        let mut objects = scene.objects_sorted().into_iter().cloned().collect();
        Self { objects }
    }

    pub fn to_scene_graph(&self) -> SceneGraph {
        let mut scene = SceneGraph::new();

        for obj in &self.objects {
            let id = scene.add_object(obj.name.clone(), obj.object_type.clone());
            if let Some(scene_obj) = scene.get_object_mut(id) {
                scene_obj.transform = obj.transform;
                scene_obj.visible = obj.visible;
            }
        }

        scene
    }

    /// Load from JSON file
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let data: SceneData = serde_json::from_str(&content)?;
        Ok(data)
    }

    /// Save to JSON file with pretty formatting
    pub fn save(&self, path: &str) -> anyhow::Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// Load or create default scene
    pub fn load_or_default(path: &str) -> Self {
        Self::load(path).unwrap_or_else(|_| {
            let scene_data = Self::default();
            let _ = scene_data.save(path);
            scene_data
        })
    }

    /// Detect discrepancies between saved config and default scene
    /// Returns true if there are differences (objects in default that aren't in saved config)
    pub fn has_discrepancies(&self, default_scene: &Self) -> bool {
        // Check if default has objects not present in saved config (by name)
        let saved_names: std::collections::HashSet<&str> =
            self.objects.iter().map(|o| o.name.as_str()).collect();

        for default_obj in &default_scene.objects {
            if !saved_names.contains(default_obj.name.as_str()) {
                return true;
            }
        }

        false
    }

    /// Merge saved config with default scene
    /// Config file objects override defaults with same name
    /// New objects from default are added to the result
    pub fn merge_with_default(self, default_scene: Self) -> Self {
        use std::collections::HashMap;

        // Build map of saved objects by name (config takes precedence)
        let mut objects_by_name: HashMap<String, SceneObject> = HashMap::new();

        // First, add all default objects
        for obj in default_scene.objects {
            objects_by_name.insert(obj.name.clone(), obj);
        }

        // Then override with saved config objects (same name = config wins)
        for obj in self.objects {
            objects_by_name.insert(obj.name.clone(), obj);
        }

        // Convert back to vec and reassign IDs sequentially
        let mut objects: Vec<SceneObject> = objects_by_name.into_values().collect();

        // Sort to maintain consistent order (Skybox/Nebula at end, Cubes/Meshes first)
        objects.sort_by(|a, b| {
            match (&a.object_type, &b.object_type) {
                (ObjectType::Skybox, ObjectType::Skybox) => a.name.cmp(&b.name),
                (ObjectType::Skybox, _) => std::cmp::Ordering::Greater,
                (_, ObjectType::Skybox) => std::cmp::Ordering::Less,
                (ObjectType::Nebula, ObjectType::Nebula) => a.name.cmp(&b.name),
                (ObjectType::Nebula, _) => std::cmp::Ordering::Greater,
                (_, ObjectType::Nebula) => std::cmp::Ordering::Less,
                _ => a.name.cmp(&b.name),
            }
        });

        // Reassign IDs sequentially
        for (new_id, obj) in objects.iter_mut().enumerate() {
            obj.id = new_id;
        }

        Self { objects }
    }

    /// Load and merge with default scene
    /// If saved config exists, merge it with default (config overrides)
    /// If no saved config, use default
    pub fn load_and_merge_with_default(path: &str) -> Self {
        let default_scene = Self::default();

        match Self::load(path) {
            Ok(saved_config) => {
                // Check if there are discrepancies
                if saved_config.has_discrepancies(&default_scene) {
                    println!("Scene config discrepancy detected:");
                    println!("  Saved config has {} objects", saved_config.objects.len());
                    println!("  Default scene has {} objects", default_scene.objects.len());
                    println!("  Merging... (config overrides defaults with same name)");

                    let merged = saved_config.merge_with_default(default_scene);

                    // Save the merged result
                    let _ = merged.save(path);
                    println!("  Merged scene saved with {} objects", merged.objects.len());

                    merged
                } else {
                    // No discrepancies, use saved config as-is
                    saved_config
                }
            }
            Err(_) => {
                // No saved config, use default and save it
                println!("No saved scene found, creating default at {}", path);
                let _ = default_scene.save(path);
                default_scene
            }
        }
    }
}

impl Default for SceneData {
    fn default() -> Self {
        Self {
            objects: vec![
                SceneObject::new(0, "Cube 1".to_string(), ObjectType::Cube)
                    .with_transform(Transform::identity()),
                SceneObject::new(1, "Cube 2".to_string(), ObjectType::Cube)
                    .with_transform(Transform::new(
                        glam::Vec3::new(3.0, 0.0, 0.0),
                        glam::Quat::IDENTITY,
                        glam::Vec3::ONE
                    )),
                SceneObject::new(2, "Fed Cruiser".to_string(), ObjectType::Mesh("content/models/Fed_cruiser_ship.obj".to_string()))
                    .with_transform(Transform::new(
                        glam::Vec3::new(0.0, 3.0, 0.0),
                        glam::Quat::IDENTITY,
                        glam::Vec3::ONE
                    )),
                SceneObject::new(3, "Nebula".to_string(), ObjectType::Nebula)
                    .with_transform(Transform::identity()),
                SceneObject::new(4, "Skybox".to_string(), ObjectType::Skybox)
                    .with_transform(Transform::identity()),
            ],
        }
    }
}
