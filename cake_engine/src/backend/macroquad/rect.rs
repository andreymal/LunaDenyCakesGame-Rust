use crate::rect::Rect;

impl Into<macroquad::prelude::Rect> for Rect {
    fn into(self) -> macroquad::prelude::Rect {
        macroquad::prelude::Rect {
            x: self.x,
            y: self.y,
            w: self.width,
            h: self.height,
        }
    }
}
