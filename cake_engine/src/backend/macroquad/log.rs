#[macro_export]
macro_rules! macroquad_error {
    ($($arg:tt)+) => (
        macroquad::logging::error!($($arg)+);
    )
}

#[macro_export]
macro_rules! macroquad_warn {
    ($($arg:tt)+) => (
        macroquad::logging::warn!($($arg)+);
    )
}

#[macro_export]
macro_rules! macroquad_info {
    ($($arg:tt)+) => (
        macroquad::logging::info!($($arg)+);
    )
}

#[macro_export]
macro_rules! macroquad_debug {
    ($($arg:tt)+) => (
        macroquad::logging::debug!($($arg)+);
    )
}

#[macro_export]
macro_rules! macroquad_trace {
    ($($arg:tt)+) => (
        macroquad::logging::trace!($($arg)+);
    )
}

pub use macroquad_debug as debug;
pub use macroquad_error as error;
pub use macroquad_info as info;
pub use macroquad_trace as trace;
pub use macroquad_warn as warn;
