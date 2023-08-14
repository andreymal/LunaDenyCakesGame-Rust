use crate::color::Color;

impl Into<macroquad::prelude::Color> for Color {
    fn into(self) -> macroquad::prelude::Color {
        macroquad::prelude::Color {
            r: self.r as f32 / 255.0,
            g: self.g as f32 / 255.0,
            b: self.b as f32 / 255.0,
            a: self.a as f32 / 255.0,
        }
    }
}
