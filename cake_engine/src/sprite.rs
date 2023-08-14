//! Спрайт.
//!
//! По сути, текстура, которая помнит свои позицию, поворот и размер.
//!
//! Также спрайт может быть анимированным — разные части текстуры отображаются поочерёдно, сменяясь
//! с указанной частотой.
//!
//! Кадры анимации считываются слева направо, построчно.
//!
//! # Examples
//!
//! ```
//! use std::path::{Path, PathBuf};
//! use cake_engine::{
//!     sprite::{Sprite, SpriteAnimationState},
//!     texture::TextureOptions,
//!     vec::Vec2,
//! };
//! # use cake_engine::{conf::Conf, context::Context, dummy::DummyContext};
//! # let mut dctx = DummyContext::new(&Conf::default());
//! # let mut ctx: &mut dyn Context = &mut dctx;
//!
//! let texture = ctx.load_texture_file(
//!     Path::new("images/texture.png"),
//!     TextureOptions::default()
//! ).unwrap();
//!
//! // Статический спрайт
//! let mut sprite = Sprite::new(texture.clone());
//! sprite.set_position(Vec2::new(64.0, 64.0));
//!
//! // Анимированный спрайт, вариант 1
//! // Задаём сетку с числом кадров в столбцах и строках
//! // Размер кадра посчитается сам
//! let mut anim_sprite1 = Sprite::new_animated_grid(
//!     texture.clone(),
//!     16.0, // число кадров в секунду — длительность анимации полсекунды
//!     (4, 2), // две строки по четыре кадра в каждой
//! );
//!
//! // Анимированный спрайт, вариант 2 — задаём размер кадра
//! let mut anim_sprite2 = Sprite::new_animated(
//!     texture.clone(),
//!     8.0, // тоже число кадров в секунду
//!     (32, 64), // ширина и высота кадра
//! );
//!
//! // Можно повращать относительно центра
//! anim_sprite1.set_origin(Vec2::new(0.5, 0.5));
//! anim_sprite1.set_rotation(45.0); // градусы по часовой стрелке
//!
//! // Можно перевернуть текстуру
//! anim_sprite2.set_flip_x(true);
//!
//! // В вашей сцене в методе process передавайте dt в анимированные спрайты,
//! // чтобы анимация обновлялась
//! let dt = 1.0 / 60.0;
//! anim_sprite1.process(dt);
//! anim_sprite2.process(dt);
//!
//! // А в методе render запускайте их рисование
//! sprite.render(ctx);
//! anim_sprite1.render(ctx).unwrap();
//! anim_sprite2.render(ctx).unwrap();
//! ```

use crate::{
    color::Color,
    context::{Context, DrawTextureParams},
    rect::Rect,
    texture::Texture,
    vec::Vec2,
};
use anyhow::Result;
use std::rc::Rc;

/// Состояние анимации в спрайте.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum SpriteAnimationState {
    /// Анимация играет безостановочно по кругу.
    Playing,
    /// Анимация играет один раз до конца и переходит в паузу.
    PlayingOnce,
    /// Анимация приостановлена.
    Paused,
}

/// Спрайт.
///
/// Подробности и примеры в [документации модуля](self).
#[derive(Clone)]
pub struct Sprite {
    texture: Rc<Texture>,
    color: Color,
    origin: Vec2,
    position: Vec2,
    rotation: f32,
    scale: Vec2,
    flip_x: bool,
    flip_y: bool,
    frame_size: (u32, u32),
    frames_per_line: u32,
    frame_count: u32,
    current_frame: u32,
    current_time: f32,
    sec_per_frame: f32,
    state: SpriteAnimationState,
    texture_src: Rect,
}

impl Sprite {
    /// Создаёт новый статический спрайт.
    ///
    /// (Фактически, создаёт приостановленную анимацию с одним кадром.)
    pub fn new(texture: Rc<Texture>) -> Sprite {
        let w = texture.width;
        let h = texture.height;
        Sprite::new_animated(texture, 0.0, (w, h))
    }

