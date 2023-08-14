//! Параметры, используемые при инициализации игры.

use crate::{rect::Rect, vec::Vec2};
use std::path::PathBuf;

/// Пути к значкам 16x16, 32x32 и 64x64 соответственно.
///
/// Какие значки реально будут использованы, зависит от бэкенда.
#[derive(Clone, Debug)]
pub struct WindowIcon {
    pub path16: PathBuf,
    pub path32: PathBuf,
    pub path64: PathBuf,
}

/// Параметры, используемые при инициализации игры.
///
/// Используются бэкендом в момент запуска игры. Впоследствии вы можете изменить некоторые
/// из этих настроек на лету через методы в [`Context`](crate::context::Context).
#[derive(Clone, Debug)]
pub struct Conf {
    /// Заголовок окна.
    pub title: String,
    /// Значок окна.
    pub icon: Option<WindowIcon>,
    /// Размер окна в логических единицах измерения (без учёта view, но с учётом HiDPI, если
    /// поддерживается бэкендом). Если HiDPI отсутствует, то единицей измерения являются
    /// обычные пиксели.
    pub logical_size: Vec2,
    /// Задаёт систему координат игры: координаты левого верхнего угла, ширина и высота.
    /// Подробнее в [документации View](crate::view).
    pub view: Option<Rect>,
    /// Полноэкранный режим.
    pub fullscreen: bool,
    /// Разрешить пользователю изменять размер окна.
    pub resizable: bool,
    /// Включить или отключить вертикальную синхронизацию, если это поддерживается бэкендом.
    pub vsync: bool,
    /// Ограничение частоты кадров, ноль отключает ограничение.
    pub fps_limit: f32,
    /// Видимость курсора мыши.
    pub mouse_cursor_visible: bool,
    /// Преобразовывать события касания в события мыши (сами события касания при этом тоже
    /// остаются).
    pub simulate_mouse_with_touch: bool,
}

impl Default for Conf {
    fn default() -> Conf {
        Conf {
            title: String::new(),
            icon: None,
            logical_size: Vec2::new(800.0, 600.0),
            view: None,
            fullscreen: false,
            resizable: true,
            vsync: true,
            fps_limit: 0.0,
            mouse_cursor_visible: true,
            simulate_mouse_with_touch: true,
        }
    }
}
