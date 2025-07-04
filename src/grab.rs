use crate::calc;
use crate::crc::*;
use image::*;
use std::fs::*;
use std::io::*;
use std::path::Path;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const default_grab_seek: std::io::SeekFrom = SeekFrom::Start(33);

fn read_header(file: &mut File) -> Result<(u32, [u8; 4])> {
    let mut buffer = <[u8; 4]>::default();
    file.read(&mut buffer)?;
    let length = u32::from_be_bytes(buffer);
    file.read(&mut buffer)?;
    Ok((length, buffer))
}

fn write_into(file: &mut File, seek: SeekFrom, data: &[u8]) -> Result<()> {
    let mut buffer = Vec::new();
    file.seek(seek)?;
    file.read_to_end(&mut buffer)?;
    file.seek(seek)?;
    file.write(data)?;
    file.write(&buffer)?;
    Ok(())
}

fn create_grab(crc: &Crc32, x: i32, y: i32) -> Vec<u8> {
    let body: &[u8] = &["grAb".as_bytes(), &x.to_be_bytes(), &y.to_be_bytes()].concat();
    [
        &8u32.to_be_bytes(),
        body,
        &crc.calculate(body).to_be_bytes(),
    ]
    .concat()
}

fn insert_grab(file: &mut File, crc: &Crc32, x: i32, y: i32) -> Result<()> {
    Ok(write_into(
        file,
        default_grab_seek,
        &create_grab(crc, x, y),
    )?)
}

///Tries to read the grab chunk if there is one
pub fn read_grab(path: &Path) -> Result<Option<(i32, i32)>> {
    let mut file = File::open(path)?;

    file.seek(default_grab_seek)?;
    let (mut length, mut name) = read_header(&mut file)?;

    while name != "IDAT".as_bytes() {
        if name == "grAb".as_bytes() {
            let mut buffer = <[u8; 4]>::default();
            file.read(&mut buffer)?;
            let x = i32::from_be_bytes(buffer);
            file.read(&mut buffer)?;
            let y = i32::from_be_bytes(buffer);
            return Ok(Some((x, y)));
        }
        file.seek(SeekFrom::Current(length as i64 + 4))?;
        (length, name) = read_header(&mut file)?;
    }

    Ok(None)
}

///Adds a new grab chunk to the specified png (will have duplicate grab chunks if there is already a grab chunk)
pub fn push_grab(path: &Path, crc: &Crc32, x: i32, y: i32) -> Result<()> {
    let mut file = File::options().read(true).write(true).open(path)?;
    insert_grab(&mut file, crc, x, y)?;
    Ok(())
}

///Adds or overwrites a grab chunck to the specified png
pub fn grab(path: &Path, crc: &Crc32, x: i32, y: i32) -> Result<()> {
    let mut file = File::options().read(true).write(true).open(path)?;

    file.seek(default_grab_seek)?;
    let (mut length, mut name) = read_header(&mut file)?;

    while name != "IDAT".as_bytes() {
        if name == "grAb".as_bytes() {
            let offset: &[u8] = &[x.to_be_bytes(), y.to_be_bytes()].concat();
            file.write(
                &[
                    offset,
                    &crc.calculate(&[&name, offset].concat()).to_be_bytes(),
                ]
                .concat(),
            )?;
            return Ok(());
        }
        file.seek(SeekFrom::Current(length as i64 + 4))?;
        (length, name) = read_header(&mut file)?;
    }
    insert_grab(&mut file, &crc, x, y)?;
    Ok(())
}

///Adds grab chunks to the specified pngs using either the `push_grab` or `grab` functions based on `should_push`
pub fn grab_all<'a>(
    paths: impl Iterator<Item = &'a Path>,
    crc: &Crc32,
    source_x: &str,
    source_y: &str,
    should_push: bool,
) -> Result<()> {
    macro_rules! error {
        ($($arg:tt)*) => {
            return Err(format!($($arg)*).into())
        };
    }

    let grab_fn = match should_push {
        true => push_grab,
        false => grab,
    };

    let width_or_height = |c| c == 'w' || c == 'h' || c == 'W' || c == 'H';
    let get_dimensions = |path| -> Result<(i32, i32)> {
        let (w, h) = image::open(path)?.dimensions();
        Ok((w as i32, h as i32))
    };

    match (
        source_x.contains(width_or_height),
        source_y.contains(width_or_height),
    ) {
        (true, true) => {
            for path in paths.into_iter() {
                let (w, h) = get_dimensions(path)?;
                match (calc::eval(&source_x, w, h), calc::eval(&source_y, w, h)) {
                    (Ok(x), Ok(y)) => {
                        grab_fn(&path, &crc, x, y)?;
                        println!("grabbed {path:?} successfully at ({x}, {y})!");
                    }
                    (Err(e1), Err(e2)) => error!(
                        "error in '{source_x}' for {path:?}: {e1}\nerror in '{source_y:}' for {path:?}: {e2}"
                    ),
                    (Err(e), _) => error!("error in '{source_x}' for '{path:?}': {e}"),
                    (_, Err(e)) => error!("error in '{source_y}' for '{path:?}': {e}"),
                }
            }
        }
        (false, false) => match (calc::eval(&source_x, 0, 0), calc::eval(&source_y, 0, 0)) {
            (Ok(x), Ok(y)) => {
                for path in paths {
                    grab_fn(&path, &crc, x, y)?;
                    println!("grabbed {path:?} successfully at ({x}, {y})!");
                }
            }
            (Err(e1), Err(e2)) => {
                error!("error in '{source_x}': {e1}\nerror in '{source_y}': {e2}")
            }
            (Err(e), _) => error!("error in '{source_x}': {e}"),
            (_, Err(e)) => error!("error in '{source_y}': {e}"),
        },
        (true, false) => match calc::eval(&source_y, 0, 0) {
            Ok(y) => {
                for path in paths {
                    let (w, h) = get_dimensions(&path)?;
                    match calc::eval(&source_x, w, h) {
                        Ok(x) => {
                            grab_fn(&path, &crc, x, y)?;
                            println!("grabbed {path:?} successfully at ({x}, {y})!");
                        }
                        Err(e) => error!("error in '{source_x}' for {path:?}: {e}"),
                    }
                }
            }
            Err(e) => {
                if let Some(path) = paths.into_iter().next() {
                    let (w, h) = get_dimensions(&path)?;
                    if let Err(e) = calc::eval(&source_x, w, h) {
                        eprintln!("error in '{source_x}' for {path:?}: {e}");
                    }
                }
                error!("error in '{source_y}': {e}");
            }
        },
        (false, true) => match calc::eval(&source_x, 0, 0) {
            Ok(x) => {
                for path in paths {
                    let (w, h) = get_dimensions(&path)?;
                    match calc::eval(&source_y, w, h) {
                        Ok(y) => {
                            grab_fn(&path, &crc, x, y)?;
                            println!("grabbed {path:?} successfully at ({x}, {y})!");
                        }
                        Err(e) => error!("error in '{source_y}' for {path:?}: {e}"),
                    }
                }
            }
            Err(e) => {
                eprintln!("error in '{source_x}': {e}");
                if let Some(path) = paths.into_iter().next() {
                    let (w, h) = get_dimensions(&path)?;
                    if let Err(e) = calc::eval(&source_y, w, h) {
                        error!("error in '{source_y}' for {path:?}: {e}");
                    }
                }
            }
        },
    }

    Ok(())
}
