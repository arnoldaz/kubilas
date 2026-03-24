use std::{collections::HashMap, f32::consts::PI, fs::File, io::BufReader};
use cgmath::{Vector2, Vector3, Zero};
use crate::vertex::Vertex;
use anyhow::Result;

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        Self { vertices, indices }
    }

    pub fn create_from_model(model_name: &str) -> Result<Self> {
        let mut reader = BufReader::new(File::open(model_name)?);
        let (models, _) = tobj::load_obj_buf(
            &mut reader,
            &tobj::LoadOptions { triangulate: true, single_index: true, ..Default::default() },
            |_| Ok(Default::default()),
        )?;

        let mut unique_vertices = HashMap::new();

        let mut vertices = Vec::<Vertex>::new();
        let mut indices = Vec::<u32>::new();

        for model in &models {
            for index in &model.mesh.indices {
                let pos_offset = (3 * index) as usize;
                let tex_coord_offset = (2 * index) as usize;

                let vertex = Vertex {
                    position: Vector3::new(
                        model.mesh.positions[pos_offset],
                        model.mesh.positions[pos_offset + 1],
                        model.mesh.positions[pos_offset + 2],
                    ),
                    color: Vector3::new(1.0, 1.0, 1.0),
                    texture_coordinates: Vector2::new(
                        model.mesh.texcoords[tex_coord_offset],
                        1.0 - model.mesh.texcoords[tex_coord_offset + 1],
                    ),
                };

                if let Some(index) = unique_vertices.get(&vertex) {
                    indices.push(*index as u32);
                } else {
                    let index = vertices.len();
                    unique_vertices.insert(vertex, index);
                    vertices.push(vertex);
                    indices.push(index as u32);
                }
            }
        }

        println!("Loaded model '{model_name}' with {} vertices and {} indices", vertices.len(), indices.len());

        Ok(Self { vertices, indices })
    }

    pub fn default_tetrahedron() -> Self {
        let vertices = vec![
            Vertex::new(Vector3::new( 1.0,  1.0,  1.0), Vector3::new(1.0, 0.0, 0.0), Vector2::zero()),
            Vertex::new(Vector3::new(-1.0, -1.0,  1.0), Vector3::new(0.0, 1.0, 0.0), Vector2::zero()),
            Vertex::new(Vector3::new(-1.0,  1.0, -1.0), Vector3::new(0.0, 0.0, 1.0), Vector2::zero()),
            Vertex::new(Vector3::new( 1.0, -1.0, -1.0), Vector3::new(1.0, 1.0, 0.0), Vector2::zero()),
        ];

        let indices: Vec<u32> = vec![
            0, 2, 1, // base
            0, 1, 3, // side 1
            1, 2, 3, // side 2
            2, 0, 3, // side 3
        ];

        Self { vertices, indices }
    }

    pub fn default_cube() -> Self {
        let vertices = vec![
            Vertex::new(Vector3::new(-1.0, -1.0, -1.0), Vector3::new(1.0, 0.0, 0.0), Vector2::zero()),
            Vertex::new(Vector3::new( 1.0, -1.0, -1.0), Vector3::new(0.0, 1.0, 0.0), Vector2::zero()),
            Vertex::new(Vector3::new( 1.0,  1.0, -1.0), Vector3::new(0.0, 0.0, 1.0), Vector2::zero()),
            Vertex::new(Vector3::new(-1.0,  1.0, -1.0), Vector3::new(1.0, 1.0, 0.0), Vector2::zero()),
            Vertex::new(Vector3::new(-1.0, -1.0,  1.0), Vector3::new(1.0, 0.0, 1.0), Vector2::zero()),
            Vertex::new(Vector3::new( 1.0, -1.0,  1.0), Vector3::new(0.0, 1.0, 1.0), Vector2::zero()),
            Vertex::new(Vector3::new( 1.0,  1.0,  1.0), Vector3::new(1.0, 1.0, 1.0), Vector2::zero()),
            Vertex::new(Vector3::new(-1.0,  1.0,  1.0), Vector3::new(0.5, 0.5, 0.5), Vector2::zero()),
        ];

        let indices: Vec<u32> = vec![
            // Front face (+Z)
            4, 5, 6,
            6, 7, 4,
            // Back face (-Z)
            0, 3, 2,
            2, 1, 0,
            // Left face (-X)
            0, 4, 7,
            7, 3, 0,
            // Right face (+X)
            1, 2, 6,
            6, 5, 1,
            // Top face (+Y)
            3, 7, 6,
            6, 2, 3,
            // Bottom face (-Y)
            0, 1, 5,
            5, 4, 0,
        ];

        Self { vertices, indices }
    }

    pub fn default_sphere() -> Self {
        Self::create_sphere(1.0, 32, 64)
    }

    pub fn create_sphere(radius: f32, stacks: u32, slices: u32) -> Self {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for i in 0..=stacks {
            let phi = PI * i as f32 / stacks as f32;
            let y = radius * phi.cos();
            let r = radius * phi.sin();
            let v = i as f32 / stacks as f32;

            for j in 0..slices {
                let theta = 2.0 * PI * j as f32 / slices as f32;
                let x = r * theta.cos();
                let z = r * theta.sin();
                let u = j as f32 / slices as f32;

                let color = Vector3::new(
                    0.5 + 0.5 * theta.cos(),
                    0.5 + 0.5 * theta.sin(),
                    1.0 - v,
                );

                vertices.push(Vertex::new(
                    Vector3::new(x, y, z),
                    color,
                    Vector2::new(u, v),
                ));
            }
        }

        for i in 0..stacks {
            for j in 0..slices {
                let next = (j + 1) % slices;

                let a = i * slices + j;
                let b = (i + 1) * slices + j;
                let c = i * slices + next;
                let d = (i + 1) * slices + next;

                indices.push(a);
                indices.push(c);
                indices.push(b);

                indices.push(b);
                indices.push(c);
                indices.push(d);
            }
        }

        Self { vertices, indices }
    }
}