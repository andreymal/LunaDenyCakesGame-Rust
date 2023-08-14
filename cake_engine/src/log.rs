//! Макросы для кроссплатформенной записи сообщений в журнал.
//!
//! Впрочем, на большинстве платформ «журналом» является обычный stdout.
//!
//! Макросы поддерживают форматирование и действуют как `format!`.
//!
//! # Examples
//!
//! ```
//! use cake_engine::{log::info, vec::Vec2};
//! let pos = Vec2::new(3.0, 4.0);
//! info!("Координаты: {:?}", pos);
//! ```

// Вот эти объявления макросов нужны только для того, чтобы они нормально отображались
// в документации, а реальная реализация берётся из текущего бэкенда
// (Впрочем, они всё равно отображаются кривовато... Ну да ладно)

#[macro_export]
macro_rules! cake_engine_error {
    ($($arg:tt)+) => (
        $crate::backend::log::error!($($arg)+);
    )
}

#[macro_export]
macro_rules! cake_engine_warn {
    ($($arg:tt)+) => (
        $crate::backend::log::warn!($($arg)+);
    )
}

#[macro_export]
macro_rules! cake_engine_info {
    ($($arg:tt)+) => (
        $crate::backend::log::info!($($arg)+);
    )
}

#[macro_export]
macro_rules! cake_engine_debug {
    ($($arg:tt)+) => (
        $crate::backend::log::debug!($($arg)+);
    )
}

#[macro_export]
macro_rules! cake_engine_trace {
    ($($arg:tt)+) => (
        $crate::backend::log::trace!($($arg)+);
    )
}

pub use cake_engine_debug as debug;
pub use cake_engine_error as error;
pub use cake_engine_info as info;
pub use cake_engine_trace as trace;
pub use cake_engine_warn as warn;
