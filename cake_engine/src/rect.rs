//! Прямоугольник.

use crate::vec::Vec2;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Прямоугольник.
///
/// `x` и `y` задают координаты левого верхнего угла.
///
/// Ширина и/или высота могут быть отрицательными, тогда прямоугольник становится перевёрнутым
/// и угол становится правым и/или нижним соответственно. При рендеринге текстур это проявляется
/// в виде соответственно перевёрнутой текстуры.
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    /// Создаёт новый прямоугольник.
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Rect {
        Rect {
            x,
            y,
            width,
            height,
        }
    }

    /// Позиция прямоугольника в виде [вектора](crate::vec::Vec2).
    pub fn get_position(&self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    /// Задаёт позицию прямоугольника из [вектора](crate::vec::Vec2).
    pub fn set_position(&mut self, position: Vec2) {
        self.x = position.x;
        self.y = position.y;
    }

    /// Размер прямоугольника в виде [вектора](crate::vec::Vec2) (`x` — ширина, `y` — высота).
    pub fn get_size(&self) -> Vec2 {
        Vec2::new(self.width, self.height)
    }

    /// Задаёт размер прямоугольника из [вектора](crate::vec::Vec2) (`x` — ширина, `y` — высота).
    pub fn set_size(&mut self, size: Vec2) {
        self.width = size.x;
        self.height = size.y;
    }

    /// Координаты центра прямоугольника.
    pub fn get_center(&self) -> Vec2 {
        Vec2::new(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    /// Является ли прямоугольник вырожденным (ширина и/или высота — ноль).
    pub fn is_degenerate(&self) -> bool {
        self.width == 0.0 || self.height == 0.0
    }

    /// Выворачивает прямоугольник по горизонтали.
    pub fn flip_x(&mut self) {
        self.x += self.width;
        self.width = -self.width;
    }

    /// Выворачивает прямоугольник по вертикали.
    pub fn flip_y(&mut self) {
        self.y += self.height;
        self.height = -self.height;
    }

    /// Нормализует размеры прямоугольника так, чтобы они стали положительными, не меняя
    /// фактическую позицию прямоугольника.
    pub fn normalize(&mut self) {
        if self.width.is_sign_negative() {
            self.x += self.width;
            self.width = self.width.abs();
        }
        if self.height.is_sign_negative() {
            self.y += self.height;
            self.height = self.height.abs();
        }
    }

    /// Возвращает нормализованную копию прямоугольника.
    pub fn normalized(&self) -> Rect {
        let mut r = self.clone();
        r.normalize();
        r
    }

    /// Возвращает `true`, если указанная точка находится внутри прямоугольника или на его границе.
    pub fn contains_point(&self, point: Vec2) -> bool {
        let contains_x = if self.width >= 0.0 {
            point.x >= self.x && point.x < self.x + self.width
        } else {
            point.x >= self.x + self.width && point.x < self.x
        };

        let contains_y = if self.height >= 0.0 {
            point.y >= self.y && point.y < self.y + self.height
        } else {
            point.y >= self.y + self.height && point.y < self.y
        };

        contains_x && contains_y
    }
}