    /// Создаёт новый анимированный спрайт. Кадры задаются путём указания числа столбцов и строк
    /// с кадрами — размер кадра посчитается из них автоматически.
    ///
    /// Число кадров по умолчанию берётся максимальное, но вы можете уменьшить его, если последняя
    /// строка в текстуре не полностью заполнена кадрами.
    ///
    /// Пример: текстура размером 128x64 и `framegrid = (4, 2)` — получаем 8 кадров размером 32x32,
    /// две строки по четыре кадра в каждой.
    pub fn new_animated_grid(texture: Rc<Texture>, fps: f32, frame_grid: (u32, u32)) -> Sprite {
        assert!(frame_grid.0 > 0, "frame cols must be positive");
        assert!(frame_grid.1 > 0, "frame lines must be positive");

        let frame_width = std::cmp::max(1, texture.width / frame_grid.0);
        let frame_height = std::cmp::max(1, texture.height / frame_grid.1);

        Sprite::new_animated(texture, fps, (frame_width, frame_height))
    }

    /// Создаёт новый анимированный спрайт. Кадры задаются путём указания их размера.
    ///
    /// Число кадров по умолчанию берётся максимальное, сколько влезает в текстуру, но вы можете
    /// уменьшить его, если последняя строка в текстуре не полностью заполнена кадрами.
    pub fn new_animated(texture: Rc<Texture>, fps: f32, frame_size: (u32, u32)) -> Sprite {
        assert!(frame_size.0 > 0, "frame width must be positive");
        assert!(frame_size.1 > 0, "frame height must be positive");

        let frames_per_line = std::cmp::max(1, texture.width / frame_size.0);
        let lines = std::cmp::max(1, texture.height / frame_size.1);
        let max_frame_count = frames_per_line * lines;

        Sprite {
            texture,
            color: Color::WHITE,
            origin: Vec2::new(0.0, 0.0),
            position: Vec2::new(0.0, 0.0),
            rotation: 0.0,
            scale: Vec2::new(1.0, 1.0),
            flip_x: false,
            flip_y: false,
            frame_size,
            frames_per_line,
            frame_count: max_frame_count,
            current_frame: 0,
            current_time: 0.0,
            sec_per_frame: 1.0 / fps.abs(),
            state: if max_frame_count > 1 {
                SpriteAnimationState::Playing
            } else {
                SpriteAnimationState::Paused
            },
            texture_src: Rect::new(0.0, 0.0, frame_size.0 as f32, frame_size.1 as f32),
        }
    }

    /// Используемая текстура.
    pub fn texture(&self) -> Rc<Texture> {
        self.texture.clone()
    }

    /// Ссылка на используемую текстуру (если вы не хотите дёргать счётчик в Rc).
    pub fn borrow_texture(&self) -> &Texture {
        &self.texture
    }

    /// Устанавливает новую текстуру с отключением анимации. Размер кадра становится равен
    /// размеру текстуры.
    pub fn set_static_texture(&mut self, texture: Rc<Texture>) {
        self.frame_size = (texture.width, texture.height);
        self.texture = texture;
        self.frames_per_line = 1;
        self.frame_count = 1;
        self.current_frame = 0;
        self.current_time = 0.0;
        self.sec_per_frame = f32::INFINITY;
        self.state = SpriteAnimationState::Paused;
        self.recalc_texture_src();
    }

    /// Устанавливает новую анимированную текстуру со сбросом анимации на начало. Состояние
    /// анимации не изменяет, поэтому не забудьте вызвать [`play`](self::Sprite::play)
    /// по необходимости.
    pub fn set_animated_texture(&mut self, texture: Rc<Texture>, frame_size: (u32, u32), fps: f32) {
        assert!(frame_size.0 > 0, "frame width must be positive");
        assert!(frame_size.1 > 0, "frame height must be positive");

        let frames_per_line = std::cmp::max(1, texture.width / frame_size.0);

        self.texture = texture;
        self.frame_size = frame_size;
        self.frames_per_line = frames_per_line;
        self.current_frame = 0;
        self.current_time = 0.0;
        self.sec_per_frame = 1.0 / fps.abs();
        self.frame_count = self.get_max_frame_count();
        self.recalc_texture_src();
    }

