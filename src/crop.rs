use crate::crc::*;
use crate::grab::*;
use image::*;
use std::path::Path;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

struct ImageCropper<'a> {
    image: DynamicImage,
    width: u32,
    height: u32,
    path: &'a Path,
}

impl<'a> ImageCropper<'a> {
    fn open(path: &'a Path) -> Result<Self> {
        let image = image::open(path)?;
        let (width, height) = image.dimensions();
        Ok(Self {
            image,
            width,
            height,
            path,
        })
    }

    fn is_visible(&self, x: u32, y: u32) -> bool {
        let pixel = self.image.get_pixel(x, y).to_rgba();
        pixel[3] != 0
    }

    fn visible_left(&self) -> u32 {
        (0..self.width)
            .find(|x| (0..self.height).any(|y| self.is_visible(*x, y)))
            .unwrap_or(0)
    }

    fn visible_right(&self) -> u32 {
        (0..self.width)
            .rev()
            .find(|x| (0..self.height).any(|y| self.is_visible(*x, y)))
            .unwrap_or(self.width - 1)
    }

    fn visible_top(&self) -> u32 {
        (0..self.height)
            .find(|y| (0..self.width).any(|x| self.is_visible(x, *y)))
            .unwrap_or(0)
    }

    fn visible_bottom(&self) -> u32 {
        (0..self.height)
            .rev()
            .find(|y| (0..self.width).any(|x| self.is_visible(x, *y)))
            .unwrap_or(self.height - 1)
    }

    fn save(&self) -> Result<(i32, i32)> {
        let (left, right, top, bottom) = (
            self.visible_left(),
            self.visible_right(),
            self.visible_top(),
            self.visible_bottom(),
        );
        let image = imageops::crop_imm(&self.image, left, top, right - left + 1, bottom - top + 1)
            .to_image();
        std::fs::remove_file(&self.path)?;
        image.save(&self.path)?;
        Ok((left as i32, top as i32))
    }
}

///Crops the specified png while preserving the relative offset
pub fn crop(path: &Path, crc: &Crc32) -> Result<()> {
    let new_offset = {
        let grab_offset = read_grab(path)?.unwrap_or_default();
        let crop_offset = ImageCropper::open(path)?.save()?;
        (grab_offset.0 - crop_offset.0, grab_offset.1 - crop_offset.1)
    };
    push_grab(&path, crc, new_offset.0, new_offset.1)?;
    Ok(())
}

///Crops all the specified pngs while preserving relative grab offsets
pub fn crop_all<'a>(paths: impl Iterator<Item = &'a Path>, crc: &Crc32) -> Result<()> {
    for path in paths {
        crop(path, crc)?;
        println!("Cropped {path:?} successfully!");
    }
    Ok(())
}
