//! Шрифт.

use std::path::{Path, PathBuf};

/// Шрифт.
pub struct Font {
    #[allow(dead_code)]
    pub(crate) id: usize,
    pub(crate) path: PathBuf,
    pub(crate) size: u16,
}

impl Font {
    /// Возвращает путь к файлу, из которого был прочитан шрифт.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Возвращает размер шрифта, с которым он был загружен.
    pub fn size(&self) -> u16 {
        self.size
    }
}
