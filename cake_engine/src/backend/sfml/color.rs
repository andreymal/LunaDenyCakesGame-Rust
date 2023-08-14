use crate::color::Color;

impl Into<sfml::graphics::Color> for Color {
    fn into(self) -> sfml::graphics::Color {
        sfml::graphics::Color::rgba(self.r, self.g, self.b, self.a)
    }
}
