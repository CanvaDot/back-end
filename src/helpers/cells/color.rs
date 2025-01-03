use std::num::ParseIntError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ColorError {
    #[error("There are not enough values in this raw split comma string for the specified format.")]
    InvalidValueAmount,

    #[error("Couldn't parse one of the numbers in the raw string.")]
    ParseI32(#[from] ParseIntError),

    #[error("The specified format is invalid, expected v for values or h for hex.")]
    InvalidFormat
}

#[derive(Clone, Copy)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self {
            r,
            g,
            b
        }
    }

    pub fn r(&self) -> u8 {
        self.r
    }

    pub fn g(&self) -> u8 {
        self.g
    }

    pub fn b(&self) -> u8 {
        self.b
    }
}

impl Into<i32> for Color {
    fn into(self) -> i32 {
        ((self.r as i32) << 16)
            | ((self.g as i32) << 8)
            | (self.b as i32)
    }
}

impl From<i32> for Color {
    fn from(value: i32) -> Self {
        Self {
            r: ((value >> 16) & 0xFF) as u8,
            g: ((value >> 8) & 0xFF) as u8,
            b: (value & 0xFF) as u8
        }
    }
}

impl TryFrom<String> for Color {
    type Error = ColorError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let split = value
            .split(",")
            .collect::<Vec<_>>();

        if split.len() <= 1 {
            return Err(ColorError::InvalidValueAmount);
        }

        match split[0] {
            "v" => {
                if split.len() != 4 {
                    return Err(ColorError::InvalidValueAmount);
                }

                Ok(Self::new(
                    split[1].trim().parse()?,
                    split[2].trim().parse()?,
                    split[3].trim().parse()?
                ))
            },

            "h" => {
                if split.len() != 2 {
                    return Err(ColorError::InvalidValueAmount);
                }

                Ok(Self::from(i32::from_str_radix(split[1].trim(), 16)?))
            },

            _ => {
                Err(ColorError::InvalidFormat)
            }
        }
    }
}

impl ToString for Color {
    fn to_string(&self) -> String {
        format!("v,{},{},{}", self.r, self.g, self.b)
    }
}
