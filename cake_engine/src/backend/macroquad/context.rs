use crate::{
    audio::{Music, Sound},
    color::Color,
    conf::Conf,
    context::{Context, DrawTextureParams},
    font::Font,
    gametime::{FPSLimiter, GameTime},
    globals::Globals,
    input::Input,
    rect::Rect,
    texture::{Texture, TextureOptions, TextureSource},
    vec::Vec2,
    view::View,
};
use anyhow::Result;
use macroquad::{audio::PlaySoundParams, miniquad::FilterMode};
use std::{collections::HashMap, path::Path, rc::Rc};

pub struct MacroquadContext<'glob> {
    pub(super) globals: &'glob mut Globals,
    pub(super) native_textures: &'glob mut HashMap<usize, macroquad::prelude::Texture2D>,
    pub(super) native_fonts: &'glob mut HashMap<usize, macroquad::prelude::Font>,
    pub(super) native_music: &'glob mut HashMap<usize, macroquad::audio::Sound>,
    pub(super) native_sounds: &'glob mut HashMap<usize, macroquad::audio::Sound>,
    pub(super) conf: &'glob mut Conf,
    pub(super) fps_limiter: FPSLimiter,
    pub(super) current_color: macroquad::prelude::Color,
    pub(super) current_music_id: &'glob mut Option<usize>,
}

impl MacroquadContext<'_> {
    fn load_mq_texture(
        &self,
        source: &TextureSource,
        options: TextureOptions,
    ) -> Result<macroquad::texture::Texture2D> {
        let mq_tex = match source {
            TextureSource::None => return Err(anyhow::anyhow!("TextureSource is None")),
            TextureSource::Data(data) => {
                macroquad::prelude::Texture2D::from_file_with_format(data, None)
            }
            TextureSource::File(path) => {
                let data = crate::fs::read_asset_to_bytes(path)?;
                macroquad::prelude::Texture2D::from_file_with_format(&data, None)
            }
            TextureSource::LangFile(path) => {
                let data = crate::fs::read_lang_asset_to_bytes(path)?;
                macroquad::prelude::Texture2D::from_file_with_format(&data, None)
            }
        };

        if options.smooth {
            mq_tex.set_filter(FilterMode::Linear);
        } else {
            mq_tex.set_filter(FilterMode::Nearest);
        }

        Ok(mq_tex)
    }

    fn load_mq_sound(&self, path: &Path) -> Result<macroquad::audio::Sound> {
        let data = crate::fs::read_asset_to_bytes(path)?;

        // Мне лень переделывать весь движок под асинхронщину, поэтому накостыляю
        Ok(futures_executor::block_on(
            macroquad::audio::load_sound_from_bytes(&data),
        )?)
    }
}

