/// Movement widget geometry and rendering
///
/// Provides 3D widget components for ship movement planning

use glam::{Vec3, Vec4, Quat, Mat4, DVec3};

/// Widget geometry type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WidgetElement {
    UpArrow,
    DownArrow,
    RollCone,
    YawCube,
    PitchCube,
    MovementPlane,
}

/// Arrow geometry for elevation control
pub struct ArrowGeometry {
    pub vertices: Vec<Vec3>,
    pub indices: Vec<u32>,
}

impl ArrowGeometry {
    /// Create arrow pointing up (+Y)
    pub fn new() -> Self {
        let shaft_radius = 0.1;
        let shaft_height = 1.5;
        let head_radius = 0.3;
        let head_height = 0.5;
        let segments = 8;

        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        // Shaft (cylinder)
        for i in 0..segments {
            let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let x = shaft_radius * angle.cos();
            let z = shaft_radius * angle.sin();

            // Bottom
            vertices.push(Vec3::new(x, 0.0, z));
            // Top of shaft
            vertices.push(Vec3::new(x, shaft_height, z));
        }

        // Indices for shaft
        for i in 0..segments {
            let next = (i + 1) % segments;
            let base = (i * 2) as u32;
            let next_base = (next * 2) as u32;

            // Two triangles per segment
            indices.push(base);
            indices.push(base + 1);
            indices.push(next_base);

            indices.push(base + 1);
            indices.push(next_base + 1);
            indices.push(next_base);
        }

        // Arrow head (cone)
        let head_base_idx = vertices.len() as u32;
        for i in 0..segments {
            let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let x = head_radius * angle.cos();
            let z = head_radius * angle.sin();

            vertices.push(Vec3::new(x, shaft_height, z));
        }

        // Tip of arrow
        let tip_idx = vertices.len() as u32;
        vertices.push(Vec3::new(0.0, shaft_height + head_height, 0.0));

        // Indices for cone
        for i in 0..segments {
            let next = (i + 1) % segments;
            indices.push(head_base_idx + i as u32);
            indices.push(tip_idx);
            indices.push(head_base_idx + next as u32);
        }

        Self { vertices, indices }
    }

    /// Create arrow pointing down (-Y)
    pub fn new_down() -> Self {
        let mut arrow = Self::new();
        // Flip Y coordinates
        for vertex in &mut arrow.vertices {
            vertex.y = -vertex.y;
        }
        arrow
    }
}

/// Cone geometry for roll control
pub struct ConeGeometry {
    pub vertices: Vec<Vec3>,
    pub indices: Vec<u32>,
}

impl ConeGeometry {
    /// Create cone pointing forward (+Z)
    pub fn new(radius: f32, height: f32) -> Self {
        let segments = 16;
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        // Base circle
        let base_center_idx = vertices.len() as u32;
        vertices.push(Vec3::new(0.0, 0.0, 0.0));

        for i in 0..segments {
            let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let x = radius * angle.cos();
            let y = radius * angle.sin();
            vertices.push(Vec3::new(x, y, 0.0));
        }

        // Tip of cone
        let tip_idx = vertices.len() as u32;
        vertices.push(Vec3::new(0.0, 0.0, height));

        // Base circle triangles
        for i in 0..segments {
            let next = (i + 1) % segments;
            indices.push(base_center_idx);
            indices.push(base_center_idx + 1 + i as u32);
            indices.push(base_center_idx + 1 + next as u32);
        }

        // Cone sides
        for i in 0..segments {
            let next = (i + 1) % segments;
            indices.push(base_center_idx + 1 + i as u32);
            indices.push(tip_idx);
            indices.push(base_center_idx + 1 + next as u32);
        }

        Self { vertices, indices }
    }
}

/// Cube geometry for rotation control
pub struct CubeGeometry {
    pub vertices: Vec<Vec3>,
    pub indices: Vec<u32>,
}

