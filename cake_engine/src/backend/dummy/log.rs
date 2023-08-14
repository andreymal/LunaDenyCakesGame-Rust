#[macro_export]
#[doc(hidden)]
macro_rules! dummy_error {
    ($($arg:tt)+) => (
        println!($($arg)+);
    )
}

#[macro_export]
#[doc(hidden)]
macro_rules! dummy_warn {
    ($($arg:tt)+) => (
        println!($($arg)+);
    )
}

#[macro_export]
#[doc(hidden)]
macro_rules! dummy_info {
    ($($arg:tt)+) => (
        println!($($arg)+);
    )
}

#[macro_export]
#[doc(hidden)]
macro_rules! dummy_debug {
    ($($arg:tt)+) => (
        println!($($arg)+);
    )
}

#[macro_export]
#[doc(hidden)]
macro_rules! dummy_trace {
    ($($arg:tt)+) => (
        println!($($arg)+);
    )
}

#[doc(hidden)]
pub use dummy_debug as debug;
#[doc(hidden)]
pub use dummy_error as error;
#[doc(hidden)]
pub use dummy_info as info;
#[doc(hidden)]
pub use dummy_trace as trace;
#[doc(hidden)]
pub use dummy_warn as warn;
