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

    /// Check which gizmo arrow is being hovered/clicked
    pub fn pick_axis(
        &mut self,
        mouse_x: f32,
        mouse_y: f32,
        viewport_width: f32,
        viewport_height: f32,
        object_pos: Vec3,
        camera: &Camera,
    ) -> GizmoAxis {
        let view = camera.view_matrix();
        let proj = camera.projection_matrix(viewport_width / viewport_height);
        let ray = Ray::from_screen(mouse_x, mouse_y, viewport_width, viewport_height, view, proj);

        let arrow_length = 1.0;
        let pick_radius = 0.15; // Generous picking radius

        let mut closest_axis = GizmoAxis::None;
        let mut closest_dist = f32::MAX;

        // Check X axis (Red)
        let x_end = object_pos + Vec3::X * arrow_length;
        if let Some(dist) = ray.intersects_cylinder(object_pos, x_end, pick_radius) {
            if dist < closest_dist {
                closest_dist = dist;
                closest_axis = GizmoAxis::X;
            }
        }

        // Check Y axis (Green)
        let y_end = object_pos + Vec3::Y * arrow_length;
        if let Some(dist) = ray.intersects_cylinder(object_pos, y_end, pick_radius) {
            if dist < closest_dist {
                closest_dist = dist;
                closest_axis = GizmoAxis::Y;
            }
        }

        // Check Z axis (Blue)
        let z_end = object_pos + Vec3::Z * arrow_length;
        if let Some(dist) = ray.intersects_cylinder(object_pos, z_end, pick_radius) {
            if dist < closest_dist {
                closest_axis = GizmoAxis::Z;
            }
        }

        self.hovered_axis = closest_axis;
        closest_axis
    }

    /// Apply drag motion to object position
    pub fn apply_drag(
        &self,
        old_mouse: (f32, f32),
        new_mouse: (f32, f32),
        viewport_width: f32,
        viewport_height: f32,
        object_pos: Vec3,
        camera: &Camera,
    ) -> Vec3 {
        if self.active_axis == GizmoAxis::None {
            return object_pos;
        }

        let view = camera.view_matrix();
        let proj = camera.projection_matrix(viewport_width / viewport_height);

        // Get rays for old and new mouse positions
        let old_ray = Ray::from_screen(old_mouse.0, old_mouse.1, viewport_width, viewport_height, view, proj);
        let new_ray = Ray::from_screen(new_mouse.0, new_mouse.1, viewport_width, viewport_height, view, proj);

        // Get axis direction in world space
        let axis_dir = match self.active_axis {
            GizmoAxis::X => Vec3::X,
            GizmoAxis::Y => Vec3::Y,
            GizmoAxis::Z => Vec3::Z,
            GizmoAxis::None => return object_pos,
        };

        // Project ray movement onto axis
        let old_point = old_ray.project_onto_axis(object_pos, axis_dir);
        let new_point = new_ray.project_onto_axis(object_pos, axis_dir);

        object_pos + (new_point - old_point)
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
        _thickness: f32,
        head_length: f32,
        _color: Vec3,
    ) {
        let base_idx = vertices.len() as u32;
        let shaft_end = start + direction * (length - head_length);
        let arrow_end = start + direction * length;

        // Build arrow as a cylinder shaft + cone head using triangles
        let segments = 8;

        // Get perpendicular vectors for the cylinder
        let (perp1, perp2) = if direction.x.abs() < 0.9 {
            let perp1 = direction.cross(Vec3::X).normalize() * 0.03;
            let perp2 = direction.cross(perp1).normalize() * 0.03;
            (perp1, perp2)
        } else {
            let perp1 = direction.cross(Vec3::Y).normalize() * 0.03;
            let perp2 = direction.cross(perp1).normalize() * 0.03;
            (perp1, perp2)
        };

        // Create cylinder shaft
        for i in 0..segments {
            let angle = (i as f32 / segments as f32) * 2.0 * std::f32::consts::PI;
            let next_angle = ((i + 1) as f32 / segments as f32) * 2.0 * std::f32::consts::PI;

            let offset = perp1 * angle.cos() + perp2 * angle.sin();
            let next_offset = perp1 * next_angle.cos() + perp2 * next_angle.sin();

            // Bottom ring
            vertices.push(Vertex {
                position: start + offset,
                normal: direction,
                uv: Vec2::ZERO,
            });

            // Top ring
            vertices.push(Vertex {
                position: shaft_end + offset,
                normal: direction,
                uv: Vec2::ZERO,
            });

            // Triangle 1 of quad
            indices.push(base_idx + (i * 2) as u32);
            indices.push(base_idx + (i * 2 + 1) as u32);
            indices.push(base_idx + ((i * 2 + 2) % (segments * 2)) as u32);

            // Triangle 2 of quad
            indices.push(base_idx + (i * 2 + 1) as u32);
            indices.push(base_idx + ((i * 2 + 3) % (segments * 2)) as u32);
            indices.push(base_idx + ((i * 2 + 2) % (segments * 2)) as u32);
        }

        let cone_base_idx = vertices.len() as u32;
        let head_radius = 0.1;

        // Create cone head
        for i in 0..segments {
            let angle = (i as f32 / segments as f32) * 2.0 * std::f32::consts::PI;
            let offset = (perp1 * angle.cos() + perp2 * angle.sin()) * (head_radius / 0.03);

            vertices.push(Vertex {
                position: shaft_end + offset,
                normal: direction,
                uv: Vec2::ZERO,
            });
        }

        // Cone tip
        vertices.push(Vertex {
            position: arrow_end,
            normal: direction,
            uv: Vec2::ZERO,
        });

        let tip_idx = vertices.len() as u32 - 1;

        // Cone triangles
        for i in 0..segments {
            indices.push(cone_base_idx + i as u32);
            indices.push(tip_idx);
            indices.push(cone_base_idx + ((i + 1) % segments) as u32);
        }
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
        // Note: For Vulkan with flipped Y projection, we need to NOT flip Y here
        let ndc_x = (2.0 * mouse_x) / viewport_width - 1.0;
        let ndc_y = (2.0 * mouse_y) / viewport_height - 1.0; // Changed: removed the flip

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

    /// Test intersection with a cylinder defined by start and end points
    pub fn intersects_cylinder(&self, start: Vec3, end: Vec3, radius: f32) -> Option<f32> {
        let axis = (end - start).normalize();
        let length = (end - start).length();

        // Vector from start to ray origin
        let oc = self.origin - start;

        // Project onto cylinder axis
        let axis_dot_dir = axis.dot(self.direction);
        let axis_dot_oc = axis.dot(oc);

        // Solve quadratic equation for cylinder intersection
        let a = 1.0 - axis_dot_dir * axis_dot_dir;
        let b = 2.0 * (oc.dot(self.direction) - axis_dot_oc * axis_dot_dir);
        let c = oc.dot(oc) - axis_dot_oc * axis_dot_oc - radius * radius;

        let discriminant = b * b - 4.0 * a * c;
        if discriminant < 0.0 {
            return None;
        }

        let t = (-b - discriminant.sqrt()) / (2.0 * a);
        if t < 0.0 {
            return None;
        }

        // Check if intersection is within cylinder length
        let point = self.origin + self.direction * t;
        let projection = (point - start).dot(axis);

        if projection >= 0.0 && projection <= length {
            Some(t)
        } else {
            None
        }
    }

    /// Project the ray onto an axis and find the closest point
    pub fn project_onto_axis(&self, point_on_axis: Vec3, axis_dir: Vec3) -> Vec3 {
        // Find the closest point on the ray to the axis
        let w = self.origin - point_on_axis;
        let a = self.direction.dot(self.direction);
        let b = self.direction.dot(axis_dir);
        let c = axis_dir.dot(axis_dir);
        let d = self.direction.dot(w);
        let e = axis_dir.dot(w);

        let denom = a * c - b * b;
        let t = if denom.abs() < 1e-6 {
            0.0
        } else {
            (b * e - c * d) / denom
        };

        // Point on ray
        let point_on_ray = self.origin + self.direction * t;

        // Project onto axis
        let projection = (point_on_ray - point_on_axis).dot(axis_dir);
        point_on_axis + axis_dir * projection
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

        // Check all objects (skip nebula and skybox - they're not selectable)
        for obj in scene.objects().values() {
            if !obj.visible || obj.object_type == ObjectType::Nebula || obj.object_type == ObjectType::Skybox {
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
