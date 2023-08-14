use crate::{
    achievements::AchievementStore,
    data::options::OPTIONS,
    utils::{tex, tex_lang},
};
use anyhow::Result;
use cake_engine::{
    audio::Music,
    button::Button,
    color::Color,
    context::{Context, DrawTextureParams},
    font::Font,
    label::Label,
    sprite::Sprite,
    texture::Texture,
    vec::Vec2,
};
use std::{path::Path, rc::Rc};

#[derive(Clone)]
pub struct CommonData {
    pub cursor: Sprite,
    pub render_cursor: bool,
    pub font_small: Rc<Font>,
    pub font_button: Rc<Font>,
    pub font_help: Rc<Font>,
    pub font_main: Rc<Font>,
    pub font_big: Rc<Font>,
    pub button: Rc<Texture>,
    pub button_small: Rc<Texture>,
    pub button_close: Button,
    pub logo: Rc<Texture>,
    pub back: Rc<Texture>,
    pub checkbox_on: Rc<Texture>,
    pub checkbox_off: Rc<Texture>,
    pub color_over: Color,
    pub color_norm: Color,
    pub music_main: Option<Rc<Music>>,
    pub achievements: AchievementStore,
    pub fps_counter_label: Label,
    pub fps: u64,
    pub draw_fps_counter: bool,
}

impl CommonData {
    pub fn new(ctx: &mut dyn Context) -> Result<CommonData> {
        let area = ctx.view().visible_area();

        let font_path = Path::new("fonts/LiberationSans-Regular.ttf");
        let font_small = ctx.load_ttf_file(&font_path, 18)?;

        let mut fps_counter_label = Label::new(font_small.clone(), Color::BLACK);
        fps_counter_label.set_position(Vec2::new(area.x + area.width, area.y + area.height));
        fps_counter_label.set_origin(Vec2::new(1.0, 1.0));

        let mut common_data = CommonData {
            // Галочка в настройках, счётчик кадров
            font_small,
            // Кнопки, достижения
            font_button: ctx.load_ttf_file(&font_path, 22)?,
            // Справка, об игре
            font_help: ctx.load_ttf_file(&font_path, 24)?,
            // Счётчики здоровья/маны
            font_main: ctx.load_ttf_file(&font_path, 28)?,
            // Конец игры, результат бенчмарка
            font_big: ctx.load_ttf_file(&font_path, 36)?,
            cursor: Sprite::new(tex!(ctx, "images/cursor.png")),
            // Android имеет неотключаемый системный курсор и не пробрасывает события перемещения
            // (и вообще большинство Android-устройств не имеет мышь), поэтому рисовать свой курсор
            // на андроидах не имеет смысла
            render_cursor: cfg![not(target_os = "android")],
            button: tex!(ctx, "images/button.png"),
            button_small: tex!(ctx, "images/button_small.png"),
            button_close: Button::new(
                tex!(ctx, "images/button_close.png"),
                Vec2::new(area.x + area.width, area.y),
            ),
            logo: tex_lang!(ctx, "images/logo.png"),
            back: tex!(ctx, "images/back.png"),
            checkbox_on: tex!(ctx, "images/checkbox_on.png"),
            checkbox_off: tex!(ctx, "images/checkbox_off.png"),
            color_over: Color::WHITE,
            color_norm: Color::new(200, 200, 200, 255),
            music_main: None,
            achievements: AchievementStore::new(),
            fps_counter_label,
            fps: 0,
            draw_fps_counter: OPTIONS.lock().unwrap().get_show_fps_counter(),
        };

        common_data.button_close.set_origin(Vec2::new(1.0, 0.0));
        common_data.button_close.set_color(common_data.color_norm);
        common_data
            .button_close
            .set_color_hover(common_data.color_over);

        if OPTIONS.lock().unwrap().get_musicon() {
            common_data.play_music(ctx)?;
        }

        Ok(common_data)
    }

    pub fn get_or_load_music<'a>(&'a mut self, ctx: &mut dyn Context) -> Result<&'a Music> {
        if self.music_main.is_none() {
            self.music_main = Some(ctx.load_music_file(Path::new("music/music_main.ogg"))?);
        }
        Ok(self.music_main.as_ref().unwrap())
    }

    pub fn play_music(&mut self, ctx: &mut dyn Context) -> Result<()> {
        let music_main = self.get_or_load_music(ctx)?;
        ctx.play_music(music_main, 1.0, true)?;
        Ok(())
    }

    pub fn process(&mut self, ctx: &mut dyn Context) -> Result<()> {
        self.cursor.set_position(ctx.input().get_mouse_position());
        if ctx.view().is_changed() {
            let area = ctx.view().visible_area();
            self.button_close
                .set_position(Vec2::new(area.x + area.width, area.y));
            self.fps_counter_label
                .set_position(Vec2::new(area.x + area.width, area.y + area.height));
        }
        self.button_close.process(ctx)?;
        if self.draw_fps_counter {
            let new_fps = ctx.time().get_fps();
            if new_fps != self.fps {
                self.fps_counter_label.set_text(format!("{} FPS", new_fps));
                self.fps = new_fps;
            }
        }
        Ok(())
    }

    pub fn draw_back(&self, ctx: &mut dyn Context) -> Result<()> {
        let area = ctx.view().visible_area();
        let back_size = self.back.size_vec();
        let scale = Vec2::new(area.width / back_size.x, area.height / back_size.y);

        ctx.draw_texture_ex(
            &self.back,
            DrawTextureParams {
                position: area.get_position(),
                scale,
                ..Default::default()
            },
        )?;
        Ok(())
    }

    pub fn draw_fps_counter(&mut self, ctx: &mut dyn Context) -> Result<()> {
        if self.draw_fps_counter {
            self.fps_counter_label.render(ctx)?;
        }
        Ok(())
    }

    pub fn draw_cursor(&self, ctx: &mut dyn Context) -> Result<()> {
        if self.render_cursor && ctx.input().is_mouse_entered() {
            self.cursor.render(ctx)?;
        }
        Ok(())
    }
}
