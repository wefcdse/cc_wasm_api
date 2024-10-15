use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ColorId {
    White = 0,
    Orange,
    Magenta,
    LightBlue,
    Yellow,
    Lime,
    Pink,
    Gray,
    LightGray,
    Cyan,
    Purple,
    Blue,
    Brown,
    Green,
    Red,
    Black,
}
impl ColorId {
    pub fn to_number(self) -> u16 {
        1 << self as u8
    }

    pub fn to_str(self) -> &'static str {
        match self as u8 {
            0 => "0x1",
            1 => "0x2",
            2 => "0x4",
            3 => "0x8",
            4 => "0x10",
            5 => "0x20",
            6 => "0x40",
            7 => "0x80",
            8 => "0x100",
            9 => "0x200",
            10 => "0x400",
            11 => "0x800",
            12 => "0x1000",
            13 => "0x2000",
            14 => "0x4000",
            15 => "0x8000",
            _ => panic!(),
        }
    }

    pub fn from_number_overflow(num: u32) -> ColorId {
        let num = num % 16;
        let num = num as u8;
        ColorId::from_number_or_panic(num)
    }

    fn from_number_or_panic(num: u8) -> ColorId {
        match num {
            0 => ColorId::White,
            1 => ColorId::Orange,
            2 => ColorId::Magenta,
            3 => ColorId::LightBlue,
            4 => ColorId::Yellow,
            5 => ColorId::Lime,
            6 => ColorId::Pink,
            7 => ColorId::Gray,
            8 => ColorId::LightGray,
            9 => ColorId::Cyan,
            10 => ColorId::Purple,
            11 => ColorId::Blue,
            12 => ColorId::Brown,
            13 => ColorId::Green,
            14 => ColorId::Red,
            15 => ColorId::Black,
            _ => panic!(),
        }
    }
}

impl<T, const N: usize> Index<ColorId> for [T; N] {
    type Output = T;

    fn index(&self, index: ColorId) -> &Self::Output {
        &self[index as usize]
    }
}
impl<T, const N: usize> IndexMut<ColorId> for [T; N] {
    fn index_mut(&mut self, index: ColorId) -> &mut Self::Output {
        &mut self[index as usize]
    }
}
#[allow(unused)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Direction {
    PosX,
    PosY,
    NegX,
    NegY,
}
impl Direction {
    pub fn to_dxdy(self) -> (isize, isize) {
        match self {
            Direction::PosX => (1, 0),
            Direction::PosY => (0, 1),
            Direction::NegX => (-1, 0),
            Direction::NegY => (0, -1),
        }
    }
}

/// as if a pixel, a basic display part of computer craft monitor
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AsIfPixel {
    text: char,
    pub background_color: ColorId,
    pub text_color: ColorId,
}
impl Default for AsIfPixel {
    fn default() -> Self {
        // Self {
        //     text: ' ',
        //     background_color: ColorId::Black,
        //     text_color: ColorId::White,
        // }
        Self::colored_whitespace(ColorId::Black)
    }
}
impl AsIfPixel {
    /// returns `None` if `text` is not within the ASCII range
    pub const fn new(text: char, background_color: ColorId, text_color: ColorId) -> Option<Self> {
        if !text.is_ascii() {
            None
        } else {
            Some(AsIfPixel {
                text,
                background_color,
                text_color,
            })
        }
    }
    pub const fn colored_whitespace(color: ColorId) -> Self {
        AsIfPixel {
            text: ' ',
            background_color: color,
            text_color: color,
        }
    }
    pub fn text(&self) -> char {
        self.text
    }
    pub fn is_whitespace(&self) -> bool {
        (self.background_color == self.text_color) || self.text.is_whitespace()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Side {
    Top,
    Bottom,
    Left,
    Right,
    Front,
    Back,
}
impl Side {
    pub fn name(&self) -> &'static str {
        match self {
            Side::Top => "top",
            Side::Bottom => "bottom",
            Side::Left => "left",
            Side::Right => "right",
            Side::Front => "front",
            Side::Back => "back",
        }
    }
}

impl TryFrom<&str> for Side {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Ok(match value {
            "top" => Self::Top,
            "bottom" => Self::Bottom,
            "left" => Self::Left,
            "right" => Self::Right,
            "front" => Self::Front,
            "back" => Self::Back,
            _ => Err(format!("invalid side:{}", value))?,
        })
    }
}
