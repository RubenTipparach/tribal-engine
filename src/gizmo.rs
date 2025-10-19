use glam::{Mat4, Vec2, Vec3, Vec4};
use crate::scene::{SceneGraph, ObjectId, ObjectType};
use crate::core::Camera;
use crate::mesh::Vertex;

/// Gizmo operation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GizmoMode {
    Translate,
    Rotate,
    Scale,
}

/// Gizmo axis being manipulated
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GizmoAxis {
    None,
    X,
    Y,
    Z,
}

/// Gizmo state and interaction
pub struct GizmoState {
    pub mode: GizmoMode,
    pub enabled: bool,
    pub using_gizmo: bool,
    pub active_axis: GizmoAxis,
    pub hovered_axis: GizmoAxis,
}

impl GizmoState {
    pub fn new() -> Self {
        Self {
            mode: GizmoMode::Translate,
            enabled: true,
            using_gizmo: false,
            active_axis: GizmoAxis::None,
            hovered_axis: GizmoAxis::None,
        }
    }

    pub fn start_drag(&mut self, axis: GizmoAxis) {
        self.active_axis = axis;
        self.using_gizmo = true;
    }

    pub fn end_drag(&mut self) {
        self.active_axis = GizmoAxis::None;
        self.using_gizmo = false;
    }
}

impl Default for GizmoState {
    fn default() -> Self {
        Self::new()
    }
}

/// Gizmo mesh generator
pub struct GizmoMesh;

impl GizmoMesh {
    /// Generate translation gizmo (3 arrows)
    pub fn generate_translate_arrows() -> (Vec<Vertex>, Vec<u32>) {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let arrow_length = 1.0;
        let arrow_thickness = 0.05;
        let arrow_head_length = 0.2;

        // X axis (Red) - Arrow along +X
        Self::add_arrow(&mut vertices, &mut indices,
            Vec3::ZERO, Vec3::X, arrow_length, arrow_thickness, arrow_head_length,
            Vec3::new(1.0, 0.0, 0.0));

        // Y axis (Green) - Arrow along +Y
        Self::add_arrow(&mut vertices, &mut indices,
            Vec3::ZERO, Vec3::Y, arrow_length, arrow_thickness, arrow_head_length,
            Vec3::new(0.0, 1.0, 0.0));

        // Z axis (Blue) - Arrow along +Z
        Self::add_arrow(&mut vertices, &mut indices,
            Vec3::ZERO, Vec3::Z, arrow_length, arrow_thickness, arrow_head_length,
            Vec3::new(0.0, 0.0, 1.0));

        (vertices, indices)
    }

    fn add_arrow(
        vertices: &mut Vec<Vertex>,
        indices: &mut Vec<u32>,
        start: Vec3,
        direction: Vec3,
        length: f32,
        thickness: f32,
        head_length: f32,
        color: Vec3,
    ) {
        let base_idx = vertices.len() as u32;
        let shaft_end = start + direction * (length - head_length);
        let arrow_end = start + direction * length;

        // Arrow shaft (simple line for now, can be made into cylinder)
        vertices.push(Vertex {
            position: start,
            normal: direction,
            uv: Vec2::ZERO,
        });
        vertices.push(Vertex {
            position: shaft_end,
            normal: direction,
            uv: Vec2::new(1.0, 0.0),
        });

        // Arrow shaft line
        indices.push(base_idx);
        indices.push(base_idx + 1);

        // Arrow head (cone tip)
        vertices.push(Vertex {
            position: shaft_end,
            normal: direction,
            uv: Vec2::ZERO,
        });
        vertices.push(Vertex {
            position: arrow_end,
            normal: direction,
            uv: Vec2::new(1.0, 0.0),
        });

        indices.push(base_idx + 2);
        indices.push(base_idx + 3);
    }
}

