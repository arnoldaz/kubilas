use std::fs::File;
use png::{ColorType, Decoder};
use anyhow::Result;

pub struct Bitmap {
    pub pixels: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

impl Bitmap {
    pub fn new(pixels: Vec<u8>, width: u32, height: u32) -> Self {
        Self { pixels, width, height }
    }

    pub fn single_color(color: [u8; 4]) -> Self {
        Self { pixels: color.to_vec(), width: 1, height: 1 }
    }

    pub fn white() -> Self {
        Self::single_color([255, 255, 255, 255])
    }

    pub fn create_from_file(image_name: &str) -> Result<Self> {
        let image = File::open(image_name)?;
        let decoder = Decoder::new(image);
        let mut reader = decoder.read_info()?;

        let mut pixels = vec![0; reader.info().raw_bytes()];
        reader.next_frame(&mut pixels)?;
        let (width, height) = reader.info().size();

        let color_type = reader.info().color_type;
        if color_type != ColorType::Rgba {
            panic!("Invalid texture image '{image_name}'");
        }

        Ok(Self { pixels, width, height })
    }

    pub fn save_png(path: &str, width: u32, height: u32,  pixels: &[u8]) -> Result<()> {
        let file = File::create(path)?;
        let buffer_writer = std::io::BufWriter::new(file);

        let mut encoder = png::Encoder::new(buffer_writer, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);

        let mut writer = encoder.write_header()?;
        writer.write_image_data(pixels)?;

        Ok(())
    }

}

