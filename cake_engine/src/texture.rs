//! Текстура.

use crate::{rect::Rect, vec::Vec2};
use std::path::PathBuf;

/// Откуда была взята текстура.
pub enum TextureSource {
    /// Из ниоткуда (используется, например, при динамической генерации).
    None,
    /// Из байтов, содержащих изображение.
    Data(Vec<u8>),
    /// Из файла в каталоге ассетов.
    File(PathBuf),
    /// Из файла в каталоге ассетов с учётом языка (движок ищет файл, содержащий в имени код
    /// текущего языка).
    LangFile(PathBuf),
}

/// Параметры загрузки текстуры.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TextureOptions {
    /// Использовать ли сглаживание при рисовании текстуры.
    pub smooth: bool,
}

impl TextureOptions {
    pub const PIXELATED: TextureOptions = TextureOptions { smooth: false };
}

impl Default for TextureOptions {
    fn default() -> Self {
        TextureOptions { smooth: true }
    }
}

/// Текстура.
pub struct Texture {
    #[allow(dead_code)]
    pub(crate) id: usize,
    pub(crate) source: TextureSource,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) options: TextureOptions,
}

impl Texture {
    /// Откуда была взята текстура.
    pub fn source(&self) -> &TextureSource {
        &self.source
    }

    /// Ширина текстуры в пикселях.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Высота текстуры в пикселях.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Размер текстуры в пикселях в виде [вектора](crate::vec::Vec2).
    pub fn size_vec(&self) -> Vec2 {
        Vec2::new(self.width as f32, self.height as f32)
    }

    /// Текстура в виде прямоугольника (координаты — нули, размер — размер текстуры).
    pub fn rect(&self) -> Rect {
        Rect::new(0.0, 0.0, self.width as f32, self.height as f32)
    }

    /// Параметры, с которыми была загружена текстура.
    pub fn options(&self) -> &TextureOptions {
        &self.options
    }
}
