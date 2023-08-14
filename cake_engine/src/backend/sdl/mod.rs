pub mod fs;
pub mod log;

mod color;
mod context;
mod error;
mod input;
mod rect;

pub use self::{context::*, error::*};

use crate::{
    conf::{Conf, WindowIcon},
    context::Context,
    fs::asset_path,
    gametime::FPSLimiter,
    globals::Globals,
    scene::{InitialScene, Scene, SceneResult},
};
use anyhow::Result;
use sdl2::{
    image::{InitFlag, LoadSurface},
    render::{BlendMode, WindowCanvas},
    surface::Surface,
    video::Window,
    VideoSubsystem,
};
use std::collections::HashMap;

fn make_window(conf: &Conf, video_subsystem: &VideoSubsystem) -> Result<WindowCanvas> {
    let mut builder = video_subsystem.window(
        conf.title.as_str(),
        conf.logical_size.x as u32,
        conf.logical_size.y as u32,
    );
    let builder = if conf.fullscreen {
        builder.fullscreen_desktop()
    } else {
        builder.position_centered()
    };

    let builder = if conf.resizable {
        builder.resizable()
    } else {
        builder
    };

    let window = builder.allow_highdpi().build()?;

    let canvas_builder = window.into_canvas().accelerated();
    let canvas_builder = if conf.vsync {
        canvas_builder.present_vsync()
    } else {
        canvas_builder
    };
    let canvas = canvas_builder.build()?;

    Ok(canvas)
}

fn set_window_icon(window: &mut Window, icon: &WindowIcon) -> Result<()> {
    let sdl2_surface = Surface::from_file(&asset_path(&icon.path64)).map_err(SdlError)?;
    window.set_icon(sdl2_surface);
    Ok(())
}

pub fn main_sdl(
    mut conf: Conf,
    scene_builder: &'static dyn Fn(&mut dyn Context) -> Result<Box<dyn Scene>>,
) -> Result<()> {
    let sdl_context = sdl2::init().map_err(SdlError)?;
    let video_subsystem = sdl_context.video().map_err(SdlError)?;

    sdl2::image::init(InitFlag::PNG | InitFlag::JPG).map_err(SdlError)?;

    let ttf_context = sdl2::ttf::init()?;

    sdl2::mixer::open_audio(
        44100,
        sdl2::mixer::AUDIO_S16LSB,
        sdl2::mixer::DEFAULT_CHANNELS,
        512,
    )
    .map_err(SdlError)?;
    let _mixer_context = sdl2::mixer::init(sdl2::mixer::InitFlag::OGG).map_err(SdlError)?;
    sdl2::mixer::allocate_channels(4);

    // native_textures привязаны к текущему окну
    let mut native_fonts = HashMap::new();
    let mut native_music = HashMap::new();
    let mut native_sounds = HashMap::new();
    let mut current_music_id = None;

    let mut events = Vec::new();
    let mut scene: Box<dyn Scene> = Box::new(InitialScene::new(scene_builder));
    let mut initial = true;
    let mut globals = Globals::new(conf.logical_size);
    let mut dt = 0.0;

    globals.view.set(conf.view);

    'mainloop: loop {
        let mut canvas = make_window(&conf, &video_subsystem)?;
        sdl_context.mouse().show_cursor(conf.mouse_cursor_visible);
        canvas.set_blend_mode(BlendMode::Blend);
        let mut event_pump = sdl_context.event_pump().map_err(SdlError)?;
        let texture_creator = canvas.texture_creator();

        if let Some(icon) = conf.icon.as_ref() {
            if let Err(e) = set_window_icon(canvas.window_mut(), icon) {
                crate::log::warn!("Failed to set window icon: {:?}", e);
            }
        }

        let mut ctx = SdlContext {
            globals: &mut globals,
            native_textures: HashMap::new(),
            native_fonts: &mut native_fonts,
            native_music: &mut native_music,
            native_sounds: &mut native_sounds,
            canvas: &mut canvas,
            texture_creator: &texture_creator,
            sdl_context: &sdl_context,
            ttf_context: &ttf_context,
            conf: &mut conf,
            fps_limiter: FPSLimiter::new(),
            rebuild_window: false,
            current_music_id: &mut current_music_id,
            allocated_sound_channels: 4,
            sound_channels: HashMap::new(),
            channel_sounds: HashMap::new(),
        };
        ctx.fps_limiter.set_fps_limit(ctx.conf.fps_limit);

        // Так как текстуры привязаны к окну, после пересоздания окна они пропадают,
        // поэтому их приходится заново перечитывать из файлов
        ctx.reload_all_textures()?;

        ctx.globals.input.reset();

        ctx.globals
            .input
            .mark_window_as_resized(ctx.canvas.window().size().into());

        loop {
            input::collect_events(
                &mut event_pump,
                &mut events,
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
                    scene.render(&mut ctx)?;
                    ctx.canvas.present();
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
                if !ctx.conf.fullscreen {
                    ctx.conf.logical_size = ctx.get_logical_window_size();
                }
                break;
            }
        }
    }

    Ok(())
}
