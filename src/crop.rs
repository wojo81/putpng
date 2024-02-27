use crate::grab::*;
use image::*;

struct ImageCropper<'a> {
    image: DynamicImage,
    width: u32,
    height: u32,
    path: &'a str,
}

impl<'a> ImageCropper<'a> {
    fn open(path: &'a str) -> Self {
        let image = image::open(path).unwrap();
        let (width, height) = image.dimensions();
        Self { image, width, height, path }
    }

    fn is_visible(&self, x: u32, y: u32) -> bool {
        self.image.get_pixel(x, y).to_rgba().0[3] != 0
    }

    fn left_offset(&self) -> u32 {
        for x in 0..self.width {
            for y in 0..self.height {
                if self.is_visible(x, y) {
                    return x;
                }
            }
        }
        0
    }

    fn right_offset(&self) -> u32 {
        for x in (0..self.width).rev() {
            for y in 0..self.height {
                if self.is_visible(x, y) {
                    return x;
                }
            }
        }
        0
    }

    fn top_offset(&self) -> u32 {
        for y in 0..self.height {
            for x in 0..self.width {
                if self.is_visible(x, y) {
                    return y;
                }
            }
        }
        0
    }

    fn bottom_offset(&self) -> u32 {
        for y in (0..self.height).rev() {
            for x in 0..self.width {
                if self.is_visible(x, y) {
                    return y;
                }
            }
        }
        0
    }

    fn save(&self) -> (i32, i32) {
        let left = self.left_offset();
        let right = self.right_offset();
        let top = self.top_offset();
        let bottom = self.bottom_offset();
        let image = imageops::crop_imm(&self.image, left, top, right - left + 1, bottom - top + 1).to_image();
        std::fs::remove_file(&self.path).unwrap();
        image.save(&self.path).unwrap();
        (left as i32, top as i32)
    }
}

pub fn apply_crop(paths: impl Iterator<Item = String>) {
    let crc = Crc32::new();
    for path in paths {
        let grab_offset = read_grab_offset(&path);
        let crop_offset = ImageCropper::open(&path).save();
        let new_offset = (grab_offset.0 - crop_offset.0, grab_offset.1 - crop_offset.1);
        if new_offset.0 != 0 && new_offset.1 != 0 {
            push_grab_chunk(&path, new_offset.0, new_offset.1, &crc)
        }
        println!("Cropped '{path}' successfully!");
    }
}