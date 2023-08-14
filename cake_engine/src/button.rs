//! Кнопка.
//!
//! По сути, текстура, которая не только помнит свои позицию и размер, но и умеет определять,
//! когда в неё тыкают мышкой (управление с клавиатуры не реализовано).
//!
//! Опционально может быть добавлена надпись.
//!
//! # Examples
//!
//! ```
//! use std::path::{Path, PathBuf};
//! use cake_engine::{
//!     button::Button,
//!     color::Color,
//!     label::Label,
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
//! // Кнопка в координатах 200/100
//! let mut button = Button::new(texture.clone(), Vec2::new(200.0, 100.0));
//! // Цвет текстуры в обычном состоянии
//! button.set_color(Color::new(200, 200, 200, 255));
//! // Цвет текстуры при наведении курсора мыши
//! button.set_color_hover(Color::new(255, 255, 255, 255));
//!
//! // Опциональная надпись
//! let font = ctx.load_ttf_file(Path::new("fonts/ubuntu.ttf"), 24).unwrap();
//! button.set_label(Some(Label::new(font.clone(), Color::WHITE)));
//!
//! // Редактировать связанный с кнопкой Label можно примерно так
//! if let Some(l) = button.label_mut() {
//!     l.set_text("Выход");
//! }
//!
//! // Но для изменения текста есть более короткий способ
//! button.set_text("Выход");
//!
//! // В метода process вашей сцены дайте кнопке обработать события мыши
//! button.process(ctx).unwrap();
//!
//! // Специальный метод, проверяющий, была ли нажата левая кнопка мыши только что
//! if button.just_clicked() {
//!     println!("Выходим");
//! }
//! ```

use crate::{
    color::Color,
    context::{Context, DrawTextureParams},
    input::MouseButton,
    label::Label,
    rect::Rect,
    texture::Texture,
    vec::Vec2,
};
use anyhow::Result;
use std::rc::Rc;

/// Кнопка.
///
/// Подробности и примеры в [документации модуля](self).
#[derive(Clone)]
pub struct Button {
    position: Vec2,
    origin: Vec2,
    scale: Vec2,
    texture: Rc<Texture>,
    texture_hover: Rc<Texture>,
    color: Color,
    color_hover: Color,
    label: Option<Label>,
    hovered: bool,
    just_pressed_mouse_button: Option<MouseButton>,
}

impl Button {
    /// Создаёт новую кнопку в указанных координатах и с указанной текстурой.
    ///
    /// Выравнивание по умолчанию по центру, масштаб 1, цвет белый, надписи нет.
    pub fn new(texture: Rc<Texture>, position: Vec2) -> Button {
        let texture_hover = texture.clone();
        Button {
            position,
            origin: Vec2::new(0.5, 0.5),
            scale: Vec2::new(1.0, 1.0),
            texture,
            texture_hover,
            color: Color::WHITE,
            color_hover: Color::WHITE,
            label: None,
            hovered: false,
            just_pressed_mouse_button: None,
        }
    }

    /// Текущее положение кнопки относительно опорной точки.
    pub fn position(&self) -> Vec2 {
        self.position
    }

    /// Изменяет положение кнопки.
    pub fn set_position(&mut self, position: Vec2) {
        self.position = position;
        self.update_label_geometry();
    }

    /// Текущая опорная точка (0.0 — левая/верхняя сторона, 1.0 — правая/нижняя, 0.5 — центр).
    pub fn origin(&self) -> Vec2 {
        self.origin
    }

    /// Изменяет опорную точку.
    pub fn set_origin(&mut self, origin: Vec2) {
        self.origin = origin;
        self.update_label_geometry();
    }

    /// Текущий масштаб кнопки (1.0 — стандартный).
    pub fn get_scale(&self) -> Vec2 {
        self.scale
    }

    /// Изменяет масштаб кнопки (1.0 — стандартный, 2.0 — увеличение в два раза и так далее).
    ///
    /// Масштаб может быть отрицательным — тогда кнопка будет перевёрнута относительно своей опорной
    /// точки (то есть, например, при опорной точке 0/0 верхний/левый угол может стать
    /// правым/нижним).
    ///
    /// Не влияет на масштаб текста, используйте `label_mut` для его изменения.
    pub fn set_scale(&mut self, scale: Vec2) {
        self.scale = scale;
        self.update_label_geometry();
    }

    /// Возвращает абсолютный размер кнопки. Может быть отрицательным, если масштаб тоже
    /// отрицательный. Не учитывает размер текста, который может торчать за пределами кнопки.
    pub fn get_absolute_size(&self) -> Vec2 {
        Vec2::new(
            self.texture.width as f32 * self.scale.x,
            self.texture.height as f32 * self.scale.y,
        )
    }

    /// Задаёт абсолютный размер кнопки. Пересчитывает заданный размер в масштаб относительно
    /// размера основной текстуры кнопки. Не влияет на масштаб текста, используйте `label_mut`
    /// для его изменения.
    pub fn set_absolute_size(&mut self, size: Vec2) {
        self.scale.x = size.x / self.texture.width as f32;
        self.scale.y = size.y / self.texture.height as f32;
        self.update_label_geometry();
    }

    /// Возвращает прямоугольник, внутри которого помещается кнопка (без учёта текста).
    ///
    /// Размеры прямоугольника могут быть отрицательными, если масштаб отрицательный.
    pub fn get_bounding_rect(&self) -> Rect {
        let size = self.get_absolute_size();
        Rect::new(
            self.position.x - self.origin.x * size.x,
            self.position.y - self.origin.y * size.y,
            size.x,
            size.y,
        )
    }

