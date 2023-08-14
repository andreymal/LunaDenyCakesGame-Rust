//! Бэкенд-пустышка.
//!
//! Ничего не делает, нужен только для тестирования.
//!
//! Но можно сделать сцену, печатающую в консоль частоту кадров, запустить и любоваться
//! на стопицот миллионов fps 🙃
pub mod fs;
pub mod log;

mod context;

pub use self::context::*;

use crate::{
    context::Context,
    input::Event,
    scene::{InitialScene, Scene, SceneResult},
};
use anyhow::Result;

use crate::conf::Conf;

pub fn main_dummy(
    conf: Conf,
    scene_builder: &'static dyn Fn(&mut dyn Context) -> Result<Box<dyn Scene>>,
) -> Result<()> {
    let mut events: Vec<Event> = Vec::new();
    let mut scene: Box<dyn Scene> = Box::new(InitialScene::new(scene_builder));
    let mut initial = true;
    let mut ctx = DummyContext::new(&conf);
    let mut dt = 0.0;

    ctx.view_mut().set(conf.view);

    'mainloop: loop {
        ctx.input_mut().reset();

        loop {
            ctx.input_mut().handle_events(&events);

            let scene_result = scene.process(&mut ctx, dt, &events)?;

            match scene_result {
                SceneResult::Normal => {
                    scene.render(&mut ctx)?;
                    dt = ctx.time_mut().tick();
                }
                SceneResult::Switch(next_scene) => {
                    let mut prev_scene = scene;
                    scene = next_scene;
                    prev_scene.stop(&mut ctx)?;
                    scene.start(&mut ctx, if !initial { Some(prev_scene) } else { None })?;
                    ctx.drop_unused_resources();
                    initial = false;
                }
                SceneResult::Quit => {
                    break 'mainloop;
                }
            }
            events.clear();
            ctx.view_mut().clear_changed_flag();
        }
    }

    Ok(())
}
