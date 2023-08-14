use crate::rect::Rect;

impl Into<sfml::graphics::IntRect> for Rect {
    fn into(self) -> sfml::graphics::IntRect {
        // Добавление дробной части x/y позволяет убрать щели между текстурами
        // (неактуально для SFML, но пусть уж будет для единообразия с SDL)
        sfml::graphics::IntRect::new(
            (self.x.round()) as i32,
            (self.y.round()) as i32,
            (self.width + self.x % 1.0).round() as i32,
            (self.height + self.y % 1.0).round() as i32,
        )
    }
}

impl Into<sfml::graphics::FloatRect> for Rect {
    fn into(self) -> sfml::graphics::FloatRect {
        sfml::graphics::FloatRect::new(self.x, self.y, self.width, self.height)
    }
}
