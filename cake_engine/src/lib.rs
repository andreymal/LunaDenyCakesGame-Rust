//! Чтобы запустить игру, вам нужны две вещи:
//!
//! * структура [`Conf`](crate::conf::Conf), содержащая параметры для инициализации;
//! * функция, которая создаст вашу первую [сцену](crate::scene).
//!
//! Например, они могут выглядеть как-то так:
//!
//! ```
//! # use cake_engine::scene::Scene;
//! # pub struct BouncingRectScene {}
//! # impl BouncingRectScene {
//! #     pub fn new() -> BouncingRectScene {
//! #         BouncingRectScene {}
//! #     }
//! # }
//! # impl Scene for BouncingRectScene {}
//! use anyhow::Result;
//! use cake_engine::{conf::Conf, context::Context, vec::Vec2};
//!
//! pub fn get_conf() -> Conf {
//!     Conf {
//!         title: "Прыгающий кубик".to_string(),
//!         logical_size: Vec2::new(800.0, 600.0),
//!         ..Default::default()
//!     }
//! }
//!
//! pub fn build_first_scene(ctx: &mut dyn Context) -> Result<Box<dyn Scene>> {
//!     Ok(Box::new(BouncingRectScene::new()))
//! }
//! ```
//!
//! Дальше нужна точка входа с функцией `main`, точный код которой отличается в зависимости
//! от бэкенда.
//!
//! SDL:
//!
//! ```ignore
//! use anyhow::Result;
//!
//! pub fn main() -> Result<()> {
//!     cake_engine::main_sdl(get_conf(), &build_first_scene)
//! }
//! ```
//!
//! SFML:
//!
//! ```ignore
//! use anyhow::Result;
//!
//! pub fn main() -> Result<()> {
//!     cake_engine::main_sfml(get_conf(), &build_first_scene)
//! }
//! ```
//!
//! Macroquad генерирует свою собственную функцию `main` (да ещё и асинхронную) и потому выглядит
//! немного сложнее:
//!
//! ```ignore
//! use anyhow::Result;
//!
//! fn window_conf() -> macroquad::prelude::Conf {
//!     get_conf().into()
//! }
//!
//! #[macroquad::main(window_conf)]
//! async fn main() -> Result<()> {
//!     cake_engine::main_macroquad(get_conf(), &build_first_scene).await
//! }
//! ```
//!
//! Ну и Dummy чисто для поржать:
//!
//! ```
//! use anyhow::Result;
//! # use cake_engine::{conf::Conf, context::Context, input::Event, scene::{Scene, SceneResult}, vec::Vec2};
//! # pub fn get_conf() -> Conf {
//! #     Conf {
//! #         title: "Прыгающий кубик".to_string(),
//! #         logical_size: Vec2::new(800.0, 600.0),
//! #         ..Default::default()
//! #     }
//! # }
//! # pub fn build_first_scene(ctx: &mut dyn Context) -> Result<Box<dyn Scene>> {
//! #     Ok(Box::new(NothingScene))
//! # }
//! # pub struct NothingScene;
//! # impl Scene for NothingScene {
//! #     fn process(&mut self, ctx: &mut dyn Context, dt: f32, events: &[Event]) -> Result<SceneResult> {
//! #         Ok(SceneResult::Quit)
//! #     }
//! # }
//!
//! pub fn main() -> Result<()> {
//!     cake_engine::dummy::main_dummy(get_conf(), &build_first_scene)
//! }
//! ```
//! Чтобы скомпилировать это чудо, создайте соответствующий
//! [binary target](https://doc.rust-lang.org/cargo/reference/cargo-targets.html#binaries)
//! в вашем `Cargo.toml`, включив для него features нужного вам бэкенда.

pub mod audio;
pub mod backend;
pub mod button;
pub mod color;
pub mod conf;
pub mod context;
pub mod font;
pub mod fs;
pub mod gametime;
pub mod input;
pub mod label;
pub mod log;
pub mod rect;
pub mod scene;
pub mod sprite;
pub mod texture;
pub mod utils;
pub mod vec;
pub mod view;

#[cfg(all(target_os = "android", feature = "macroquad"))]
pub mod android;

mod globals;

#[doc(hidden)]
pub use backend::*;
