use std::num::ParseIntError;
use thiserror::Error;


#[derive(Error, Debug)]
pub enum PositionError {
    #[error("The length of the raw split coma string wasn't two.")]
    ParseLength,

    #[error("Couldn't parse one of the numbers in the raw string.")]
    ParseI32(#[from] ParseIntError)
}

#[derive(Clone, Copy)]
pub struct Position {
    x: i32,
    y: i32
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self {
            x,
            y
        }
    }

    pub fn x(&self) -> i32 {
        self.x
    }

    pub fn y(&self) -> i32 {
        self.y
    }
}

impl TryFrom<String> for Position {
    type Error = PositionError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let split = value
            .split(",")
            .collect::<Vec<_>>();

        if split.len() != 2 {
            return Err(PositionError::ParseLength);
        }

        Ok(Self::new(
            split[0].trim().parse()?,
            split[1].trim().parse()?
        ))
    }
}

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl ToString for Position {
    fn to_string(&self) -> String {
        format!("{},{}", self.x, self.y)
    }
}
