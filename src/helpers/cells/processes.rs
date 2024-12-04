use super::{color::Color, position::Position};

pub struct CanvasSpec<'r> {
    columns: i32,
    rows: i32,
    cells: &'r [u8]
}

impl<'r> ToString for CanvasSpec<'r> {
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

// this will run every time someone paints in a cell.
// if you return Ok(()) it will continue as intended, otherwise simply send the error with it's
// specified OP code.
pub fn process_written_cell(_position: Position, _color: Color) -> Result<(), String> {
    Ok(())
}

// this will run every time someone opens a connection for the first time.
// if you return Ok(_) it will send the canvas spec, otherwise simply close the connection.
pub fn get_canvas_spec<'r>() -> Result<CanvasSpec<'r>, String> {
    Ok(CanvasSpec {
        columns: 0,
        rows: 0,
        cells: &[0; 0]
    })
}
