#![doc(hidden)]

pub mod dummy;

#[cfg(feature = "macroquad")]
mod macroquad;

#[cfg(feature = "sdl")]
mod sdl;

#[cfg(feature = "sfml")]
mod sfml;

cfg_if::cfg_if! {
    if #[cfg(feature = "macroquad")] {
        pub use self::macroquad::*;
    } else if #[cfg(feature = "sdl")] {
        pub use self::sdl::*;
    } else if #[cfg(feature = "sfml")] {
        pub use self::sfml::*;
    } else {
        pub use self::dummy::*;
    }
}
