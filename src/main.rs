use std::fs::*;
use std::io::*;
use clap::{Parser, Subcommand};

struct Crc32 {
    table: [u32; 256],
}

impl Crc32 {
    fn new() -> Self {
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

    fn calculate(&self, bytes: &[u8]) -> u32 {
        let mut answer = u32::MAX;
        bytes.iter().for_each(|byte| answer = self.table[(answer as usize ^ *byte as usize) & 0xff] ^ ((answer >> 8) & 0xffffff));
        !answer
    }
}

fn read_chunk_header(file: &mut File) -> (u32, [u8; 4]) {
    let mut buffer: [u8; 4] = Default::default();
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

fn create_grab_chunk(crc: &Crc32, x: u32, y: u32) -> Vec<u8> {
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

fn insert_grab_chunk(file: &mut File, seek: SeekFrom, crc: &Crc32, x: u32, y: u32) {
    write_into(file, seek, &create_grab_chunk(crc, x, y))
}

fn change_grab_to(file_name: &String, x: u32, y: u32, crc: &Crc32) {
    let mut file = File::options().read(true).write(true).open(file_name).unwrap();

    file.seek(SeekFrom::Start(8)).unwrap();
    let mut ihdr_length = 0;

    loop {
        let (length, name) = read_chunk_header(&mut file);
        if name == "IHDR".as_bytes() {
            ihdr_length = length;
        } else if name == "grAb".as_bytes() {
            file.write(&x.to_be_bytes()).unwrap();
            file.write(&y.to_be_bytes()).unwrap();
            file.write(&crc.calculate(&["grAb".as_bytes(), &x.to_be_bytes(), &y.to_be_bytes()].concat()).to_be_bytes()).unwrap();
            return;
        } else if name == "IDAT".as_bytes() {
            let seek = SeekFrom::Start(ihdr_length as u64 + 20);
            insert_grab_chunk(&mut file, seek, &crc, x, y);
            return;
        }
        file.seek(SeekFrom::Current(length as i64 + 4)).unwrap();
    }
}

fn apply_grab<'a>(file_names: impl Iterator<Item=&'a String>, x: u32, y: u32) {
    let crc = Crc32::new();
    for file_name in file_names {
        change_grab_to(file_name, x, y, &crc);
        println!("Grabbed '{file_name}' successfully!");
    }
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long)]
    ignore: Vec<String>,
}

#[derive(Subcommand)]
enum Commands {
    Grab {x: u32, y: u32, files: Vec<String>}
}

impl Cli {
    fn keep_file(&self, file_name: &String) -> bool {
        for ignored_name in self.ignore.iter() {
            if file_name.contains(ignored_name) {
                return false;
            }
        }
        true
    }
}

fn main() {
    let cli = Cli::parse_from(wild::args());

    match cli.command {
        Commands::Grab { x, y, ref files } => {
            apply_grab(files.iter().filter(|e| cli.keep_file(e)), x, y);
        }
    }
}