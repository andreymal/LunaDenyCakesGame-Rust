use crate::vec::Vec2;
use sfml::system::{Vector2f, Vector2u};

impl Into<Vector2f> for Vec2 {
    fn into(self) -> Vector2f {
        Vector2f::new(self.x, self.y)
    }
}

impl From<Vector2f> for Vec2 {
    fn from(value: Vector2f) -> Self {
        Vec2::new(value.x, value.y)
    }
}

impl From<Vector2u> for Vec2 {
    fn from(value: Vector2u) -> Self {
        Vec2::new(value.x as f32, value.y as f32)
    }
}
