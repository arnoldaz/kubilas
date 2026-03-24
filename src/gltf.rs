
use std::{any::Any, time::Instant};

use anyhow::{anyhow, Result};
use cgmath::{Vector2, Vector3};
use gltf::{image::Format};

use crate::{bitmap::Bitmap, mesh::Mesh, vertex::Vertex};


pub struct Gltf {

}

impl Gltf {
    pub fn load() -> Result<(Mesh, Bitmap)> {
        // Load a glTF file with all resources
        let now = Instant::now();
        let (document, buffers, images) = gltf::import("assets/Suzanne.gltf")?;
        let elapsed_time = now.elapsed();
        println!("Running slow_function() took {} seconds.", elapsed_time.as_secs());

        let mut meshes = Vec::new();

        for mesh in document.meshes() {
            println!("Mesh #{}", mesh.index());
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                let material = primitive.material();
                if let Some(texture_info) = material.pbr_metallic_roughness().base_color_texture() {
                    let index = texture_info.texture().index();
                }

                let positions: Vec<[f32; 3]> =
                    reader.read_positions()
                    .ok_or_else(|| anyhow::anyhow!("Missing positions"))?
                    .collect();

                let texcoords: Vec<[f32; 2]> =
                    reader.read_tex_coords(0)
                    .map(|tc| tc.into_f32().collect())
                    .unwrap_or_else(|| vec![[0.0, 0.0]; positions.len()]);

                let vertices: Vec<Vertex> = positions
                    .into_iter()
                    .zip(texcoords)
                    .map(|(p, uv)| Vertex {
                        position: Vector3::new(p[0], p[1], p[2]),
                        color: Vector3::new(1.0, 1.0, 1.0), // glTF model has no vertex colors
                        texture_coordinates: Vector2::new(uv[0], uv[1]),
                    })
                    .collect();

                let indices: Vec<u32> = reader
                    .read_indices()
                    .ok_or_else(|| anyhow::anyhow!("Missing indices"))?
                    .into_u32()
                    .collect();

                println!("{} vert", vertices.len());
                println!("{} ind", indices.len());
                let mesh = Mesh::new(vertices, indices);
                meshes.push(mesh);

                println!("- Primitive #{}", primitive.index());
                for (semantic, _) in primitive.attributes() {
                    println!("-- {:?}", semantic);
                }
            }
        }

        let mut bitmaps = Vec::new();

        for image in images {
            println!("width: {}", image.width);
            println!("height: {}", image.height);
            println!("format: {:?}", image.format);

            let pixels = &image.pixels;
            bitmaps.push(Gltf::get_bitmap_from_gltf_image(&image)?);
        }

        let mesh = meshes.remove(0);
        let bitmap = bitmaps.remove(0);
        Ok((mesh, bitmap))
    }

    fn get_bitmap_from_gltf_image(image: &gltf::image::Data) -> Result<Bitmap> {
        let pixels = match image.format {
            Format::R8G8B8A8 => image.pixels.clone(),
            Format::R8G8B8 => {
                let mut out = Vec::with_capacity((image.width * image.height * 4) as usize);
                for chunk in image.pixels.chunks(3) {
                    out.extend_from_slice(&[chunk[0], chunk[1], chunk[2], 255]);
                }
                out
            }
            _ => return Err(anyhow!("glTF image format not supported"))
        };

        Ok(Bitmap::new(pixels, image.width, image.height))
    }

}