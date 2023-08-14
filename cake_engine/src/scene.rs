//! Сцена — главный объект, через который движок взаимодействует с вашим кодом.
//!
//! Суть такова: вы создаёте какую-нибудь структуру, реализуете для неё типаж [`Scene`](self::Scene)
//! и пихаете в движок. Движок в цикле вызывает у неё методы `process` для выполнения игровой
//! логики и `render` для рисования — до тех пор, пока вы не прервёте процесс. Также перед запуском
//! цикла вызывается метод `start`, а перед сменой сцены на другую — метод `stop`.
//!
//! Движок передаёт в эти методы мутабельную ссылку на [`Context`](crate::context::Context) — через
//! него вы взаимодействуете с движком: рисуете, читаете ввод и так далее.
//!
//! В методе `process` вы возвращаете движку значение [`SceneResult`](self::SceneResult), которое
//! сообщает ему, что делать дальше.
//!
//! Не забывайте обработать событие выхода из игры (через список `events` или метод
//! `ctx.input().is_quit_requested()`), чтобы пользователь мог завершить игру.
//!
//! # Examples
//!
//! Пример простой сцены с прыгающим квадратом. Демонстрирует основные принципы: обработку ввода,
//! независимую от частоты кадров обработку физики и простенький рендеринг.
//!
//! ```
//! use anyhow::Result;
//! use cake_engine::{
//!     color::Color,
//!     context::Context,
//!     input::{Event, ScanCode},
//!     rect::Rect,
//!     scene::{Scene, SceneResult},
//!     vec::Vec2,
//! };
//!
//! pub struct BouncingRectScene {
//!     size: f32,
//!     pos: f32,
//!     speed: f32,
//!     gravity: f32,
//! }
//!
//! impl BouncingRectScene {
//!     pub fn new() -> BouncingRectScene {
//!         BouncingRectScene {
//!             size: 200.0,
//!             pos: 0.0,
//!             speed: 0.0,
//!             gravity: 600.0,
//!         }
//!     }
//! }
//!
//! impl Scene for BouncingRectScene {
//!     fn process(
//!         &mut self,
//!         ctx: &mut dyn Context,
//!         dt: f32,
//!         _events: &[Event],
//!     ) -> Result<SceneResult> {
//!         // Выход, если пользователь закрывает окно крестиком
//!         if ctx.input().is_quit_requested() {
//!             return Ok(SceneResult::Quit);
//!         }
//!
//!         if ctx.input().is_key_just_pressed(ScanCode::R) {
//!             // Перезапуск. Пример изменения сцены, хотя в данном примере
//!             // просто пересоздаётся эта же сцена
//!             let next_scene = Box::new(BouncingRectScene::new());
//!             return Ok(SceneResult::Switch(next_scene));
//!         }
//!
//!         // Размер окна, преобразованный в координаты нашей игры
//!         // (если view не задан и нет HiDPI, то будет совпадать с размером окна в пикселях)
//!         let area = ctx.view().visible_area();
//!         let bottom = area.y + area.height;
//!
//!         if ctx.input().is_key_just_pressed(ScanCode::Space) {
//!             // Прыжок
//!             self.speed = -300.0;
//!         } else {
//!             // Падение под действием гравитации
//!             self.speed += self.gravity * dt;
//!         }
//!
//!         self.pos += self.speed * dt;
//!         if self.pos >= bottom - self.size {
//!             // Отскок от нижнего края окна
//!             self.pos = (bottom - self.size) * 2.0 - self.pos;
//!             self.speed = -self.speed.abs() * 0.5;
//!         }
//!
//!         Ok(SceneResult::Normal)
//!     }
//!
//!     fn render(&mut self, ctx: &mut dyn Context) -> Result<()> {
//!         let area = ctx.view().visible_area();
//!         let center = area.x + area.width / 2.0;
//!
//!         ctx.set_fill_color(Color::WHITE);
//!         ctx.clear()?;
//!         ctx.set_fill_color(Color::RED);
//!         ctx.fill_rect(
//!             Rect::new(center - self.size / 2.0, self.pos, self.size, self.size),
//!         )?;
//!         Ok(())
//!     }
//! }
//! # use cake_engine::{conf::Conf, dummy::DummyContext};
//! # let mut dctx = DummyContext::new(&Conf::default());
//! # let mut scene = BouncingRectScene::new();
//! # assert!(matches!(scene.process(&mut dctx, 1.0 / 60.0, &[]), Ok(SceneResult::Normal)));
//! # scene.render(&mut dctx).unwrap();
//! # dctx.input_mut().handle_events(&[Event::Quit]);
//! # assert!(matches!(scene.process(&mut dctx, 1.0 / 60.0, &[Event::Quit]), Ok(SceneResult::Quit)));
//! ```

