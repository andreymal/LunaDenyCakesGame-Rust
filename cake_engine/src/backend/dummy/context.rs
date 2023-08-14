use crate::{
    audio::{Music, Sound},
    color::Color,
    conf::Conf,
    context::{Context, DrawTextureParams},
    font::Font,
    gametime::GameTime,
    globals::Globals,
    input::Input,
    rect::Rect,
    texture::{Texture, TextureOptions, TextureSource},
    vec::Vec2,
    view::View,
};
use anyhow::Result;
use std::{path::Path, rc::Rc};

pub struct DummyContext {
    pub conf: Conf,
    globals: Globals,
    current_music_id: Option<usize>,
}

impl DummyContext {
    pub fn new(conf: &Conf) -> DummyContext {
        DummyContext {
            conf: conf.clone(),
            globals: Globals::new(conf.logical_size),
            current_music_id: None,
        }
    }

    pub fn input_mut(&mut self) -> &mut Input {
        &mut self.globals.input
    }
}

impl Context for DummyContext {
    fn get_backend_name(&self) -> &'static str {
        "Dummy"
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
        Vec2::new(1.0, 1.0)
    }

    fn get_logical_window_size(&self) -> Vec2 {
        self.conf.logical_size
    }

    fn get_physical_window_size(&self) -> (u32, u32) {
        (
            self.conf.logical_size.x as u32,
            self.conf.logical_size.y as u32,
        )
    }

    fn get_fullscreen(&self) -> bool {
        self.conf.fullscreen
    }

    fn set_fullscreen(&mut self, fullscreen: bool) -> Result<()> {
        self.conf.fullscreen = fullscreen;
        Ok(())
    }

    fn get_vsync(&self) -> bool {
        self.conf.vsync
    }

    fn set_vsync(&mut self, vsync: bool) -> Result<()> {
        self.conf.vsync = vsync;
        Ok(())
    }

    fn set_fps_limit(&mut self, _value: f32) {}

    fn get_mouse_cursor_visibility(&self) -> bool {
        self.conf.mouse_cursor_visible
    }

    fn set_mouse_cursor_visibility(&mut self, visible: bool) -> Result<()> {
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

    fn set_fill_color(&mut self, _color: Color) {}

    fn clear(&mut self) -> Result<()> {
        Ok(())
    }

    fn fill_rect(&mut self, _rect: Rect) -> Result<()> {
        Ok(())
    }

    fn draw_texture_ex(&mut self, _texture: &Texture, _params: DrawTextureParams) -> Result<()> {
        Ok(())
    }

    fn draw_text_to_texture(
        &mut self,
        _text: &str,
        _font: &Font,
        _color: Color,
        _smooth: bool,
    ) -> Result<Option<Rc<Texture>>> {
        Ok(None)
    }

    fn draw_text(
        &mut self,
        _text: &str,
        _font: &Font,
        _color: Color,
        _smooth: bool,
        _position: Vec2,
        _scale: Vec2,
    ) -> Result<()> {
        Ok(())
    }

    // audio

    fn play_music(&mut self, music: &Music, _volume: f32, _looping: bool) -> Result<()> {
        self.current_music_id = Some(music.id);
        Ok(())
    }

    fn stop_music(&mut self) -> Result<()> {
        self.current_music_id = None;
        Ok(())
    }

    fn get_playing_music(&self) -> Result<Option<Rc<Music>>> {
        if let Some(id) = self.current_music_id.as_ref() {
            Ok(self.globals.music.get(id).cloned())
        } else {
            Ok(None)
        }
    }

    fn play_sound(&mut self, _sound: &Sound, _volume: f32, _looping: bool) -> Result<()> {
        Ok(())
    }

    fn stop_sound(&mut self, _sound: &Sound) -> Result<()> {
        Ok(())
    }

    // resources - textures

    fn is_texture_valid(&self, texture: &Texture) -> bool {
        self.globals.textures.contains_key(&texture.id)
    }

    fn load_texture(
        &mut self,
        source: TextureSource,
        options: TextureOptions,
    ) -> Result<Rc<Texture>> {
        let t = self.globals.add_texture(source, 128, 128, options);
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
            self.globals.textures.remove(&id);
        }
        unused
    }

    fn drop_unused_textures(&mut self) {
        for t_id in self.globals.get_unused_texture_ids() {
            self.globals.textures.remove(&t_id);
        }
    }

    fn reload_lang_textures(&mut self) -> Result<()> {
        Ok(())
    }

    // resources - fonts

    fn load_ttf_file(&mut self, path: &Path, size: u16) -> Result<Rc<Font>> {
        let f = self.globals.add_font(path, size);
        Ok(f)
    }

    fn drop_unused_fonts(&mut self) {
        for f_id in self.globals.get_unused_font_ids() {
            self.globals.fonts.remove(&f_id);
        }
    }

    fn get_text_size(&self, text: &str, font: &Font) -> Result<Vec2> {
        Ok(Vec2::new(
            text.chars().count() as f32 * font.size as f32,
            font.size as f32,
        ))
    }

    fn get_font_metrics(&self, font: &Font) -> Result<(f32, f32)> {
        let ascent = font.size as f32 * 0.75;
        Ok((ascent, font.size as f32 - ascent))
    }

    fn load_music_file(&mut self, path: &Path) -> Result<Rc<Music>> {
        let m = self.globals.add_music(path);
        Ok(m)
    }

    fn drop_unused_music(&mut self) {
        for m_id in self.globals.get_unused_music_ids() {
            self.globals.music.remove(&m_id);
        }
    }

    fn load_sound_file(&mut self, path: &Path) -> Result<Rc<Sound>> {
        let s = self.globals.add_sound(path);
        Ok(s)
    }

    fn drop_unused_sounds(&mut self) {
        for s_id in self.globals.get_unused_sound_ids() {
            self.globals.sounds.remove(&s_id);
        }
    }
}
