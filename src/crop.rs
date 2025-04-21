use crate::grab::*;
use image::*;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

struct ImageCropper<'a> {
    image: DynamicImage,
    width: u32,
    height: u32,
    path: &'a str,
}

impl<'a> ImageCropper<'a> {
    fn open(path: &'a str) -> Result<Self> {
        let image = image::open(path)?;
        let (width, height) = image.dimensions();
        Ok(Self { image, width, height, path })
    }

    fn is_visible(&self, x: u32, y: u32) -> bool {
        let pixel = self.image.get_pixel(x, y).to_rgba();
        pixel[0] != 0 && pixel[1] != 0 && pixel[2] != 0 && pixel[3] != 0
    }

    fn left_offset(&self) -> u32 {
        for x in 0..self.width {
            if (0..self.height).any(|y| self.is_visible(x, y)) {
                return x;
            }
        }
        0
    }

    fn right_offset(&self) -> u32 {
        for x in (0..self.width).rev() {
            if (0..self.height).any(|y| self.is_visible(x, y)) {
                return x;
            }
        }
        self.width - 1
    }

    fn top_offset(&self) -> u32 {
        for y in 0..self.height {
            if (0..self.width).any(|x| self.is_visible(x, y)) {
                return y;
            }
        }
        0
    }

    fn bottom_offset(&self) -> u32 {
        for y in (0..self.height).rev() {
            if (0..self.width).any(|x| self.is_visible(x, y)) {
                return y;
            }
        }
        self.height - 1
    }

    fn save(&self) -> Result<(i32, i32)> {
        let left = self.left_offset();
        let right = self.right_offset();
        let top = self.top_offset();
        let bottom = self.bottom_offset();
        let image = imageops::crop_imm(&self.image, left, top, right - left + 1, bottom - top + 1).to_image();
        std::fs::remove_file(&self.path)?;
        image.save(&self.path)?;
        Ok((left as i32, top as i32))
    }
}

pub fn apply_crop(paths: impl Iterator<Item = String>) -> Result<()> {
    let crc = Crc32::new();
    for path in paths {
        let grab_offset = read_grab_offset(&path)?.unwrap_or_default();
        let crop_offset = ImageCropper::open(&path)?.save()?;
        let new_offset = (grab_offset.0 - crop_offset.0, grab_offset.1 - crop_offset.1);
        push_grab_chunk(&path, new_offset.0, new_offset.1, &crc)?;
        println!("Cropped '{path}' successfully!");
    }
    Ok(())
}