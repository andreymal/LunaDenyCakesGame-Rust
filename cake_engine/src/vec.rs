//! Вектор.

use std::ops::Mul;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Вектор. Точка на плоскости, если угодно.
#[derive(Copy, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const fn new(x: f32, y: f32) -> Vec2 {
        Vec2 { x, y }
    }
}

impl From<(i32, i32)> for Vec2 {
    fn from(value: (i32, i32)) -> Self {
        Vec2::new(value.0 as f32, value.1 as f32)
    }
}

impl From<(u32, u32)> for Vec2 {
    fn from(value: (u32, u32)) -> Self {
        Vec2::new(value.0 as f32, value.1 as f32)
    }
}

impl Mul<f32> for Vec2 {
    type Output = Vec2;

    fn mul(self, rhs: f32) -> Self::Output {
        Vec2::new(self.x * rhs, self.y * rhs)
    }
}
