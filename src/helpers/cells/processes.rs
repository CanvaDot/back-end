use std::{fs::{File, OpenOptions}, io::{Result as IoResult, Read, Seek, SeekFrom, Write}, sync::OnceLock};
use crate::models::user::User;
use super::{color::Color, position::Position};

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;

pub struct CanvasSpec {
    columns: u32,
    rows: u32,
    pub cells: Vec<u8>
}

impl ToString for CanvasSpec {
    fn to_string(&self) -> String {
        format!(
            "{},{},{}",
            self.columns,
            self.rows,
            self.cells
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join(" ")
        )
    }
}

static CELLS_FILE: OnceLock<File> = OnceLock::new();

fn open_file() -> IoResult<&'static File> {
    if let Some(file) = CELLS_FILE.get() {
        return Ok(file)
    }

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open("cells.bin")?;

    const SIZE: usize = (HEIGHT * WIDTH * 7) as usize;

    if !file.read(&mut [])? != SIZE {
        file.write(&[0u8; SIZE])?;
    }

    Ok(CELLS_FILE.get_or_init(|| file))
}

// this will run every time someone paints in a cell.
// if you return Ok(()) it will continue as intended, otherwise simply send the error with it's
// specified OP code.
//
// On Ok it will consume a token,
// on Err it wont.
pub fn process_written_cell(author: &User, position: Position, color: Color) -> Result<(), String> {
    if position.x() >= WIDTH || position.y() >= HEIGHT {
        return Err("Coordinates out of bounds.".into());
    }

    let mut buffer = vec![color.r(), color.g(), color.b()];
    buffer.extend_from_slice(&author.id().to_le_bytes());

    let mut file = open_file()
        .map_err(|err| err.to_string())?;

    file.seek(SeekFrom::Start(((position.y() * WIDTH + position.x()) as u64) * 7))
        .map_err(|err| err.to_string())?;

    file.write_all(&buffer)
        .map_err(|err| err.to_string())?;

    Ok(())
}

// this will run every time someone opens a connection for the first time.
// if you return Ok(_) it will send the canvas spec, otherwise simply close the connection.
pub fn get_canvas_spec() -> Result<CanvasSpec, String> {
    let mut file = open_file()
        .map_err(|err| err.to_string())?;

    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)
        .map_err(|err| err.to_string())?;

    Ok(CanvasSpec {
        columns: WIDTH,
        rows: HEIGHT,
        cells: buffer
    })
}