    /// Текстура, используемая, когда на кнопку не наведён курсор мыши.
    pub fn texture(&self) -> Rc<Texture> {
        self.texture.clone()
    }

    /// Ссылка на текстуру, используемую, когда на кнопку не наведён курсор мыши (если вы
    /// не хотите дёргать счётчик в Rc).
    pub fn borrow_texture(&self) -> &Texture {
        &self.texture
    }

    /// Устанавливает текстуру, используемую, когда на кнопку не наведён курсор мыши.
    pub fn set_texture(&mut self, texture: Rc<Texture>) {
        self.texture = texture;
        self.update_label_geometry();
    }

    /// Текстура, используемая, когда на кнопку наведён курсор мыши.
    pub fn texture_hover(&self) -> Rc<Texture> {
        self.texture_hover.clone()
    }

    /// Ссылка на текстуру, используемую, когда на кнопку наведён курсор мыши (если вы
    /// не хотите дёргать счётчик в Rc).
    pub fn borrow_texture_hover(&self) -> &Texture {
        &self.texture_hover
    }

    /// Устанавливает текстуру, используемую, когда на кнопку наведён курсор мыши.
    pub fn set_texture_hover(&mut self, texture: Rc<Texture>) {
        self.texture_hover = texture;
    }

    /// Цвет текстуры, когда на кнопку не наведён курсор мыши.
    pub fn color(&self) -> Color {
        self.color
    }

    /// Изменяет цвет текстуры, когда на кнопку не наведён курсор мыши.
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    /// Цвет текстуры, когда на кнопку наведён курсор мыши.
    pub fn color_hover(&self) -> Color {
        self.color_hover
    }

    /// Изменяет цвет текстуры, когда на кнопку наведён курсор мыши.
    pub fn set_color_hover(&mut self, color: Color) {
        self.color_hover = color;
    }

    /// Ссылка на надпись, если она у кнопки есть.
    pub fn label(&self) -> Option<&Label> {
        self.label.as_ref()
    }

    /// Мутабельная ссылка на надпись, если она у кнопки есть.
    pub fn label_mut(&mut self) -> Option<&mut Label> {
        self.label.as_mut()
    }

    /// Заменяет надпись на другую.
    ///
    /// Имейте в виду, что кнопка изменяет опорную точку у надписи, поэтому, если вы хотите
    /// задать свой origin, используйте `label_mut()` для его изменения после вызова `set_label`.
    pub fn set_label(&mut self, label: Option<Label>) {
        self.label = label;
        if let Some(l) = self.label.as_mut() {
            // Не в update_label_geometry, чтобы у пользователя оставалась возможность изменить
            // origin на свой вкус
            l.set_origin(Vec2::new(0.5, 0.5));
        }
        self.update_label_geometry();
    }

    /// Если есть надпись, то изменяет её текст и возвращает `true`.
    ///
    /// Если надписи нет, ничего не делает и просто возвращает `false`.
    ///
    /// Этот метод используется для экономии кода в таком часто встречающемся паттерне:
    /// `if let Some(l) = button.label_mut() { l.set_text(text); }`
    pub fn set_text<S: ToString>(&mut self, text: S) -> bool {
        if let Some(l) = self.label.as_mut() {
            l.set_text(text);
            true
        } else {
            false
        }
    }

    /// Наведён ли курсор мыши на кнопку.
    pub fn is_hovered(&self) -> bool {
        self.hovered
    }

    /// Кнопка мыши, которая нажала на кнопку только что.
    pub fn just_pressed_mouse_button(&self) -> Option<MouseButton> {
        self.just_pressed_mouse_button
    }

    /// Возвращает `true`, если кнопку только что нажали левой кнопкой мыши.
    pub fn just_clicked(&self) -> bool {
        self.just_pressed_mouse_button == Some(MouseButton::Left)
    }

    fn update_label_geometry(&mut self) {
        if self.label.is_some() {
            let rect = self.get_bounding_rect();
            self.label.as_mut().unwrap().set_position(Vec2::new(
                rect.x + rect.width / 2.0,
                rect.y + rect.height / 2.0,
            ));
        }
    }

    /// Обработка. Позволяет кнопке проверить состояние курсора мыши и узнать, нажимает ли она
    /// на кнопку.
    pub fn process(&mut self, ctx: &mut dyn Context) -> Result<()> {
        self.just_pressed_mouse_button = None;

        let input = ctx.input();
        let mouse_pos = input.get_mouse_position();
        let rect = self.get_bounding_rect();

        if input.is_mouse_entered() && rect.contains_point(mouse_pos) {
            self.hovered = true;
            for b in input.just_pressed_mouse_buttons() {
                self.just_pressed_mouse_button = Some(*b);
                break;
            }
        } else {
            self.hovered = false;
        }

        Ok(())
    }

    /// Рисование кнопки.
    pub fn render(&mut self, ctx: &mut dyn Context) -> Result<()> {
        ctx.draw_texture_ex(
            if self.hovered {
                &self.texture_hover
            } else {
                &self.texture
            },
            DrawTextureParams {
                position: self.position,
                origin: self.origin,
                scale: self.scale,
                color: if self.hovered {
                    self.color_hover
                } else {
                    self.color
                },
                ..Default::default()
            },
        )?;

        if let Some(label) = self.label.as_mut() {
            label.render(ctx)?;
        }

        Ok(())
    }
}
