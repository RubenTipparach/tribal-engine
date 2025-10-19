use ash::vk;
use glam::{Vec2, Vec3};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
    pub uv: Vec2,
}

impl Vertex {
    pub fn get_binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(std::mem::size_of::<Vertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
    }

    pub fn get_attribute_descriptions() -> [vk::VertexInputAttributeDescription; 3] {
        [
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(0)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(0),
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(1)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(std::mem::size_of::<Vec3>() as u32),
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(2)
                .format(vk::Format::R32G32_SFLOAT)
                .offset((std::mem::size_of::<Vec3>() * 2) as u32),
        ]
    }
}

#[derive(Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn create_cube() -> Self {
        let vertices = vec![
            // Front face (Z+)
            Vertex {
                position: Vec3::new(-0.5, -0.5, 0.5),
                normal: Vec3::new(0.0, 0.0, 1.0),
                uv: Vec2::new(0.0, 0.0),
            },
            Vertex {
                position: Vec3::new(0.5, -0.5, 0.5),
                normal: Vec3::new(0.0, 0.0, 1.0),
                uv: Vec2::new(1.0, 0.0),
            },
            Vertex {
                position: Vec3::new(0.5, 0.5, 0.5),
                normal: Vec3::new(0.0, 0.0, 1.0),
                uv: Vec2::new(1.0, 1.0),
            },
            Vertex {
                position: Vec3::new(-0.5, 0.5, 0.5),
                normal: Vec3::new(0.0, 0.0, 1.0),
                uv: Vec2::new(0.0, 1.0),
            },
            // Back face (Z-)
            Vertex {
                position: Vec3::new(0.5, -0.5, -0.5),
                normal: Vec3::new(0.0, 0.0, -1.0),
                uv: Vec2::new(0.0, 0.0),
            },
            Vertex {
                position: Vec3::new(-0.5, -0.5, -0.5),
                normal: Vec3::new(0.0, 0.0, -1.0),
                uv: Vec2::new(1.0, 0.0),
            },
            Vertex {
                position: Vec3::new(-0.5, 0.5, -0.5),
                normal: Vec3::new(0.0, 0.0, -1.0),
                uv: Vec2::new(1.0, 1.0),
            },
            Vertex {
                position: Vec3::new(0.5, 0.5, -0.5),
                normal: Vec3::new(0.0, 0.0, -1.0),
                uv: Vec2::new(0.0, 1.0),
            },
            // Top face (Y+)
            Vertex {
                position: Vec3::new(-0.5, 0.5, 0.5),
                normal: Vec3::new(0.0, 1.0, 0.0),
                uv: Vec2::new(0.0, 0.0),
            },
            Vertex {
                position: Vec3::new(0.5, 0.5, 0.5),
                normal: Vec3::new(0.0, 1.0, 0.0),
                uv: Vec2::new(1.0, 0.0),
            },
            Vertex {
                position: Vec3::new(0.5, 0.5, -0.5),
                normal: Vec3::new(0.0, 1.0, 0.0),
                uv: Vec2::new(1.0, 1.0),
            },
            Vertex {
                position: Vec3::new(-0.5, 0.5, -0.5),
                normal: Vec3::new(0.0, 1.0, 0.0),
                uv: Vec2::new(0.0, 1.0),
            },
            // Bottom face (Y-)
            Vertex {
                position: Vec3::new(-0.5, -0.5, -0.5),
                normal: Vec3::new(0.0, -1.0, 0.0),
                uv: Vec2::new(0.0, 0.0),
            },
            Vertex {
                position: Vec3::new(0.5, -0.5, -0.5),
                normal: Vec3::new(0.0, -1.0, 0.0),
                uv: Vec2::new(1.0, 0.0),
            },
            Vertex {
                position: Vec3::new(0.5, -0.5, 0.5),
                normal: Vec3::new(0.0, -1.0, 0.0),
                uv: Vec2::new(1.0, 1.0),
            },
            Vertex {
                position: Vec3::new(-0.5, -0.5, 0.5),
                normal: Vec3::new(0.0, -1.0, 0.0),
                uv: Vec2::new(0.0, 1.0),
            },
            // Right face (X+)
            Vertex {
                position: Vec3::new(0.5, -0.5, 0.5),
                normal: Vec3::new(1.0, 0.0, 0.0),
                uv: Vec2::new(0.0, 0.0),
            },
            Vertex {
                position: Vec3::new(0.5, -0.5, -0.5),
                normal: Vec3::new(1.0, 0.0, 0.0),
                uv: Vec2::new(1.0, 0.0),
            },
            Vertex {
                position: Vec3::new(0.5, 0.5, -0.5),
                normal: Vec3::new(1.0, 0.0, 0.0),
                uv: Vec2::new(1.0, 1.0),
            },
            Vertex {
                position: Vec3::new(0.5, 0.5, 0.5),
                normal: Vec3::new(1.0, 0.0, 0.0),
                uv: Vec2::new(0.0, 1.0),
            },
            // Left face (X-)
            Vertex {
                position: Vec3::new(-0.5, -0.5, -0.5),
                normal: Vec3::new(-1.0, 0.0, 0.0),
                uv: Vec2::new(0.0, 0.0),
            },
            Vertex {
                position: Vec3::new(-0.5, -0.5, 0.5),
                normal: Vec3::new(-1.0, 0.0, 0.0),
                uv: Vec2::new(1.0, 0.0),
            },
            Vertex {
                position: Vec3::new(-0.5, 0.5, 0.5),
                normal: Vec3::new(-1.0, 0.0, 0.0),
                uv: Vec2::new(1.0, 1.0),
            },
            Vertex {
                position: Vec3::new(-0.5, 0.5, -0.5),
                normal: Vec3::new(-1.0, 0.0, 0.0),
                uv: Vec2::new(0.0, 1.0),
            },
        ];

        #[rustfmt::skip]
        let indices = vec![
            0, 1, 2, 2, 3, 0,       // Front
            4, 5, 6, 6, 7, 4,       // Back
            8, 9, 10, 10, 11, 8,    // Top
            12, 13, 14, 14, 15, 12, // Bottom
            16, 17, 18, 18, 19, 16, // Right
            20, 21, 22, 22, 23, 20, // Left
        ];

        Self { vertices, indices }
    }

    pub fn create_inverted_sphere(radius: f32, segments: u32, rings: u32) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        // Generate vertices
        for ring in 0..=rings {
            let phi = std::f32::consts::PI * ring as f32 / rings as f32;
            let sin_phi = phi.sin();
            let cos_phi = phi.cos();

            for segment in 0..=segments {
                let theta = 2.0 * std::f32::consts::PI * segment as f32 / segments as f32;
                let sin_theta = theta.sin();
                let cos_theta = theta.cos();

                let x = sin_phi * cos_theta;
                let y = cos_phi;
                let z = sin_phi * sin_theta;

                let position = Vec3::new(x * radius, y * radius, z * radius);
                // Inverted normals point inward
                let normal = Vec3::new(-x, -y, -z);
                let uv = Vec2::new(segment as f32 / segments as f32, ring as f32 / rings as f32);

                vertices.push(Vertex {
                    position,
                    normal,
                    uv,
                });
            }
        }

        // Generate indices (reverse winding for inverted sphere)
        for ring in 0..rings {
            for segment in 0..segments {
                let current = ring * (segments + 1) + segment;
                let next = current + segments + 1;

                // Reversed triangle winding for inverted sphere
                indices.push(current);
                indices.push(current + 1);
                indices.push(next);

                indices.push(next);
                indices.push(current + 1);
                indices.push(next + 1);
            }
        }

        Self { vertices, indices }
    }

    pub fn from_obj(path: &str) -> anyhow::Result<Self> {
        let (models, _materials) = tobj::load_obj(
            path,
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
        )?;

        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for model in models {
            let mesh = &model.mesh;

            for i in 0..mesh.positions.len() / 3 {
                let position = Vec3::new(
                    mesh.positions[i * 3],
                    mesh.positions[i * 3 + 1],
                    mesh.positions[i * 3 + 2],
                );

                let normal = if !mesh.normals.is_empty() {
                    Vec3::new(
                        mesh.normals[i * 3],
                        mesh.normals[i * 3 + 1],
                        mesh.normals[i * 3 + 2],
                    )
                } else {
                    Vec3::Y
                };

                let uv = if !mesh.texcoords.is_empty() {
                    Vec2::new(mesh.texcoords[i * 2], 1.0 - mesh.texcoords[i * 2 + 1])
                } else {
                    Vec2::ZERO
                };

                vertices.push(Vertex {
                    position,
                    normal,
                    uv,
                });
            }

            let base_index = indices.len() as u32;
            for &index in &mesh.indices {
                indices.push(base_index + index);
            }
        }

        Ok(Self { vertices, indices })
    }

    /// Create a directional light visualization (arrow pointing in light direction)
    pub fn create_directional_light_viz() -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        // Arrow shaft (cylinder)
        let shaft_radius = 0.05;
        let shaft_length = 1.0;
        let segments = 8;

        // Arrow points down -Y axis (light shines downward by default)
        // Shaft vertices
        for i in 0..=segments {
            let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let x = angle.cos() * shaft_radius;
            let z = angle.sin() * shaft_radius;

            // Top of shaft
            vertices.push(Vertex {
                position: Vec3::new(x, 0.0, z),
                normal: Vec3::new(x, 0.0, z).normalize(),
                uv: Vec2::ZERO,
            });

            // Bottom of shaft
            vertices.push(Vertex {
                position: Vec3::new(x, -shaft_length, z),
                normal: Vec3::new(x, 0.0, z).normalize(),
                uv: Vec2::ZERO,
            });
        }

        // Create shaft indices
        for i in 0..segments {
            let top1 = i * 2;
            let bot1 = i * 2 + 1;
            let top2 = (i + 1) * 2;
            let bot2 = (i + 1) * 2 + 1;

            indices.push(top1);
            indices.push(bot1);
            indices.push(top2);

            indices.push(top2);
            indices.push(bot1);
            indices.push(bot2);
        }

        let base_vertex_count = vertices.len() as u32;

        // Arrow head (cone)
        let cone_radius = 0.15;
        let cone_height = 0.3;
        let cone_base_y = -shaft_length;

        // Cone base vertices
        for i in 0..=segments {
            let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let x = angle.cos() * cone_radius;
            let z = angle.sin() * cone_radius;

            vertices.push(Vertex {
                position: Vec3::new(x, cone_base_y, z),
                normal: Vec3::new(x, 0.5, z).normalize(),
                uv: Vec2::ZERO,
            });
        }

        // Cone tip
        let tip_idx = vertices.len() as u32;
        vertices.push(Vertex {
            position: Vec3::new(0.0, cone_base_y - cone_height, 0.0),
            normal: Vec3::new(0.0, -1.0, 0.0),
            uv: Vec2::ZERO,
        });

        // Create cone indices
        for i in 0..segments {
            let base1 = base_vertex_count + i;
            let base2 = base_vertex_count + i + 1;

            indices.push(base1);
            indices.push(base2);
            indices.push(tip_idx);
        }

        Self { vertices, indices }
    }
}
