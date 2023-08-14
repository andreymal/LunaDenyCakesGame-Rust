use crate::color::Color;

impl Into<sdl2::pixels::Color> for Color {
    fn into(self) -> sdl2::pixels::Color {
        sdl2::pixels::Color::RGBA(self.r, self.g, self.b, self.a)
    }
}
