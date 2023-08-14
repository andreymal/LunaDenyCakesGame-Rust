pub mod fs;
pub mod log;

mod color;
mod conf;
mod context;
mod input;
mod rect;

pub use self::context::*;

use crate::{
    context::Context,
    input::Event,
    scene::{InitialScene, Scene, SceneResult},
};
use anyhow::Result;
use macroquad::prelude::next_frame;
use std::collections::HashMap;

use crate::{conf::Conf, gametime::FPSLimiter, globals::Globals};

pub async fn main_macroquad(
    mut conf: Conf,
    scene_builder: &'static dyn Fn(&mut dyn Context) -> Result<Box<dyn Scene>>,
) -> Result<()> {
    macroquad::prelude::prevent_quit();

    // Мы сами всё симулируем
    macroquad::input::simulate_mouse_with_touch(false);

    // Изначально окно создаётся без учёта HiDPI, поэтому ставим правильный размер после создания
    if !conf.fullscreen {
        macroquad::prelude::request_new_screen_size(conf.logical_size.x, conf.logical_size.y);
    }

    macroquad::input::show_mouse(conf.mouse_cursor_visible);

    let mut native_textures = HashMap::new();
    let mut native_fonts = HashMap::new();
    let mut native_music = HashMap::new();
    let mut native_sounds = HashMap::new();
    let mut current_music_id = None;

    let event_subscriber_id = macroquad::input::utils::register_input_subscriber();
    let mut events: Vec<Event> = Vec::new();
    let mut scene: Box<dyn Scene> = Box::new(InitialScene::new(scene_builder));
    let mut initial = true;
    let mut globals = Globals::new(conf.logical_size);
    let mut dt = 0.0;

    globals.view.set(conf.view);

    'mainloop: loop {
        let mut ctx = MacroquadContext {
            globals: &mut globals,
            native_textures: &mut native_textures,
            native_fonts: &mut native_fonts,
            native_music: &mut native_music,
            native_sounds: &mut native_sounds,
            conf: &mut conf,
            fps_limiter: FPSLimiter::new(),
            current_color: macroquad::prelude::Color::new(0.0, 0.0, 0.0, 1.0),
            current_music_id: &mut current_music_id,
        };
        ctx.fps_limiter.set_fps_limit(ctx.conf.fps_limit);

        ctx.globals.input.reset();

        ctx.globals
            .input
            .mark_window_as_resized(ctx.get_logical_window_size());

        loop {
            input::collect_events(
                event_subscriber_id,
                &mut events,
                &mut ctx.globals.view,
                ctx.conf.simulate_mouse_with_touch,
            );
            ctx.globals.input.handle_events(&events);

            let scene_result = scene.process(&mut ctx, dt, &events)?;

            match scene_result {
                SceneResult::Normal => {
                    scene.render(&mut ctx)?;
                    next_frame().await;
                    ctx.fps_limiter.tick(); // TODO: async?
                    dt = ctx.globals.time.tick();
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
            ctx.globals.input.clear();
            ctx.globals.view.clear_changed_flag();
        }
    }

    Ok(())
}
