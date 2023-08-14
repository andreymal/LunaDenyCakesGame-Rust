use crate::rect::Rect;

impl Into<sdl2::rect::Rect> for Rect {
    fn into(self) -> sdl2::rect::Rect {
        // Добавление дробной части x/y позволяет убрать щели между текстурами
        sdl2::rect::Rect::new(
            (self.x.round()) as i32,
            (self.y.round()) as i32,
            (self.width + self.x % 1.0).round() as u32,
            (self.height + self.y % 1.0).round() as u32,
        )
    }
}
