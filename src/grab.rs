use std::fs::*;
use std::io::*;
use image::*;
use crate::calc;

pub struct Crc32 {
    table: [u32; 256],
}

impl Crc32 {
    pub fn new() -> Self {
        Self {table: core::array::from_fn(|i| {
            let mut e = i as u32;
            for _ in 0..8 {
                if e & 1 == 1 {
                    e = 0xedb88320 ^ ((e >> 1) & 0x7fffffff);
                } else {
                    e = (e >> 1) & 0x7fffffff;
                }
            }
            e
        })}
    }

    pub fn calculate(&self, bytes: &[u8]) -> u32 {
        let mut answer = u32::MAX;
        bytes.iter().for_each(|byte| answer = self.table[(answer as usize ^ *byte as usize) & 0xff] ^ ((answer >> 8) & 0xffffff));
        !answer
    }
}

const default_grab_seek: std::io::SeekFrom = SeekFrom::Start(33);

fn read_chunk_header(file: &mut File) -> (u32, [u8; 4]) {
    let mut buffer = <[u8; 4]>::default();
    file.read(&mut buffer).unwrap();
    let length = u32::from_be_bytes(buffer);
    file.read(&mut buffer).unwrap();
    (length, buffer)
}

fn write_into(file: &mut File, seek: SeekFrom, data: &[u8]) {
    let mut buffer = Vec::new();
    file.seek(seek).unwrap();
    file.read_to_end(&mut buffer).unwrap();
    file.seek(seek).unwrap();
    file.write(data).unwrap();
    file.write(&buffer).unwrap();
}

fn create_grab_chunk(crc: &Crc32, x: i32, y: i32) -> Vec<u8> {
    let body = [
        "grAb".as_bytes(),
        &x.to_be_bytes(),
        &y.to_be_bytes()
    ].concat();

    let body: &[u8] = &body;

    [
        &8u32.to_be_bytes(),
        body,
        &crc.calculate(body).to_be_bytes()
    ].concat()
}

pub fn insert_grab_chunk(file: &mut File, seek: SeekFrom, crc: &Crc32, x: i32, y: i32) {
    write_into(file, seek, &create_grab_chunk(crc, x, y))
}

fn change_grab_to(path: &str, x: i32, y: i32, crc: &Crc32) {
    let mut file = File::options().read(true).write(true).open(path).unwrap();

    file.seek(default_grab_seek).unwrap();
    let (mut length, mut name) = read_chunk_header(&mut file);

    while name != "IDAT".as_bytes() {
        if name == "grAb".as_bytes() {
            let offset: &[u8] = &[x.to_be_bytes(), y.to_be_bytes()].concat();
            file.write(&[offset, &crc.calculate(&[&name, offset].concat()).to_be_bytes()].concat()).unwrap();
            return;
        }
        file.seek(SeekFrom::Current(length as i64 + 4)).unwrap();
        (length, name) = read_chunk_header(&mut file);
    }
    insert_grab_chunk(&mut file, default_grab_seek, &crc, x, y);
}

pub fn read_grab_offset(path: &str) -> Option<(i32, i32)> {
    let mut file = File::open(path).unwrap();

    file.seek(default_grab_seek).unwrap();
    let (mut length, mut name) = read_chunk_header(&mut file);

    while name != "IDAT".as_bytes() {
        if name == "grAb".as_bytes() {
            let mut buffer = <[u8; 4]>::default();
            file.read(&mut buffer).unwrap();
            let x = i32::from_be_bytes(buffer);
            file.read(&mut buffer).unwrap();
            let y = i32::from_be_bytes(buffer);
            return Some((x, y))
        }
        file.seek(SeekFrom::Current(length as i64 + 4)).unwrap();
        (length, name) = read_chunk_header(&mut file);
    }

    None
}

pub fn push_grab_chunk(path: &str, x: i32, y: i32, crc: &Crc32) {
    let mut file = File::options().read(true).write(true).open(path).unwrap();
    insert_grab_chunk(&mut file, default_grab_seek, crc, x, y);
}

pub fn apply_grab(paths: impl Iterator<Item = String>, x: String, y: String) {
    let crc = Crc32::new();

    for path in paths {
        println!("{path:?}");
        let (width, height) = image::open(path.clone()).unwrap().dimensions();
        let width = width as i32;
        let height = height as i32;
        let (x, y) = (calc::eval(&x, width, height).unwrap(), calc::eval(&y, width, height).unwrap());
        change_grab_to(&path, x, y, &crc);
        println!("Grabbed '{path}' successfully!");
    }
}