impl CubeGeometry {
    /// Create solid cube with triangles
    pub fn new(size: f32) -> Self {
        let half = size * 0.5;

        // 24 vertices (4 per face for proper normals)
        let vertices = vec![
            // Front face (+Z)
            Vec3::new(-half, -half,  half), Vec3::new( half, -half,  half),
            Vec3::new( half,  half,  half), Vec3::new(-half,  half,  half),
            // Back face (-Z)
            Vec3::new( half, -half, -half), Vec3::new(-half, -half, -half),
            Vec3::new(-half,  half, -half), Vec3::new( half,  half, -half),
            // Top face (+Y)
            Vec3::new(-half,  half,  half), Vec3::new( half,  half,  half),
            Vec3::new( half,  half, -half), Vec3::new(-half,  half, -half),
            // Bottom face (-Y)
            Vec3::new(-half, -half, -half), Vec3::new( half, -half, -half),
            Vec3::new( half, -half,  half), Vec3::new(-half, -half,  half),
            // Right face (+X)
            Vec3::new( half, -half,  half), Vec3::new( half, -half, -half),
            Vec3::new( half,  half, -half), Vec3::new( half,  half,  half),
            // Left face (-X)
            Vec3::new(-half, -half, -half), Vec3::new(-half, -half,  half),
            Vec3::new(-half,  half,  half), Vec3::new(-half,  half, -half),
        ];

        // Triangle indices (2 triangles per face = 12 triangles = 36 indices)
        let indices = vec![
            // Front
            0, 1, 2,  0, 2, 3,
            // Back
            4, 5, 6,  4, 6, 7,
            // Top
            8, 9, 10,  8, 10, 11,
            // Bottom
            12, 13, 14,  12, 14, 15,
            // Right
            16, 17, 18,  16, 18, 19,
            // Left
            20, 21, 22,  20, 22, 23,
        ];

        Self { vertices, indices }
    }
}

/// Cylinder geometry for movement range visualization
pub struct CylinderGeometry {
    pub vertices: Vec<Vec3>,
    pub indices: Vec<u32>,
}

impl CylinderGeometry {
    /// Create cylinder with horizontal slices
    pub fn new(radius: f32, height_min: f32, height_max: f32, num_slices: u32) -> Self {
        let segments = 32;
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let slice_height = (height_max - height_min) / (num_slices - 1) as f32;

        // Create vertices for each slice
        for slice in 0..num_slices {
            let y = height_min + slice as f32 * slice_height;

            for seg in 0..segments {
                let angle = (seg as f32 / segments as f32) * std::f32::consts::TAU;
                let x = radius * angle.cos();
                let z = radius * angle.sin();

                vertices.push(Vec3::new(x, y, z));
            }
        }

        // Horizontal circle indices (slices)
        for slice in 0..num_slices {
            let base = (slice * segments) as u32;
            for seg in 0..segments {
                let next = (seg + 1) % segments;
                indices.push(base + seg as u32);
                indices.push(base + next as u32);
            }
        }

        // Vertical line indices
        for seg in 0..segments {
            for slice in 0..(num_slices - 1) {
                let base = (slice * segments + seg) as u32;
                let next_slice = ((slice + 1) * segments + seg) as u32;
                indices.push(base);
                indices.push(next_slice);
            }
        }

        Self { vertices, indices }
    }
}

/// Rotation constraint arc (90-degree cone)
pub struct RotationArcGeometry {
    pub vertices: Vec<Vec3>,
    pub indices: Vec<u32>,
}

impl RotationArcGeometry {
    /// Create 90-degree arc showing rotation constraint
    pub fn new(radius: f32, max_angle: f32) -> Self {
        let segments = 32;
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        let half_angle = max_angle / 2.0;

        // Center point
        let center_idx = vertices.len() as u32;
        vertices.push(Vec3::ZERO);

        // Arc vertices
        for i in 0..=segments {
            let t = i as f32 / segments as f32;
            let angle = -half_angle + t * max_angle;

            let x = radius * angle.sin();
            let z = radius * angle.cos();

            vertices.push(Vec3::new(x, 0.0, z));
        }

        // Arc line indices
        for i in 0..segments {
            indices.push(center_idx + 1 + i as u32);
            indices.push(center_idx + 2 + i as u32);
        }

        // Lines from center to arc endpoints
        indices.push(center_idx);
        indices.push(center_idx + 1);

        indices.push(center_idx);
        indices.push(center_idx + segments as u32 + 1);

        Self { vertices, indices }
    }
}

