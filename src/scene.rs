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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectType {
    Cube,
    Nebula,
    Skybox,
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
}

impl SceneObject {
    pub fn new(id: ObjectId, name: String, object_type: ObjectType) -> Self {
        Self {
            id,
            name,
            object_type,
            transform: Transform::default(),
            visible: true,
        }
    }

    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = transform;
        self
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
            let id = scene.add_object(obj.name.clone(), obj.object_type);
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
}

impl Default for SceneData {
    fn default() -> Self {
        Self {
            objects: vec![
                SceneObject::new(0, "Cube".to_string(), ObjectType::Cube)
                    .with_transform(Transform::identity()),
                SceneObject::new(1, "Nebula".to_string(), ObjectType::Nebula)
                    .with_transform(Transform::identity()),
                SceneObject::new(2, "Skybox".to_string(), ObjectType::Skybox)
                    .with_transform(Transform::identity()),
            ],
        }
    }
}
