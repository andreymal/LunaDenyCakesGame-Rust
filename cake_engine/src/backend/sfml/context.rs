use crate::{
    audio::{Music, Sound},
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
use sfml::{
    audio::{SoundSource, SoundStatus},
    graphics::{IntRect, RectangleShape, RenderTarget, RenderWindow, Shape, Sprite, Transformable},
    SfBox,
};
use std::{collections::HashMap, path::Path, rc::Rc};

pub struct SfmlContext<'glob, 'win, 'snd> {
    pub(super) globals: &'glob mut Globals,
    pub(super) native_textures: &'glob mut HashMap<usize, SfBox<sfml::graphics::Texture>>,
    pub(super) native_fonts: &'glob mut HashMap<usize, SfBox<sfml::graphics::Font>>,
    pub(super) native_music: &'glob mut HashMap<usize, sfml::audio::Music<'snd>>,
    pub(super) native_sounds: &'glob mut HashMap<usize, super::audio::SfmlSound>,
    pub(super) window: &'win mut RenderWindow,
    pub(super) fps_limiter: FPSLimiter,
    pub(super) rebuild_window: bool,
    pub(super) conf: &'glob mut Conf,
    pub(super) current_color: sfml::graphics::Color,
    pub(super) current_music_id: &'glob mut Option<usize>,
}

impl SfmlContext<'_, '_, '_> {
    pub(super) fn recalc_sfml_view(&mut self) {
        let wr = self.globals.view.visible_area();
        let view = sfml::graphics::View::new(
            (wr.width / 2.0 + wr.x, wr.height / 2.0 + wr.y).into(),
            (wr.width, wr.height).into(),
        );
        self.window.set_view(&view);
    }

    pub(super) fn load_sfml_texture(
        &mut self,
        source: &TextureSource,
        options: TextureOptions,
    ) -> Result<SfBox<sfml::graphics::Texture>> {
        let mut sfml_tex = sfml::graphics::Texture::new().unwrap();

        match source {
            TextureSource::None => return Err(anyhow::anyhow!("TextureSource is None")),
            TextureSource::Data(data) => {
                sfml_tex.load_from_memory(data, sfml::graphics::IntRect::default())?;
            }
            TextureSource::File(path) => {
                let data = crate::fs::read_asset_to_bytes(path)?;
                sfml_tex.load_from_memory(&data, sfml::graphics::IntRect::default())?;
            }
            TextureSource::LangFile(path) => {
                let data = crate::fs::read_lang_asset_to_bytes(path)?;
                sfml_tex.load_from_memory(&data, sfml::graphics::IntRect::default())?;
            }
        }

        if options.smooth {
            sfml_tex.set_smooth(true);
            sfml_tex.generate_mipmap();
        }

        Ok(sfml_tex)
    }
}

