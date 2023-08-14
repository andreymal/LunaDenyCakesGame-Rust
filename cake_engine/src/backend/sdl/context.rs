use crate::{
    audio::{Music, Sound},
    backend::sdl::SdlError,
    color::Color,
    conf::Conf,
    context::{Context, DrawTextureParams},
    font::Font,
    fs::asset_path,
    gametime::{FPSLimiter, GameTime},
    globals::Globals,
    input::Input,
    rect::Rect,
    texture::{Texture, TextureOptions, TextureSource},
    vec::Vec2,
    view::View,
};
use anyhow::Result;
use sdl2::{
    image::LoadTexture,
    render::{TextureCreator, WindowCanvas},
    ttf::Sdl2TtfContext,
    video::{FullscreenType, WindowContext},
};
use std::{collections::HashMap, path::Path, rc::Rc};

pub struct SdlContext<'glob, 'win, 'ttf, 'f, 'm> {
    pub(super) globals: &'glob mut Globals,
    pub(super) native_textures: HashMap<usize, sdl2::render::Texture<'win>>,
    pub(super) native_fonts: &'glob mut HashMap<usize, sdl2::ttf::Font<'ttf, 'f>>,
    pub(super) native_music: &'glob mut HashMap<usize, sdl2::mixer::Music<'m>>,
    pub(super) native_sounds: &'glob mut HashMap<usize, sdl2::mixer::Chunk>,
    pub(super) canvas: &'win mut WindowCanvas,
    pub(super) texture_creator: &'win TextureCreator<WindowContext>,
    pub(super) sdl_context: &'glob sdl2::Sdl,
    pub(super) ttf_context: &'ttf Sdl2TtfContext,
    pub(super) conf: &'glob mut Conf,
    pub(super) fps_limiter: FPSLimiter,
    pub(super) rebuild_window: bool,
    pub(super) current_music_id: &'glob mut Option<usize>,
    pub(super) allocated_sound_channels: i32,
    pub(super) sound_channels: HashMap<usize, i32>,
    pub(super) channel_sounds: HashMap<i32, usize>,
}

impl<'win> SdlContext<'_, 'win, '_, '_, '_> {
    pub(super) fn load_sdl_texture(
        &mut self,
        source: &TextureSource,
        options: TextureOptions,
    ) -> Result<sdl2::render::Texture<'win>> {
        sdl2::hint::set(
            "SDL_RENDER_SCALE_QUALITY",
            if options.smooth { "linear" } else { "nearest" },
        );

        let sdl_tex = match source {
            TextureSource::None => return Err(SdlError("TextureSource is None".to_string()).into()),
            TextureSource::Data(data) => self
                .texture_creator
                .load_texture_bytes(data)
                .map_err(SdlError)?,
            TextureSource::File(path) => {
                let data = crate::fs::read_asset_to_bytes(path)?;
                self.texture_creator
                    .load_texture_bytes(&data)
                    .map_err(SdlError)?
            }
            TextureSource::LangFile(path) => {
                let data = crate::fs::read_lang_asset_to_bytes(path)?;
                self.texture_creator
                    .load_texture_bytes(&data)
                    .map_err(SdlError)?
            }
        };

        Ok(sdl_tex)
    }

    pub(super) fn render_text_to_sdl_texture(
        &mut self,
        text: &str,
        font: &Font,
        color: Color,
        smooth: bool,
    ) -> Result<Option<sdl2::render::Texture<'win>>> {
        let sdl_font = match self.native_fonts.get(&font.id) {
            Some(f) => f,
            None => return Err(SdlError("Font not loaded".to_string()).into()),
        };

        if text.is_empty() {
            return Ok(None);
        }

        let surface = sdl_font.render(text).blended(color)?;

        sdl2::hint::set(
            "SDL_RENDER_SCALE_QUALITY",
            if smooth { "linear" } else { "nearest" },
        );
        Ok(Some(
            self.texture_creator.create_texture_from_surface(&surface)?,
        ))
    }

    pub(super) fn reload_all_textures(&mut self) -> Result<()> {
        self.native_textures.clear();
        let textures_to_load = self.globals.textures.clone();
        for (t_id, t) in textures_to_load {
            match t.source {
                TextureSource::None => {}
                _ => {
                    let sdl_tex = self.load_sdl_texture(&t.source, t.options)?;
                    self.native_textures.insert(t_id, sdl_tex);
                }
            }
        }
        Ok(())
    }

    fn rect_to_canvas(&self, rect: Rect) -> Rect {
        let rect2 = self.globals.view.rect_to_target(rect);
        let dpi = self.get_dpi_scale();
        Rect::new(
            rect2.x * dpi.x,
            rect2.y * dpi.y,
            rect2.width * dpi.x,
            rect2.height * dpi.y,
        )
    }

    fn find_free_sound_channel(&self) -> Option<i32> {
        for i in 0..self.allocated_sound_channels {
            if !self.channel_sounds.contains_key(&i) {
                return Some(i);
            }
        }
        None
    }
}