    /// Цвет текстуры.
    pub fn get_color(&self) -> Color {
        self.color
    }

    /// Изменяет цвет текстуры.
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    /// Текущая опорная точка (0.0 — левая/верхняя сторона, 1.0 — правая/нижняя, 0.5 — центр).
    pub fn get_origin(&self) -> Vec2 {
        self.origin
    }

    /// Изменяет опорную точку.
    pub fn set_origin(&mut self, origin: Vec2) {
        self.origin = origin;
    }

    /// Текущее положение спрайта. Место, в котором фактически отрисуется спрайт, может быть
    /// изменено опорной точкой, вращением и масштабом.
    pub fn get_position(&self) -> Vec2 {
        self.position
    }

    /// Изменяет положение спрайта.
    pub fn set_position(&mut self, position: Vec2) {
        self.position = position;
    }

    /// Текущий угол вращения спрайта (в градусах по часовой стрелке).
    pub fn get_rotation(&self) -> f32 {
        self.rotation
    }

    /// Задаёт угол вращения спрайта (в градусах по часовой стрелке).
    pub fn set_rotation(&mut self, rotation: f32) {
        self.rotation = rotation;
    }

    /// Текущий масштаб спрайта (1.0 — стандартный).
    pub fn get_scale(&self) -> Vec2 {
        self.scale
    }

    /// Изменяет масштаб спрайта (1.0 — стандартный, 2.0 — увеличение в два раза и так далее).
    ///
    /// Масштаб может быть отрицательным — тогда спрайт будет перевёрнут относительно своей опорной
    /// точки (то есть, например, при опорной точке 0/0 верхний/левый угол может стать
    /// правым/нижним).
    pub fn set_scale(&mut self, scale: Vec2) {
        self.scale = scale;
    }

    /// Возвращает абсолютный размер спрайта (размер кадра, умноженный на масштаб). Может быть
    /// отрицательным, если масштаб тоже отрицательный.
    pub fn get_absolute_size(&self) -> Vec2 {
        Vec2::new(
            self.frame_size.0 as f32 * self.scale.x,
            self.frame_size.1 as f32 * self.scale.y,
        )
    }

    /// Задаёт абсолютный размер спрайта. Пересчитывает заданный размер в масштаб относительно
    /// размера кадра.
    pub fn set_absolute_size(&mut self, size: Vec2) {
        self.scale.x = size.x / self.frame_size.0 as f32;
        self.scale.y = size.y / self.frame_size.1 as f32;
    }

    /// Возвращает прямоугольник, внутри которого помещается спрайт (однако текущая реализация
    /// не учитывает вращение).
    ///
    /// Размеры прямоугольника могут быть отрицательными, если масштаб отрицательный.
    pub fn get_bounding_rect(&self) -> Rect {
        // TODO: добавить ещё один метод, учитывающий rotation
        let size = self.get_absolute_size();
        Rect::new(
            self.position.x - self.origin.x * size.x,
            self.position.y - self.origin.y * size.y,
            size.x,
            size.y,
        )
    }

    /// Перевёрнута ли текстура по горизонтали.
    pub fn get_flip_x(&self) -> bool {
        self.flip_x
    }

    /// Задаёт переворот текстуры по горизонтали (не влияет на позицию спрайта).
    pub fn set_flip_x(&mut self, flip_x: bool) {
        self.flip_x = flip_x;
        self.recalc_texture_src();
    }

    /// Перевёрнута ли текстура по вертикали.
    pub fn get_flip_y(&self) -> bool {
        self.flip_y
    }

    /// Задаёт переворот текстуры по вертикали (не влияет на позицию спрайта).
    pub fn set_flip_y(&mut self, flip_y: bool) {
        self.flip_y = flip_y;
        self.recalc_texture_src();
    }

    /// Текущий размер кадра анимации. Если спрайт не анимированный, то совпадает с размером
    /// тексутры.
    pub fn get_frame_size(&self) -> (u32, u32) {
        self.frame_size
    }

