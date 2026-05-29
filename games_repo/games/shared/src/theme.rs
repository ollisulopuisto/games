use macroquad::prelude::Color;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BlockColor {
    ColorA,
    ColorB,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BlockShape {
    Square,
    Circle,
    Diamond,
    Cross,
}

#[derive(Clone, Debug)]
pub struct Theme {
    pub name: String,
    pub color_a: Color,
    pub color_b: Color,
    pub bg_color: Color,
    pub ui_accent: Color,
    pub bpm: f32,
    pub shape_a: BlockShape,
    pub shape_b: BlockShape,
}

impl Theme {
    pub fn get_color(&self, color: BlockColor) -> Color {
        match color {
            BlockColor::ColorA => self.color_a,
            BlockColor::ColorB => self.color_b,
        }
    }

    pub fn get_shape(&self, color: BlockColor) -> BlockShape {
        match color {
            BlockColor::ColorA => self.shape_a,
            BlockColor::ColorB => self.shape_b,
        }
    }
}

pub struct ThemeEngine {
    pub themes: Vec<Theme>,
    pub current_theme_idx: usize,
}

impl ThemeEngine {
    pub fn new(themes: Vec<Theme>) -> Self {
        assert!(
            !themes.is_empty(),
            "ThemeEngine must be initialized with at least one theme"
        );
        Self {
            themes,
            current_theme_idx: 0,
        }
    }

    pub fn current(&self) -> &Theme {
        &self.themes[self.current_theme_idx]
    }

    pub fn get_suggested_theme_idx(&self, level: u32) -> usize {
        // Lumines Challenge Mode progression:
        // First 15 skins: 4 levels each (60 levels total)
        // Next 9 skins: 5 levels each (45 levels total)
        // Total loop: 105 levels
        let skin_num = if level <= 60 {
            (level.saturating_sub(1) / 4) as usize
        } else if level <= 105 {
            15 + ((level.saturating_sub(61) / 5) as usize)
        } else {
            // Loop back after level 105
            let loop_level = (level.saturating_sub(106) % 105) + 1;
            return self.get_suggested_theme_idx(loop_level);
        };

        skin_num % self.themes.len()
    }
}