use crate::{context::Context, input::Event};
use anyhow::Result;
use std::fmt::{Debug, Formatter};

/// Результат работы сцены. Сообщает движку, что делать дальше.
pub enum SceneResult {
    /// Обычое выполнение — движок запустит рендеринг и продолжит обрабатывать текущую сцену
    /// в главном цикле.
    Normal,
    /// Закрытие игры.
    Quit,
    /// Переключение на другую сцену. Текущая сцена отрендерена НЕ будет.
    Switch(Box<dyn Scene>),
}

impl Debug for SceneResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            SceneResult::Normal => "SceneResult::Normal",
            SceneResult::Quit => "SceneResult::Quit",
            SceneResult::Switch(_) => "SceneResult::Switch(...)",
        })
    }
}

/// Сцена.
///
/// Подробности и пример в [документации модуля](self).
pub trait Scene {
    /// Этот метод вызывается после того, когда сцена становится текущей, перед первым вызовом
    /// метода [`process`](self::Scene::process). Основной смысл этого метода — передать сюда
    /// предыдущую сцену: это можно использовать, например, для последующего переключения
    /// на предыдущую сцену.
    #[allow(unused_variables)]
    fn start(&mut self, ctx: &mut dyn Context, prev_scene: Option<Box<dyn Scene>>) -> Result<()> {
        Ok(())
    }
    /// Обработка ввода и игровой логики.
    ///
    /// Движок вызывает этот метод в главном цикле после завершения рендеринга предыдущего кадра.
    ///
    /// Параметры:
    ///
    /// * `ctx` — текущий [контекст](crate::context);
    /// * `dt` — время в секундах, затраченное на обработку крайнего кадра;
    /// * `events` — массив произошедших [событий](crate::input::Event) (можно не использовать,
    ///   если для вас более удобно обращаться к [`Input`](crate::input::Input)
    ///   через `ctx.input()`).
    ///
    /// Возвращаемое значение сообщает движку, что делать дальше.
    #[allow(unused_variables)]
    fn process(&mut self, ctx: &mut dyn Context, dt: f32, events: &[Event]) -> Result<SceneResult> {
        Ok(SceneResult::Normal)
    }

    /// Рисование.
    ///
    /// Движок вызывает этот метод сразу после метода [`process`](self::Scene::process), если он
    /// вернул [`SceneResult::Normal`](self::SceneResult::Normal).
    #[allow(unused_variables)]
    fn render(&mut self, ctx: &mut dyn Context) -> Result<()> {
        Ok(())
    }

    /// Этот метод вызывается перед переключением на новую сцену и вызовом метода
    /// [`start`](self::Scene::start) у новой сцены. Можно использовать это, например,
    /// для остановки фоновых звуков.
    #[allow(unused_variables)]
    fn stop(&mut self, ctx: &mut dyn Context) -> Result<()> {
        Ok(())
    }
}

pub(crate) struct InitialScene<'a> {
    scene_builder: &'a dyn Fn(&mut dyn Context) -> Result<Box<dyn Scene>>,
}

impl<'a> InitialScene<'a> {
    pub fn new(scene_builder: &dyn Fn(&mut dyn Context) -> Result<Box<dyn Scene>>) -> InitialScene {
        InitialScene { scene_builder }
    }
}

impl<'a> Scene for InitialScene<'a> {
    fn process(
        &mut self,
        ctx: &mut dyn Context,
        _dt: f32,
        _events: &[Event],
    ) -> Result<SceneResult> {
        Ok(SceneResult::Switch((self.scene_builder)(ctx)?))
    }
}