impl Context for SfmlContext<'_, '_, '_> {
    fn get_backend_name(&self) -> &'static str {
        "SFML"
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
        // Кажется, SFML ещё не научился в HiDPI
        Vec2::new(1.0, 1.0)
    }

    fn get_logical_window_size(&self) -> Vec2 {
        let size = self.window.size();
        Vec2::new(size.x as f32, size.y as f32)
    }

    fn get_physical_window_size(&self) -> (u32, u32) {
        let size = self.window.size();
        (size.x, size.y)
    }

    fn get_fullscreen(&self) -> bool {
        self.conf.fullscreen
    }

    fn set_fullscreen(&mut self, fullscreen: bool) -> Result<()> {
        if fullscreen != self.conf.fullscreen {
            self.conf.fullscreen = fullscreen;
            self.rebuild_window = true;
        }
        Ok(())
    }

    fn get_vsync(&self) -> bool {
        self.conf.vsync
    }

    fn set_vsync(&mut self, vsync: bool) -> Result<()> {
        self.window.set_vertical_sync_enabled(vsync);
        self.conf.vsync = vsync;
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
        self.window.set_mouse_cursor_visible(visible);
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
        self.window.clear(self.current_color);
        Ok(())
    }

    fn fill_rect(&mut self, rect: Rect) -> Result<()> {
        let mut shape = RectangleShape::new();
        shape.set_position(rect.get_position());
        shape.set_size(rect.get_size());
        shape.set_fill_color(self.current_color);
        self.window.draw(&shape);
        Ok(())
    }

    fn draw_texture_ex(&mut self, texture: &Texture, params: DrawTextureParams) -> Result<()> {
        let sfml_texture = match self.native_textures.get(&texture.id) {
            Some(t) => t,
            None => return Err(anyhow::anyhow!("Texture not loaded")),
        };

        let src = match params.src {
            Some(src) => src,
            None => Rect::new(0.0, 0.0, texture.width as f32, texture.height as f32),
        };

        let origin_abs = Vec2::new(
            params.origin.x * src.width.abs(),
            params.origin.y * src.height.abs(),
        );

        let mut sprite = Sprite::with_texture(&sfml_texture);
        sprite.set_texture_rect(src.into());
        sprite.set_origin(origin_abs);
        sprite.set_position(params.position);
        sprite.set_rotation(params.rotation);
        sprite.set_scale(params.scale);
        sprite.set_color(params.color.into());
        self.window.draw(&sprite);
        Ok(())
    }

    fn draw_text_to_texture(
        &mut self,
        text: &str,
        font: &Font,
        color: Color,
        smooth: bool,
    ) -> Result<Option<Rc<Texture>>> {
        let sfml_font = match self.native_fonts.get(&font.id) {
            Some(f) => f,
            None => return Err(anyhow::anyhow!("Font not loaded")),
        };

        if text.is_empty() {
            return Ok(None);
        }

        // Создаём объект Text и узнаём размеры текста
        let mut sfml_text = sfml::graphics::Text::new(text, &sfml_font, font.size as u32);
        sfml_text.set_fill_color(color.into());
        let size = sfml_text.local_bounds();

        let tex_width = (size.left + size.width).ceil() as u32;
        let tex_height = (size.top + size.height).ceil() as u32;

        // Рендерим текст в текстуру
        let mut rt = match sfml::graphics::RenderTexture::new(tex_width, tex_height) {
            Some(rt) => rt,
            None => return Err(anyhow::anyhow!("Failed to create RenderTexture")),
        };
        let mut bg = color;
        bg.a = 0;
        rt.clear(bg.into());
        rt.draw(&sfml_text);
        rt.display();

        // RenderTexture владеет свой текстурой, но он нам больше не нужен, поэтому копируем
        let sfml_image = match rt.texture().copy_to_image() {
            Some(im) => im,
            None => return Err(anyhow::anyhow!("Failed to copy RenderTexture to Image")),
        };

        let mut sfml_tex = match sfml::graphics::Texture::new() {
            Some(t) => t,
            None => return Err(anyhow::anyhow!("Failed to create texture")),
        };
        if !sfml_tex.create(tex_width, tex_height) {
            return Err(anyhow::anyhow!("Failed to create texture"));
        }
        sfml_tex.load_from_image(
            &sfml_image,
            IntRect::new(0, 0, tex_width as i32, tex_height as i32),
        )?;

        if smooth {
            sfml_tex.set_smooth(true);
            sfml_tex.generate_mipmap();
        }

        let t = self.globals.add_texture(
            TextureSource::None,
            tex_width,
            tex_height,
            TextureOptions { smooth },
        );
        self.native_textures.insert(t.id, sfml_tex);
        Ok(Some(t))
    }

    fn draw_text(
        &mut self,
        text: &str,
        font: &Font,
        color: Color,
        _smooth: bool, // SFML не позволяет отрендерить пиксельный текст?
        position: Vec2,
        scale: Vec2,
    ) -> Result<()> {
        let sfml_font = match self.native_fonts.get(&font.id) {
            Some(f) => f,
            None => return Err(anyhow::anyhow!("Font not loaded")),
        };

        if text.is_empty() {
            return Ok(());
        }

        let mut sfml_text = sfml::graphics::Text::new(text, &sfml_font, font.size as u32);
        sfml_text.set_fill_color(color.into());
        sfml_text.set_position(position);
        sfml_text.set_scale(scale);
        self.window.draw(&sfml_text);

        Ok(())
    }

    // audio

    fn play_music(&mut self, music: &Music, volume: f32, looping: bool) -> Result<()> {
        self.stop_music()?;
        let sfml_music = match self.native_music.get_mut(&music.id) {
            Some(m) => m,
            None => return Err(anyhow::anyhow!("Music not loaded")),
        };
        sfml_music.set_looping(looping);
        sfml_music.set_volume(volume * 100.0);
        sfml_music.play();
        *self.current_music_id = Some(music.id);
        Ok(())
    }

    fn stop_music(&mut self) -> Result<()> {
        if let Some(id) = self.current_music_id {
            if let Some(m) = self.native_music.get_mut(&id) {
                m.stop();
            }
            *self.current_music_id = None;
        }
        Ok(())
    }

    fn get_playing_music(&self) -> Result<Option<Rc<Music>>> {
        if let Some(id) = self.current_music_id.as_ref() {
            if let Some(sfml_music) = self.native_music.get(id) {
                let music = match self.globals.music.get(id) {
                    Some(m) => m,
                    None => return Ok(None),
                };
                if sfml_music.status() != SoundStatus::STOPPED {
                    return Ok(Some(music.clone()));
                } else {
                    return Ok(None);
                }
            }
        }
        Ok(None)
    }

    fn play_sound(&mut self, sound: &Sound, volume: f32, looping: bool) -> Result<()> {
        let sfml_sound = match self.native_sounds.get_mut(&sound.id) {
            Some(s) => s,
            None => return Err(anyhow::anyhow!("Sound not loaded")),
        };
        sfml_sound.with_dependent_mut(|_, sound| {
            sound.set_volume(volume * 100.0);
            sound.set_looping(looping);
            sound.play();
        });
        Ok(())
    }

    fn stop_sound(&mut self, sound: &Sound) -> Result<()> {
        let sfml_sound = match self.native_sounds.get_mut(&sound.id) {
            Some(s) => s,
            None => return Err(anyhow::anyhow!("Sound not loaded")),
        };
        sfml_sound.with_dependent_mut(|_, sound| {
            sound.stop();
        });
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
        let sfml_tex = self.load_sfml_texture(&source, options)?;

        let size = sfml_tex.size();

        let t = self.globals.add_texture(source, size.x, size.y, options);
        self.native_textures.insert(t.id, sfml_tex);
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
            let sfml_tex = self.load_sfml_texture(&t.source, t.options)?;
            self.native_textures.insert(t.id, sfml_tex);
        }
        Ok(())
    }

    // resources - fonts

    fn load_ttf_file(&mut self, path: &Path, size: u16) -> Result<Rc<Font>> {
        let sfml_font = match sfml::graphics::Font::from_file(asset_path(path).to_str().unwrap()) {
            Some(f) => f,
            None => return Err(anyhow::anyhow!("Failed to load font")),
        };
        let f = self.globals.add_font(path, size);
        self.native_fonts.insert(f.id, sfml_font);
        Ok(f)
    }

    fn drop_unused_fonts(&mut self) {
        for f_id in self.globals.get_unused_font_ids() {
            self.native_fonts.remove(&f_id);
            self.globals.fonts.remove(&f_id);
        }
    }

    fn get_text_size(&self, text: &str, font: &Font) -> Result<Vec2> {
        let sfml_font = match self.native_fonts.get(&font.id) {
            Some(f) => f,
            None => return Err(anyhow::anyhow!("Font not loaded")),
        };

        let sfml_text = sfml::graphics::Text::new(text, &sfml_font, font.size as u32);
        let size = sfml_text.local_bounds();
        Ok(Vec2::new(size.left + size.width, size.top + size.height))
    }

    fn get_font_metrics(&self, font: &Font) -> Result<(f32, f32)> {
        let sfml_font = match self.native_fonts.get(&font.id) {
            Some(f) => f,
            None => return Err(anyhow::anyhow!("Font not loaded")),
        };

        // SFML серьёзно не предоставляет метода для чтения ascent и descent? Я неприятно удивлён
        let mut sfml_text = sfml::graphics::Text::new("W`", &sfml_font, font.size as u32);
        let size_ascent = sfml_text.local_bounds();
        sfml_text.set_string("W`q");
        let size_full = sfml_text.local_bounds();
        Ok((
            size_ascent.top + size_ascent.height,
            size_full.top + size_full.height - size_ascent.top - size_ascent.height,
        ))
    }

    // resources - audio

    fn load_music_file(&mut self, path: &Path) -> Result<Rc<Music>> {
        let sfml_music = match sfml::audio::Music::from_file(asset_path(path).to_str().unwrap()) {
            Some(m) => m,
            None => return Err(anyhow::anyhow!("Failed to load music")),
        };

        let m = self.globals.add_music(path);
        self.native_music.insert(m.id, sfml_music);
        Ok(m)
    }

    fn drop_unused_music(&mut self) {
        for m_id in self.globals.get_unused_music_ids() {
            self.native_music.remove(&m_id);
            self.globals.music.remove(&m_id);
        }
    }

    fn load_sound_file(&mut self, path: &Path) -> Result<Rc<Sound>> {
        let buf = sfml::audio::SoundBuffer::from_file(asset_path(path).to_str().unwrap())?;
        let sfml_sound = super::audio::SfmlSound::new(std::rc::Rc::new(buf), |buf| {
            let mut sound = sfml::audio::Sound::new();
            sound.set_buffer(buf);
            sound
        });

        let s = self.globals.add_sound(path);
        self.native_sounds.insert(s.id, sfml_sound);
        Ok(s)
    }

    fn drop_unused_sounds(&mut self) {
        for s_id in self.globals.get_unused_sound_ids() {
            self.native_sounds.remove(&s_id);
            self.globals.sounds.remove(&s_id);
        }
    }
}
