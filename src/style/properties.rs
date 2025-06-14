use glam::{Vec2, Vec4, vec2};

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Display {
    Block,
    Flex,
    Grid,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum FlexDir {
    Row,
    Column,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Justify {
    Start,
    Center,
    End,
    SpaceBetween,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Align {
    Start,
    Center,
    End,
    Stretch,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Flex {
    pub dir: FlexDir,
    pub justify: Justify,
    pub align: Align,
    pub gap: f32,
    pub fill_cross: bool,
}

impl Default for Flex {
    fn default() -> Self {
        Self {
            dir: FlexDir::Row,
            justify: Justify::Start,
            align: Align::Stretch,
            gap: 0.0,
            fill_cross: false,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Grid {
    pub cols: u16,
    pub row_height: f32,
    pub gap: Vec2,
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            cols: 2,
            row_height: 24.0,
            gap: vec2(4.0, 4.0),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Style {
    pub display: Display,
    pub flex: Flex,
    pub grid: Grid,
    pub flex_grow: f32,
    pub padding: Vec2,
    pub background_color: Option<Vec4>,
    pub width: Option<f32>,
    pub height: Option<f32>,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            display: Display::Block,
            flex: Flex::default(),
            grid: Grid::default(),
            flex_grow: 0.0,
            padding: Vec2::ZERO,
            background_color: None,
            width: None,
            height: None,
        }
    }
}

impl Style {
    pub fn padding_total(self) -> Vec2 {
        self.padding * 2.0
    }

    pub fn padding_tl(self) -> Vec2 {
        self.padding
    }
}