/// Movement widget state
pub struct MovementWidget {
    /// Widget position in world space (64-bit)
    pub position: DVec3,

    /// Widget rotation
    pub rotation: Quat,

    /// Currently hovered element
    pub hovered_element: Option<WidgetElement>,

    /// Currently dragging element
    pub dragging_element: Option<WidgetElement>,

    /// Is widget visible?
    pub visible: bool,

    /// Widget scale
    pub scale: f32,

    /// Ship mesh bounds (local space) for widget positioning
    pub bounds_min: Vec3,
    pub bounds_max: Vec3,
}

impl MovementWidget {
    pub fn new() -> Self {
        Self {
            position: DVec3::ZERO,
            rotation: Quat::IDENTITY,
            hovered_element: None,
            dragging_element: None,
            visible: false,
            scale: 1.0,
            bounds_min: Vec3::new(-1.0, -1.0, -1.0),
            bounds_max: Vec3::new(1.0, 1.0, 1.0),
        }
    }

    /// Get color for widget element based on hover/drag state
    pub fn get_element_color(&self, element: WidgetElement) -> Vec4 {
        let base_color = match element {
            WidgetElement::UpArrow => Vec4::new(0.0, 1.0, 0.0, 1.0),     // Green
            WidgetElement::DownArrow => Vec4::new(1.0, 0.0, 0.0, 1.0),   // Red
            WidgetElement::RollCone => Vec4::new(1.0, 1.0, 0.0, 1.0),    // Yellow
            WidgetElement::YawCube => Vec4::new(0.0, 0.0, 1.0, 1.0),     // Blue
            WidgetElement::PitchCube => Vec4::new(0.0, 1.0, 1.0, 1.0),   // Cyan
            WidgetElement::MovementPlane => Vec4::new(1.0, 1.0, 1.0, 0.3), // White transparent
        };

        // Brighten on hover
        if self.hovered_element == Some(element) {
            Vec4::new(
                (base_color.x * 1.5).min(1.0),
                (base_color.y * 1.5).min(1.0),
                (base_color.z * 1.5).min(1.0),
                base_color.w,
            )
        } else {
            base_color
        }
    }

    /// Get model matrix for widget element
    pub fn get_element_transform(&self, element: WidgetElement, camera_origin: DVec3) -> Mat4 {
        // Widget offset distance from bounds
        let widget_offset = 1.0;

        // Get offset in local space (same logic as ray_pick)
        let element_offset_local = match element {
            WidgetElement::UpArrow => {
                let offset_y = self.bounds_max.y + widget_offset;
                Vec3::new(0.0, offset_y, 0.0)
            }
            WidgetElement::DownArrow => {
                let offset_y = self.bounds_min.y - widget_offset;
                Vec3::new(0.0, offset_y, 0.0)
            }
            WidgetElement::RollCone => {
                let offset_z = self.bounds_max.z + widget_offset;
                Vec3::new(0.0, 0.0, offset_z)
            }
            WidgetElement::YawCube => {
                let offset_z = self.bounds_min.z - widget_offset;
                Vec3::new(0.0, 0.0, offset_z)
            }
            WidgetElement::PitchCube => {
                let offset_y = self.bounds_max.y + widget_offset;
                let offset_z = self.bounds_min.z - widget_offset;
                Vec3::new(0.0, offset_y, offset_z)
            }
            WidgetElement::MovementPlane => Vec3::ZERO,
        };

        // Apply ship rotation to local offset
        let rotated_offset = self.rotation * element_offset_local;

        // Calculate world position (64-bit)
        let element_world_pos = self.position + (rotated_offset * self.scale).as_dvec3();

        // Convert to camera-relative 32-bit position
        let camera_relative_pos = (element_world_pos - camera_origin).as_vec3();

        // Create transform at the final position (no rotation, widgets always face camera)
        Mat4::from_scale_rotation_translation(
            Vec3::splat(self.scale),
            Quat::IDENTITY,
            camera_relative_pos,
        )
    }