    /// Текущий кадр анимации. Если спрайт не анимированный, то всегда ноль.
    pub fn get_current_frame(&self) -> u32 {
        self.current_frame
    }

    /// Задаёт текущий кадр анимации.
    pub fn set_current_frame(&mut self, frame: u32) {
        self.current_frame = frame % self.frame_count;
        self.current_time = self.current_frame as f32 * self.sec_per_frame;
        self.recalc_texture_src();
    }

    /// Текущее состояние анимации. Если спрайт не анимированный, то всегда пауза.
    pub fn get_animation_state(&self) -> SpriteAnimationState {
        self.state
    }

    /// Воспроизводит анимацию в цикле без остановки.
    pub fn play(&mut self) {
        if self.frame_count > 1 {
            self.state = SpriteAnimationState::Playing;
        } else {
            self.state = SpriteAnimationState::Paused;
        }
    }

    /// Воспроизводит анимацию один раз, после завершения перейдёт в паузу.
    pub fn play_once(&mut self) {
        if self.frame_count > 1 {
            self.state = SpriteAnimationState::PlayingOnce;
        } else {
            self.state = SpriteAnimationState::Paused;
        }
    }

    /// Приостанавливает анимацию.
    pub fn pause(&mut self) {
        self.state = SpriteAnimationState::Paused;
    }

    /// Возвращает максимальное возможное число кадров при текущих размерах текстуры и кадра.
    pub fn get_max_frame_count(&self) -> u32 {
        let lines = std::cmp::max(1, self.texture.height / self.frame_size.1);
        self.frames_per_line * lines
    }

    /// Число кадров в анимации. Если спрайт не анимированный, то всегда 1.
    pub fn get_frame_count(&self) -> u32 {
        self.frame_count
    }

    /// Изменяет число кадров в анимации. Не может быть меньше 1 и больше чем влезает в текстуру
    /// при текущих размерах. Возвращает число кадров, вписанное в эти ограничения.
    pub fn set_frame_count(&mut self, frame_count: u32) -> u32 {
        self.frame_count = frame_count.clamp(1, self.get_max_frame_count());
        self.frame_count
    }

    /// Обновляет состояние анимации.
    pub fn process(&mut self, dt: f32) {
        if self.frame_count < 2 || self.state == SpriteAnimationState::Paused {
            return;
        }

        self.current_time += dt;
        let next_frame = (self.current_time / self.sec_per_frame) as u32;
        if next_frame == self.current_frame {
            return;
        }

        if next_frame >= self.frame_count {
            if self.state == SpriteAnimationState::PlayingOnce {
                self.current_frame = 0;
                self.current_time = 0.0;
                self.state = SpriteAnimationState::Paused;
            } else {
                self.current_frame = next_frame % self.frame_count;
                self.current_time -= self.sec_per_frame * self.frame_count as f32;
            }
        } else {
            self.current_frame = next_frame;
        }

        self.recalc_texture_src();
    }

    /// Рисование спрайта.
    pub fn render(&self, ctx: &mut dyn Context) -> Result<()> {
        ctx.draw_texture_ex(
            &self.texture,
            DrawTextureParams {
                src: Some(self.texture_src),
                origin: self.origin,
                position: self.position,
                rotation: self.rotation,
                scale: self.scale,
                color: self.color,
                ..Default::default()
            },
        )?;

        Ok(())
    }

    fn recalc_texture_src(&mut self) {
        let line = self.current_frame / self.frames_per_line;
        let column = self.current_frame % self.frames_per_line;
        let mut src = Rect::new(
            (self.frame_size.0 * column) as f32,
            (self.frame_size.1 * line) as f32,
            self.frame_size.0 as f32,
            self.frame_size.1 as f32,
        );
        if self.flip_x {
            src.x += src.width;
            src.width = -src.width;
        }
        if self.flip_y {
            src.y += src.height;
            src.height = -src.height;
        }
        self.texture_src = src;
    }
}
