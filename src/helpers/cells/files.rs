use core::str;
use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, BufWriter, Read, Seek, SeekFrom, Write},
    os::unix::fs::FileExt,
    path::Path,
};

use crate::models::user::User;

use super::{color::Color, position::Position, processes::get_canvas_spec};

fn open_file() -> Result<File, String> {
    let create = !Path::new("dots").exists();
    println!("{create}");
    let Ok(mut file) = OpenOptions::new()
        .read(true)
        .write(true)
        .create(create)
        .open("dots")
    else {
        return Err("Cannot open file".to_string());
    };

    let Ok(canvas) = get_canvas_spec() else {
        return Err("Cannot get Canvas Spec".to_string());
    };

    if create == true {
        if file.write_all(&canvas.cells).is_err() {
            return Err("Cannot write to file".to_string());
        } else {
            return Ok(file);
        }
    } else {
        return Ok(file);
    }
}

fn open_file_users() -> Result<File, String> {
    let create = !Path::new("users").exists();

    let Ok(mut file) = OpenOptions::new()
        .read(true)
        .write(true)
        .create(create)
        .open("users")
    else {
        return Err("Cannot open file".to_string());
    };

    let vec = vec![" \n"; 1920 * 1080];

    if create == true {
        if file.write_all(vec.join("").as_bytes()).is_err() {
            return Err("Cannot write to file".to_string());
        } else {
            return Ok(file);
        }
    } else {
        return Ok(file);
    }
}

fn insert_username(user: &str, position: Position, mut file: &File) /*-> Result<usize, String>*/
{
    let mut reader = BufReader::new(file);
    let (i, line) = reader.lines().enumerate().find(|(i, _)| *i == (position.y() * 1920 + position.x()) as usize).unwrap();
    file.seek(SeekFrom::Start((position.y() * 1920 + position.x()) as u64));
    file.write(b"chiwa");
}

fn write_pixel(position: Position, color: Color, file: &File) -> Result<usize, String> {
    let bytes = [color.r(), color.g(), color.b()];
    if let Ok(u) = file.write_at(&bytes, (position.y() * 1920 + position.x() * 3) as u64) {
        return Ok(u);
    } else {
        return Err("Cannot write pixel".to_string());
    }
}

fn get_pixel(position: Position, file: &File) -> Result<[u8; 3], String> {
    let mut buf: [u8; 3] = [0; 3];
    if file
        .read_exact_at(&mut buf, (position.y() * 1920 + position.x() * 3) as u64)
        .is_err()
    {
        return Err("Cannot read pixel".to_string());
    } else {
        return Ok(buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pixels_file() {
        if let Ok(f) = open_file() {
            let w = write_pixel(Position::new(2, 0), Color::new(10, 10, 10), &f);
            let pixel = get_pixel(Position::new(2, 0), &f);
            println!("{w:?}, {pixel:?}")
        }
    }

    #[test]
    fn users_file() {
        if let Ok(f) = open_file_users() {
            insert_username("chiwa", Position::new(3, 0), &f);
        }
    }
}