impl<'glob> Context for MacroquadContext<'glob> {
    fn get_backend_name(&self) -> &'static str {
        "Macroquad"
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
        let dpi = macroquad::miniquad::window::dpi_scale();
        Vec2::new(dpi, dpi)
    }

    fn get_logical_window_size(&self) -> Vec2 {
        Vec2::new(
            macroquad::window::screen_width(),
            macroquad::window::screen_height(),
        )
    }

    fn get_physical_window_size(&self) -> (u32, u32) {
        let dpi = macroquad::miniquad::window::dpi_scale();
        (
            (macroquad::window::screen_width() * dpi) as u32,
            (macroquad::window::screen_height() * dpi) as u32,
        )
    }

    fn get_fullscreen(&self) -> bool {
        self.conf.fullscreen
    }

    fn set_fullscreen(&mut self, fullscreen: bool) -> Result<()> {
        // Поскольку macroquad не умеет выходить из полноэкранного режима (по крайней мере
        // на линуксе), тыкаем set_fullscreen только когда просят изменить состояние
        if fullscreen != self.conf.fullscreen {
            macroquad::prelude::set_fullscreen(fullscreen);
            self.conf.fullscreen = fullscreen;
        }
        Ok(())
    }

    fn get_vsync(&self) -> bool {
        // macroquad не контролирует vsync, поэтому он может оказаться как включен,
        // так и выключен, как повезёт
        true
    }

    fn set_vsync(&mut self, _vsync: bool) -> Result<()> {
        crate::log::warn!("Setting vsync is not implemented in macroquad");
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
        macroquad::input::show_mouse(visible);
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
        self.current_color = color.into();
    }

    fn clear(&mut self) -> Result<()> {
        macroquad::prelude::clear_background(self.current_color);
        Ok(())
    }

    fn fill_rect(&mut self, rect: Rect) -> Result<()> {
        let mut rect_projected = self.globals.view.rect_to_target(rect);
        rect_projected.normalize();
        if !rect_projected.is_degenerate() {
            macroquad::prelude::draw_rectangle(
                rect_projected.x,
                rect_projected.y,
                rect_projected.width,
                rect_projected.height,
                self.current_color,
            );
        }
        Ok(())
    }

    fn draw_texture_ex(&mut self, texture: &Texture, params: DrawTextureParams) -> Result<()> {
        let mq_texture = match self.native_textures.get(&texture.id) {
            Some(t) => t,
            None => return Err(anyhow::anyhow!("Texture not loaded")),
        };

        let src = match params.src {
            Some(src) => src,
            None => Rect::new(0.0, 0.0, texture.width as f32, texture.height as f32),
        };
        let src_norm = src.normalized();

        // macroquad поддерживает вывернутые прямоугольники
        let dst = Rect::new(
            params.position.x - src_norm.width * params.scale.x * params.origin.x,
            params.position.y - src_norm.height * params.scale.y * params.origin.y,
            src_norm.width * params.scale.x,
            src_norm.height * params.scale.y,
        );
        let dst_projected = self.globals.view.rect_to_target(dst);

        let pivot_projected = self.globals.view.point_to_target(params.position);

        macroquad::prelude::draw_texture_ex(
            mq_texture,
            dst_projected.x,
            dst_projected.y,
            params.color.into(),
            macroquad::texture::DrawTextureParams {
                dest_size: Some(macroquad::prelude::Vec2::new(
                    dst_projected.width,
                    dst_projected.height,
                )),
                source: Some(src.into()),
                rotation: params.rotation.to_radians(),
                pivot: Some(macroquad::prelude::Vec2::new(
                    pivot_projected.x,
                    pivot_projected.y,
                )),
                ..Default::default()
            },
        );

        Ok(())
    }

    fn draw_text_to_texture(
        &mut self,
        _text: &str,
        _font: &Font,
        _color: Color,
        _smooth: bool,
    ) -> Result<Option<Rc<Texture>>> {
        // macroquad не умеет рендерить текст в текстуру
        Ok(None)
    }

    fn draw_text(
        &mut self,
        text: &str,
        font: &Font,
        color: Color,
        _smooth: bool, // macroquad при масштабировании текста рисует его без сглаживания
        position: Vec2,
        scale: Vec2,
    ) -> Result<()> {
        let mq_font = match self.native_fonts.get(&font.id) {
            Some(f) => f,
            None => return Err(anyhow::anyhow!("Texture not loaded")),
        };

        if text.is_empty() {
            return Ok(());
        }

        // macroquad размещает текст относительно baseline, а я хочу относительно верхнего края,
        // поэтому выправляю
        // TODO: сделать опцию для выбора выравнивания?
        let (ascent, _) = self.get_font_metrics(font)?;

        let pos_projected = self.globals.view.point_to_target(position);
        let view_scale = self.globals.view.get_scale();

        macroquad::prelude::draw_text_ex(
            text,
            pos_projected.x,
            pos_projected.y + ascent * scale.y * view_scale.y,
            macroquad::prelude::TextParams {
                font: Some(&mq_font),
                font_size: font.size,
                font_scale: scale.y * view_scale.y,
                font_scale_aspect: (scale.x / scale.y) * (view_scale.x / view_scale.y),
                color: color.into(),
                ..Default::default()
            },
        );
        Ok(())
    }

    // audio

    fn play_music(&mut self, music: &Music, volume: f32, looping: bool) -> Result<()> {
        let mq_music = match self.native_music.get(&music.id) {
            Some(m) => m,
            None => return Err(anyhow::anyhow!("Music not loaded")),
        };

        macroquad::audio::play_sound(
            mq_music,
            PlaySoundParams {
                looped: looping,
                volume,
            },
        );

        *self.current_music_id = Some(music.id);

        Ok(())
    }

    fn stop_music(&mut self) -> Result<()> {
        if let Some(id) = self.current_music_id.as_ref() {
            if let Some(mq_music) = self.native_music.get(id) {
                macroquad::audio::stop_sound(mq_music);
            }
            *self.current_music_id = None;
        }
        Ok(())
    }

    fn get_playing_music(&self) -> Result<Option<Rc<Music>>> {
        if let Some(id) = self.current_music_id.as_ref() {
            Ok(self.globals.music.get(id).cloned())
        } else {
            Ok(None)
        }
    }

    fn play_sound(&mut self, sound: &Sound, volume: f32, looping: bool) -> Result<()> {
        let mq_sound = match self.native_sounds.get(&sound.id) {
            Some(s) => s,
            None => return Err(anyhow::anyhow!("Sound not loaded")),
        };

        macroquad::audio::stop_sound(mq_sound);
        macroquad::audio::play_sound(
            mq_sound,
            PlaySoundParams {
                looped: looping,
                volume,
            },
        );

        Ok(())
    }

    fn stop_sound(&mut self, sound: &Sound) -> Result<()> {
        let mq_sound = match self.native_sounds.get(&sound.id) {
            Some(s) => s,
            None => return Err(anyhow::anyhow!("Sound not loaded")),
        };

        macroquad::audio::stop_sound(mq_sound);
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
        let mq_tex = self.load_mq_texture(&source, options)?;

        let t = self.globals.add_texture(
            source,
            mq_tex.width() as u32,
            mq_tex.height() as u32,
            options,
        );
        self.native_textures.insert(t.id, mq_tex);
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
            let mq_tex = self.load_mq_texture(&t.source, t.options)?;
            self.native_textures.insert(t.id, mq_tex);
        }
        Ok(())
    }

    // resources - fonts

    fn load_ttf_file(&mut self, path: &Path, size: u16) -> Result<Rc<Font>> {
        let data = crate::fs::read_asset_to_bytes(path)?;
        let mq_font = macroquad::text::load_ttf_font_from_bytes(&data)?;
        let f = self.globals.add_font(path, size);
        self.native_fonts.insert(f.id, mq_font);
        Ok(f)
    }

    fn drop_unused_fonts(&mut self) {
        for f_id in self.globals.get_unused_font_ids() {
            self.native_fonts.remove(&f_id);
            self.globals.fonts.remove(&f_id);
        }
    }

    fn get_text_size(&self, text: &str, font: &Font) -> Result<Vec2> {
        let mq_font = match self.native_fonts.get(&font.id) {
            Some(f) => f,
            None => return Err(anyhow::anyhow!("Font not loaded")),
        };

        if text.is_empty() {
            return Ok(Vec2::new(0.0, self.get_font_line_height(font)?));
        }

        // По моей задумке scale влияет только на итоговый рендеринг, поэтому здесь scale = 1.0
        let size = macroquad::prelude::measure_text(&text, Some(mq_font), font.size, 1.0);
        Ok(Vec2::new(size.width, size.height))
    }

    fn get_font_metrics(&self, font: &Font) -> Result<(f32, f32)> {
        let mq_font = match self.native_fonts.get(&font.id) {
            Some(f) => f,
            None => return Err(anyhow::anyhow!("Font not loaded")),
        };

        // Кажется, в macroquad нет более адекватного способа получить высоту строки?
        let size = macroquad::prelude::measure_text("W`q", Some(mq_font), font.size, 1.0);
        Ok((size.offset_y, size.height - size.offset_y))
    }

    fn load_music_file(&mut self, path: &Path) -> Result<Rc<Music>> {
        // macroquad не умеет подгружать музыку на лету, и она грузится и декодируется целиком,
        // что вызывает жор памяти и подвисание на несколько секунд :(
        let mq_music = self.load_mq_sound(path)?;

        let m = self.globals.add_music(path);
        self.native_music.insert(m.id, mq_music);
        Ok(m)
    }

    fn drop_unused_music(&mut self) {
        for m_id in self.globals.get_unused_music_ids() {
            macroquad::audio::stop_sound(&self.native_music.remove(&m_id).unwrap());
            self.globals.music.remove(&m_id);
        }
    }

    fn load_sound_file(&mut self, path: &Path) -> Result<Rc<Sound>> {
        let mq_sound = self.load_mq_sound(path)?;

        let s = self.globals.add_sound(path);
        self.native_sounds.insert(s.id, mq_sound);
        Ok(s)
    }

    fn drop_unused_sounds(&mut self) {
        for s_id in self.globals.get_unused_sound_ids() {
            macroquad::audio::stop_sound(&self.native_sounds.remove(&s_id).unwrap());
            self.globals.sounds.remove(&s_id);
        }
    }
}