    /// Get label text for a widget element
    pub fn get_element_label(&self, element: WidgetElement) -> &'static str {
        match element {
            WidgetElement::UpArrow => "Elevation Up",
            WidgetElement::DownArrow => "Elevation Down",
            WidgetElement::RollCone => "Roll",
            WidgetElement::YawCube => "Yaw",
            WidgetElement::PitchCube => "Pitch",
            WidgetElement::MovementPlane => "Movement Plane",
        }
    }

    /// Check if a ray intersects any widget element (returns closest hit)
    pub fn ray_pick(&mut self, ray_origin: DVec3, ray_dir: DVec3, _camera_origin: DVec3) -> Option<WidgetElement> {
        let cube_size = 0.8 * self.scale;
        let half_size = cube_size / 2.0;

        let elements = [
            WidgetElement::UpArrow,
            WidgetElement::DownArrow,
            WidgetElement::RollCone,
            WidgetElement::YawCube,
            WidgetElement::PitchCube,
        ];

        let mut closest_element = None;
        let mut closest_distance = f64::MAX;

        // Widget offset distance from bounds (must match get_element_transform!)
        let widget_offset = 1.0;

        for element in elements.iter() {
            // Get element offset in LOCAL space (MUST match rendering offsets!)
            let element_offset_local = match element {
                WidgetElement::UpArrow => {
                    let offset_y = self.bounds_max.y + widget_offset;
                    Vec3::new(0.0, offset_y, 0.0)
                }
                WidgetElement::DownArrow => {
                    let offset_y = self.bounds_min.y - widget_offset;
                    Vec3::new(0.0, offset_y, 0.0)
                }
                WidgetElement::RollCone => {
                    let offset_z = self.bounds_max.z + widget_offset;
                    Vec3::new(0.0, 0.0, offset_z)
                }
                WidgetElement::YawCube => {
                    let offset_z = self.bounds_min.z - widget_offset;
                    Vec3::new(0.0, 0.0, offset_z)
                }
                WidgetElement::PitchCube => {
                    let offset_y = self.bounds_max.y + widget_offset;
                    let offset_z = self.bounds_min.z - widget_offset;
                    Vec3::new(0.0, offset_y, offset_z)
                }
                _ => Vec3::ZERO,
            };

            // Apply widget rotation to offset (ship rotation)
            let rotated_offset = self.rotation * element_offset_local;

            // Scale the offset and convert to 64-bit
            let element_offset = rotated_offset.as_dvec3() * self.scale as f64;

            // Calculate world position
            let element_pos = self.position + element_offset;

            // AABB intersection test
            let min = element_pos - DVec3::splat(half_size as f64);
            let max = element_pos + DVec3::splat(half_size as f64);

            if let Some(distance) = ray_aabb_intersect(ray_origin, ray_dir, min, max) {
                if distance < closest_distance {
                    closest_distance = distance;
                    closest_element = Some(*element);
                }
            }
        }

        closest_element
    }
}

/// Ray-AABB intersection test
fn ray_aabb_intersect(ray_origin: DVec3, ray_dir: DVec3, aabb_min: DVec3, aabb_max: DVec3) -> Option<f64> {
    let inv_dir = DVec3::new(
        if ray_dir.x.abs() < 1e-10 { 1e10 } else { 1.0 / ray_dir.x },
        if ray_dir.y.abs() < 1e-10 { 1e10 } else { 1.0 / ray_dir.y },
        if ray_dir.z.abs() < 1e-10 { 1e10 } else { 1.0 / ray_dir.z },
    );

    let t1 = (aabb_min.x - ray_origin.x) * inv_dir.x;
    let t2 = (aabb_max.x - ray_origin.x) * inv_dir.x;
    let t3 = (aabb_min.y - ray_origin.y) * inv_dir.y;
    let t4 = (aabb_max.y - ray_origin.y) * inv_dir.y;
    let t5 = (aabb_min.z - ray_origin.z) * inv_dir.z;
    let t6 = (aabb_max.z - ray_origin.z) * inv_dir.z;

    let tmin = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
    let tmax = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));

    if tmax < 0.0 || tmin > tmax {
        None
    } else {
        Some(tmin.max(0.0))
    }
}
