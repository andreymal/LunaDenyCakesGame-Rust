#[macro_export]
macro_rules! sfml_error {
    ($($arg:tt)+) => (
        println!($($arg)+);
    )
}

#[macro_export]
macro_rules! sfml_warn {
    ($($arg:tt)+) => (
        println!($($arg)+);
    )
}

#[macro_export]
macro_rules! sfml_info {
    ($($arg:tt)+) => (
        println!($($arg)+);
    )
}

#[macro_export]
macro_rules! sfml_debug {
    ($($arg:tt)+) => (
        println!($($arg)+);
    )
}

#[macro_export]
macro_rules! sfml_trace {
    ($($arg:tt)+) => (
        println!($($arg)+);
    )
}

pub use sfml_debug as debug;
pub use sfml_error as error;
pub use sfml_info as info;
pub use sfml_trace as trace;
pub use sfml_warn as warn;