/// Ray for 3D picking
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    /// Create a ray from screen coordinates
    pub fn from_screen(
        mouse_x: f32,
        mouse_y: f32,
        viewport_width: f32,
        viewport_height: f32,
        view_matrix: Mat4,
        proj_matrix: Mat4,
    ) -> Self {
        // Normalize screen coordinates to NDC (-1 to 1)
        let ndc_x = (2.0 * mouse_x) / viewport_width - 1.0;
        let ndc_y = 1.0 - (2.0 * mouse_y) / viewport_height;

        // Create ray in clip space
        let ray_clip = Vec4::new(ndc_x, ndc_y, -1.0, 1.0);

        // Transform to view space
        let inv_proj = proj_matrix.inverse();
        let ray_view = inv_proj * ray_clip;
        let ray_view = Vec4::new(ray_view.x, ray_view.y, -1.0, 0.0);

        // Transform to world space
        let inv_view = view_matrix.inverse();
        let ray_world = inv_view * ray_view;

        let direction = Vec3::new(ray_world.x, ray_world.y, ray_world.z).normalize();
        let origin = inv_view.w_axis.truncate();

        Self { origin, direction }
    }

    /// Test intersection with axis-aligned bounding box
    pub fn intersects_aabb(&self, min: Vec3, max: Vec3) -> bool {
        let inv_dir = Vec3::new(
            1.0 / self.direction.x,
            1.0 / self.direction.y,
            1.0 / self.direction.z,
        );

        let t1 = (min.x - self.origin.x) * inv_dir.x;
        let t2 = (max.x - self.origin.x) * inv_dir.x;
        let t3 = (min.y - self.origin.y) * inv_dir.y;
        let t4 = (max.y - self.origin.y) * inv_dir.y;
        let t5 = (min.z - self.origin.z) * inv_dir.z;
        let t6 = (max.z - self.origin.z) * inv_dir.z;

        let tmin = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
        let tmax = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));

        tmax >= tmin && tmax >= 0.0
    }

    /// Test intersection with sphere
    pub fn intersects_sphere(&self, center: Vec3, radius: f32) -> Option<f32> {
        let oc = self.origin - center;
        let a = self.direction.dot(self.direction);
        let b = 2.0 * oc.dot(self.direction);
        let c = oc.dot(oc) - radius * radius;
        let discriminant = b * b - 4.0 * a * c;

        if discriminant < 0.0 {
            None
        } else {
            let t = (-b - discriminant.sqrt()) / (2.0 * a);
            if t >= 0.0 {
                Some(t)
            } else {
                None
            }
        }
    }
}

/// Object picker for selecting objects in 3D space
pub struct ObjectPicker {
    pub hovered_object: Option<ObjectId>,
}

impl ObjectPicker {
    pub fn new() -> Self {
        Self {
            hovered_object: None,
        }
    }

    /// Pick object from scene using ray casting
    pub fn pick_object(
        &mut self,
        mouse_x: f32,
        mouse_y: f32,
        viewport_width: f32,
        viewport_height: f32,
        scene: &SceneGraph,
        camera: &Camera,
    ) -> Option<ObjectId> {
        let view = camera.view_matrix();
        let proj = camera.projection_matrix(viewport_width / viewport_height);

        let ray = Ray::from_screen(mouse_x, mouse_y, viewport_width, viewport_height, view, proj);

        let mut closest_object: Option<(ObjectId, f32)> = None;

        // Check all objects (skip nebula - it's not directly selectable)
        for obj in scene.objects().values() {
            if !obj.visible || obj.object_type == ObjectType::Nebula {
                continue;
            }

            let pos = obj.transform.position;
            let scale = obj.transform.scale;

            // Use a larger bounding sphere for easier picking
            // Radius is the max of the scale components * 1.5 for easier selection
            let radius = scale.x.max(scale.y).max(scale.z) * 1.5;

            if let Some(distance) = ray.intersects_sphere(pos, radius) {
                if let Some((_, closest_dist)) = closest_object {
                    if distance < closest_dist {
                        closest_object = Some((obj.id, distance));
                    }
                } else {
                    closest_object = Some((obj.id, distance));
                }
            }
        }

        self.hovered_object = closest_object.map(|(id, _)| id);
        self.hovered_object
    }
}

impl Default for ObjectPicker {
    fn default() -> Self {
        Self::new()
    }
}
