#[macro_export]
macro_rules! sdl_error {
    ($($arg:tt)+) => (
        println!($($arg)+);
    )
}

#[macro_export]
macro_rules! sdl_warn {
    ($($arg:tt)+) => (
        println!($($arg)+);
    )
}

#[macro_export]
macro_rules! sdl_info {
    ($($arg:tt)+) => (
        println!($($arg)+);
    )
}

#[macro_export]
macro_rules! sdl_debug {
    ($($arg:tt)+) => (
        println!($($arg)+);
    )
}

#[macro_export]
macro_rules! sdl_trace {
    ($($arg:tt)+) => (
        println!($($arg)+);
    )
}

pub use sdl_debug as debug;
pub use sdl_error as error;
pub use sdl_info as info;
pub use sdl_trace as trace;
pub use sdl_warn as warn;
