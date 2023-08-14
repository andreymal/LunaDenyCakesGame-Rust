pub mod fs;
pub mod log;

mod audio;
mod color;
mod context;
mod input;
mod rect;
mod vec;

pub use context::*;

use crate::{
    color::Color,
    conf::{Conf, WindowIcon},
    context::Context,
    gametime::FPSLimiter,
    globals::Globals,
    input::Event,
    scene::{InitialScene, Scene, SceneResult},
};
use anyhow::Result;
use sfml::{
    graphics::{Font, RenderTarget, RenderWindow, Texture},
    window::{ContextSettings, Style, VideoMode},
    SfBox,
};
use std::collections::HashMap;

fn make_window(conf: &Conf) -> Result<RenderWindow> {
    let display_mode = VideoMode::desktop_mode();

    let (w, h) = if conf.fullscreen {
        (display_mode.width, display_mode.height)
    } else {
        (conf.logical_size.x as u32, conf.logical_size.y as u32)
    };

    let style = if conf.fullscreen {
        Style::FULLSCREEN
    } else {
        let mut s = Style::TITLEBAR | Style::CLOSE;
        if conf.resizable {
            s = s | Style::RESIZE;
        }
        s
    };

    let mut window = RenderWindow::new(
        VideoMode::new(w, h, display_mode.bits_per_pixel),
        conf.title.as_str(),
        style,
        &ContextSettings::default(),
    );
    window.set_vertical_sync_enabled(conf.vsync);
    window.set_mouse_cursor_visible(conf.mouse_cursor_visible);
    Ok(window)
}

fn set_window_icon(window: &mut RenderWindow, icon: &WindowIcon) -> Result<()> {
    let data = crate::fs::read_asset_to_bytes(&icon.path64)?;
    let im = match sfml::graphics::Image::from_memory(&data) {
        Some(im) => im,
        None => return Err(anyhow::anyhow!("Failed to create SFML image from file")),
    };
    let size = im.size();

    if size.x > 64 || size.y > 64 {
        return Err(anyhow::anyhow!("Window icon is too big"));
    }
    if size.x < 2 || size.y < 2 {
        return Err(anyhow::anyhow!("Window icon is too small"));
    }

    assert_eq!(im.pixel_data().len(), (size.x * size.y * 4) as usize);

    unsafe {
        window.set_icon(size.x, size.y, im.pixel_data());
    }
    Ok(())
}

pub fn main_sfml(
    mut conf: Conf,
    scene_builder: &'static dyn Fn(&mut dyn Context) -> Result<Box<dyn Scene>>,
) -> Result<()> {
    let mut native_textures: HashMap<usize, SfBox<Texture>> = HashMap::new();
    let mut native_fonts: HashMap<usize, SfBox<Font>> = HashMap::new();
    let mut native_music: HashMap<usize, sfml::audio::Music> = HashMap::new();
    let mut native_sounds: HashMap<usize, self::audio::SfmlSound> = HashMap::new();
    let mut current_music_id = None;

    let mut events: Vec<Event> = Vec::new();
    let mut scene: Box<dyn Scene> = Box::new(InitialScene::new(scene_builder));
    let mut initial = true;
    let mut globals = Globals::new(conf.logical_size);
    let mut dt = 0.0;

    globals.view.set(conf.view);

    'mainloop: loop {
        let mut window = make_window(&conf)?;

        if let Some(icon) = conf.icon.as_ref() {
            if let Err(e) = set_window_icon(&mut window, icon) {
                crate::log::warn!("Failed to set window icon: {:?}", e);
            }
        }

        let mut ctx = SfmlContext {
            globals: &mut globals,
            native_textures: &mut native_textures,
            native_fonts: &mut native_fonts,
            native_music: &mut native_music,
            native_sounds: &mut native_sounds,
            window: &mut window,
            rebuild_window: false,
            conf: &mut conf,
            fps_limiter: FPSLimiter::new(),
            current_color: Color::BLACK.into(),
            current_music_id: &mut current_music_id,
        };
        ctx.fps_limiter.set_fps_limit(ctx.conf.fps_limit);

        // Во время пересоздания окна некоторые события, в том числе KeyUp, могут потеряться,
        // что приводит к «залипанию» клавиш, поэтому чистим списки нажатых клавиш
        ctx.globals.input.reset();

        // Но нужно обработать факт возможно изменившегося размера нового окна
        ctx.globals
            .input
            .mark_window_as_resized(ctx.window.size().into());
        ctx.globals.view.set_target_size(ctx.window.size().into());
        ctx.recalc_sfml_view();

        loop {
            let was_fullscreen = ctx.conf.fullscreen;

            input::collect_events(
                ctx.window,
                &mut events,
                &ctx.globals.input,
                &ctx.globals.view,
                ctx.conf.simulate_mouse_with_touch,
            );
            ctx.globals.input.handle_events(&events);

            if let Some(logical_size) = ctx.globals.input.is_window_just_resized() {
                ctx.globals.view.set_target_size(logical_size);
            }

            let scene_result = scene.process(&mut ctx, dt, &events)?;

            match scene_result {
                SceneResult::Normal => {
                    if ctx.globals.view.is_changed() {
                        ctx.recalc_sfml_view();
                    }
                    scene.render(&mut ctx)?;
                    ctx.window.display();
                    ctx.fps_limiter.tick();
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

            if ctx.rebuild_window {
                // Перед переходом в полноэкранный режим запоминаем размер окна,
                // чтобы при выходе из полноэкранного размера вернулся прежний размер
                if ctx.conf.fullscreen && !was_fullscreen {
                    ctx.conf.logical_size = ctx.get_logical_window_size();
                }
                break;
            }
        }
    }

    Ok(())
}