impl Context for SdlContext<'_, '_, '_, '_, '_> {
    fn get_backend_name(&self) -> &'static str {
        "SDL"
    }

    // time

    fn time(&self) -> &GameTime {
        &self.globals.time
    }

    fn time_mut(&mut self) -> &mut GameTime {
        &mut self.globals.time
    }

    // input

    fn input(&self) -> &Input {
        &self.globals.input
    }

    // view

    fn view(&self) -> &View {
        &self.globals.view
    }

    fn view_mut(&mut self) -> &mut View {
        &mut self.globals.view
    }

    // window

    fn get_dpi_scale(&self) -> Vec2 {
        let (pw, ph) = self.canvas.output_size().unwrap();
        let (lw, lh) = self.canvas.window().size();
        let xdpi = pw as f32 / lw as f32;
        let ydpi = ph as f32 / lh as f32;
        Vec2::new(xdpi, ydpi)
    }

    fn get_logical_window_size(&self) -> Vec2 {
        self.canvas.window().size().into()
    }

    fn get_physical_window_size(&self) -> (u32, u32) {
        self.canvas.output_size().unwrap()
    }

    fn get_fullscreen(&self) -> bool {
        match self.canvas.window().fullscreen_state() {
            FullscreenType::Off => false,
            FullscreenType::Desktop => true,
            FullscreenType::True => true,
        }
    }

    fn set_fullscreen(&mut self, fullscreen: bool) -> Result<()> {
        self.canvas
            .window_mut()
            .set_fullscreen(if fullscreen {
                FullscreenType::Desktop
            } else {
                FullscreenType::Off
            })
            .map_err(SdlError)?;
        self.conf.fullscreen = fullscreen;
        Ok(())
    }

    fn get_vsync(&self) -> bool {
        self.conf.vsync
    }

    fn set_vsync(&mut self, vsync: bool) -> Result<()> {
        // TODO: использовать SDL_RenderSetVSync, когда его реализуют в rust-sdl2
        if self.conf.vsync != vsync {
            self.conf.vsync = vsync;
            self.rebuild_window = true;
        }
        Ok(())
    }

    fn set_fps_limit(&mut self, value: f32) {
        self.fps_limiter.set_fps_limit(value);
        self.conf.fps_limit = value;
    }

    fn get_mouse_cursor_visibility(&self) -> bool {
        self.conf.mouse_cursor_visible
    }

    fn set_mouse_cursor_visibility(&mut self, visible: bool) -> Result<()> {
        self.sdl_context.mouse().show_cursor(visible);
        self.conf.mouse_cursor_visible = visible;
        Ok(())
    }

    fn is_simulating_mouse_with_touch(&self) -> bool {
        self.conf.simulate_mouse_with_touch
    }

    fn simulate_mouse_with_touch(&mut self, enabled: bool) {
        self.conf.simulate_mouse_with_touch = enabled;
    }

    // drawing

    fn set_fill_color(&mut self, color: Color) {
        self.canvas.set_draw_color(color);
    }

    fn clear(&mut self) -> Result<()> {
        self.canvas.clear();
        Ok(())
    }

    fn fill_rect(&mut self, rect: Rect) -> Result<()> {
        let mut rect_projected = self.rect_to_canvas(rect);
        rect_projected.normalize();
        if !rect_projected.is_degenerate() {
            self.canvas
                .fill_rect(Some(rect_projected.into()))
                .map_err(SdlError)?;
        }
        Ok(())
    }

    fn draw_texture_ex(&mut self, texture: &Texture, params: DrawTextureParams) -> Result<()> {
        let src = match params.src {
            Some(src) => src,
            None => Rect::new(0.0, 0.0, texture.width as f32, texture.height as f32),
        };
        let src_norm = src.normalized();

        // SDL не поддерживает вывернутые прямоугольники
        let dst = Rect::new(
            params.position.x - src_norm.width * params.scale.x * params.origin.x,
            params.position.y - src_norm.height * params.scale.y * params.origin.y,
            src_norm.width * params.scale.x,
            src_norm.height * params.scale.y,
        );
        let dst_projected = self.rect_to_canvas(dst);

        // Если dst_projected по какой-то причине оказался вывернутый,
        // то точку вращения тоже не забываем вывернуть
        let mut pivot_projected_norm = Vec2::new(
            dst_projected.width * params.origin.x,
            dst_projected.height * params.origin.y,
        );
        if dst_projected.width < 0.0 {
            pivot_projected_norm.x -= dst_projected.width;
        }
        if dst_projected.height < 0.0 {
            pivot_projected_norm.y -= dst_projected.height;
        }

        // Если размеры были заданы отрицательные, то преобразуем вывернутость в параметры SDL
        // (и не забываем, что минус на минус даёт плюс и тогда ничего выворачивать не надо)
        let flip_x = (src.width < 0.0) != (dst_projected.width < 0.0);
        let flip_y = (src.height < 0.0) != (dst_projected.height < 0.0);

        let sdl_texture = match self.native_textures.get_mut(&texture.id) {
            Some(t) => t,
            None => return Err(SdlError("Texture not loaded".to_string()).into()),
        };

        sdl_texture.set_color_mod(params.color.r, params.color.g, params.color.b);
        sdl_texture.set_alpha_mod(params.color.a);

        self.canvas
            .copy_ex(
                &sdl_texture,
                Some(src_norm.into()),
                Some(dst_projected.normalized().into()),
                params.rotation as f64,
                Some(sdl2::rect::Point::new(
                    (pivot_projected_norm.x + 0.5) as i32,
                    (pivot_projected_norm.y + 0.5) as i32,
                )),
                flip_x,
                flip_y,
            )
            .map_err(SdlError)?;

        Ok(())
    }

    fn draw_text_to_texture(
        &mut self,
        text: &str,
        font: &Font,
        color: Color,
        smooth: bool,
    ) -> Result<Option<Rc<Texture>>> {
        let texture = match self.render_text_to_sdl_texture(text, font, color, smooth)? {
            Some(t) => t,
            None => return Ok(None),
        };

        let info = texture.query();

        let t = self.globals.add_texture(
            TextureSource::None,
            info.width,
            info.height,
            TextureOptions { smooth },
        );
        self.native_textures.insert(t.id, texture);
        Ok(Some(t))
    }

    fn draw_text(
        &mut self,
        text: &str,
        font: &Font,
        color: Color,
        smooth: bool,
        position: Vec2,
        scale: Vec2,
    ) -> Result<()> {
        let texture = match self.render_text_to_sdl_texture(text, font, color, smooth)? {
            Some(t) => t,
            None => return Ok(()),
        };

        let info = texture.query();

        let dst_projected = self.rect_to_canvas(Rect::new(
            position.x,
            position.y,
            info.width as f32 * scale.x,
            info.height as f32 * scale.y,
        ));

        self.canvas
            .copy_ex(
                &texture,
                None,
                Some(dst_projected.normalized().into()),
                0.0,
                None,
                dst_projected.width < 0.0,
                dst_projected.height < 0.0,
            )
            .map_err(SdlError)?;

        Ok(())
    }

    // audio

    fn play_music(&mut self, music: &Music, volume: f32, looping: bool) -> Result<()> {
        let sdl_music = match self.native_music.get_mut(&music.id) {
            Some(m) => m,
            None => return Err(anyhow::anyhow!("Music not loaded")),
        };

        sdl2::mixer::Music::set_volume((sdl2::mixer::MAX_VOLUME as f32 * volume) as i32);
        sdl_music
            .play(if looping { -1 } else { 0 })
            .map_err(SdlError)?;

        *self.current_music_id = Some(music.id);
        Ok(())
    }

    fn stop_music(&mut self) -> Result<()> {
        sdl2::mixer::Music::halt();
        *self.current_music_id = None;
        Ok(())
    }

    fn get_playing_music(&self) -> Result<Option<Rc<Music>>> {
        if !sdl2::mixer::Music::is_playing() {
            return Ok(None);
        }
        if let Some(id) = self.current_music_id.as_ref() {
            match self.globals.music.get(id) {
                Some(m) => Ok(Some(m.clone())),
                None => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    fn play_sound(&mut self, sound: &Sound, volume: f32, looping: bool) -> Result<()> {
        let sdl_sound = match self.native_sounds.get_mut(&sound.id) {
            Some(s) => s,
            None => return Err(anyhow::anyhow!("Sound not loaded")),
        };
        sdl_sound.set_volume((sdl2::mixer::MAX_VOLUME as f32 * volume) as i32);

        let channel = sdl2::mixer::Channel(self.sound_channels[&sound.id]);
        channel.halt();
        channel
            .play(&sdl_sound, if looping { -1 } else { 0 })
            .map_err(SdlError)?;
        Ok(())
    }

    fn stop_sound(&mut self, sound: &Sound) -> Result<()> {
        if let Some(cid) = self.sound_channels.get(&sound.id) {
            let channel = sdl2::mixer::Channel(*cid);
            channel.halt();
        }
        Ok(())
    }

    // resources - textures

    fn is_texture_valid(&self, texture: &Texture) -> bool {
        self.native_textures.contains_key(&texture.id)
    }

    fn load_texture(
        &mut self,
        source: TextureSource,
        options: TextureOptions,
    ) -> Result<Rc<Texture>> {
        let sdl_tex = self.load_sdl_texture(&source, options)?;
        let info = sdl_tex.query();

        let t = self
            .globals
            .add_texture(source, info.width, info.height, options);
        self.native_textures.insert(t.id, sdl_tex);
        Ok(t)
    }

    fn drop_texture_if_unused(&mut self, texture: Rc<Texture>) -> bool {
        let id = texture.id;
        drop(texture);

        let unused = if let Some(t) = self.globals.textures.get(&id) {
            Rc::strong_count(t) < 2
        } else {
            true
        };

        if unused {
            self.native_textures.remove(&id);
            self.globals.textures.remove(&id);
        }
        unused
    }

    fn drop_unused_textures(&mut self) {
        for t_id in self.globals.get_unused_texture_ids() {
            self.native_textures.remove(&t_id);
            self.globals.textures.remove(&t_id);
        }
    }

    fn reload_lang_textures(&mut self) -> Result<()> {
        for t in self.globals.get_lang_textures() {
            let sdl_tex = self.load_sdl_texture(&t.source, t.options)?;
            self.native_textures.insert(t.id, sdl_tex);
        }
        Ok(())
    }

    // resources - fonts

    fn load_ttf_file(&mut self, path: &Path, size: u16) -> Result<Rc<Font>> {
        let sdl_font = self
            .ttf_context
            .load_font(&asset_path(path), size)
            .map_err(SdlError)?;
        let f = self.globals.add_font(path, size);
        self.native_fonts.insert(f.id, sdl_font);
        Ok(f)
    }

    fn drop_unused_fonts(&mut self) {
        for f_id in self.globals.get_unused_font_ids() {
            self.native_fonts.remove(&f_id);
            self.globals.fonts.remove(&f_id);
        }
    }

    fn get_text_size(&self, text: &str, font: &Font) -> Result<Vec2> {
        let sdl_font = match self.native_fonts.get(&font.id) {
            Some(f) => f,
            None => return Err(SdlError("Font not loaded".to_string()).into()),
        };

        Ok(sdl_font.size_of(text)?.into())
    }

    fn get_font_metrics(&self, font: &Font) -> Result<(f32, f32)> {
        let sdl_font = match self.native_fonts.get(&font.id) {
            Some(f) => f,
            None => return Err(SdlError("Font not loaded".to_string()).into()),
        };
        Ok((sdl_font.ascent() as f32, -sdl_font.descent() as f32))
    }

    fn get_font_line_height(&self, font: &Font) -> Result<f32> {
        let sdl_font = match self.native_fonts.get(&font.id) {
            Some(f) => f,
            None => return Err(SdlError("Font not loaded".to_string()).into()),
        };
        Ok(sdl_font.recommended_line_spacing() as f32)
    }

    // resources - audio

    fn load_music_file(&mut self, path: &Path) -> Result<Rc<Music>> {
        let sdl_music = sdl2::mixer::Music::from_file(&asset_path(path)).map_err(SdlError)?;

        let m = self.globals.add_music(path);
        self.native_music.insert(m.id, sdl_music);
        Ok(m)
    }

    fn drop_unused_music(&mut self) {
        for m_id in self.globals.get_unused_music_ids() {
            self.native_music.remove(&m_id);
            self.globals.music.remove(&m_id);
        }
    }

    fn load_sound_file(&mut self, path: &Path) -> Result<Rc<Sound>> {
        let channel_id = match self.find_free_sound_channel() {
            Some(i) => i,
            None => {
                sdl2::mixer::allocate_channels(self.allocated_sound_channels + 1);
                self.allocated_sound_channels += 1;
                self.allocated_sound_channels - 1
            }
        };

        let sdl_sound = sdl2::mixer::Chunk::from_file(&asset_path(path)).map_err(SdlError)?;

        let s = self.globals.add_sound(path);
        self.native_sounds.insert(s.id, sdl_sound);
        self.sound_channels.insert(s.id, channel_id);
        self.channel_sounds.insert(channel_id, s.id);
        Ok(s)
    }

    fn drop_unused_sounds(&mut self) {
        for s_id in self.globals.get_unused_sound_ids() {
            let cid = self.sound_channels.remove(&s_id).unwrap();
            self.channel_sounds.remove(&cid);
            self.native_sounds.remove(&s_id);
            self.globals.sounds.remove(&s_id);
        }
    }
}
