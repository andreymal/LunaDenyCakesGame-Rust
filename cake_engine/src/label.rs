//! Текст. Поддерживает многострочность и кэширование текстур с текстом.

use crate::{
    color::Color,
    context::{Context, DrawTextureParams},
    font::Font,
    rect::Rect,
    texture::Texture,
    utils::wrap_text,
    vec::Vec2,
};
use anyhow::Result;
use std::rc::Rc;

#[derive(Clone)]
struct LabelLine {
    line: String,
    size: Vec2,
    texture: Option<Rc<Texture>>,
}

/// Текст. Поддерживает многострочность и кэширование текстур с текстом.
///
/// По сути, обёртка над методами
/// [`Context.draw_text_to_texture`](crate::context::Context::draw_text_to_texture) и
/// [`Context.draw_text`](crate::context::Context::draw_text), но умеет делать дополнительные
/// штуки:
///
/// * делит текст на строки по словам (опционально с указанием максимальной допустимой ширины);
/// * если бэкенд умеет рисовать текст в текстуру, то хранит кэш текстур со строками текста,
///   что позволяет избавиться от повторной растеризации и сильно повысить производительность.
///
/// Все размеры текста считаются в пикселях, [`scale`](self::Label::get_scale) влияет только
/// на итоговый рендеринг, но не на расчёт строк.
#[derive(Clone)]
pub struct Label {
    text: String,
    font: Rc<Font>,
    color: Color,
    scale: Vec2,
    position: Vec2,
    origin: Vec2,
    text_align: f32,
    max_width: f32,
    lines: Vec<LabelLine>,
    total_size: Vec2,
    smooth: bool,
    shadow_color: Option<Color>,
    shadow_offset: Vec2,
    need_rebuild: bool,
}

impl Label {
    /// Создаёт новый объект с указанными шрифтом и цветом и пустым тектом. Остальные параметры
    /// можно задать через методы.
    pub fn new(font: Rc<Font>, color: Color) -> Label {
        Label {
            text: String::new(),
            font,
            color,
            scale: Vec2::new(1.0, 1.0),
            position: Vec2::new(0.0, 0.0),
            origin: Vec2::new(0.0, 0.0),
            text_align: 0.0,
            max_width: 0.0,
            lines: Vec::new(),
            total_size: Vec2::new(0.0, 0.0),
            smooth: false,
            shadow_color: None,
            shadow_offset: Vec2::new(0.0, 0.0),
            need_rebuild: true,
        }
    }

    /// Пересчитывает строки текста, если в этом есть необходимость.
    pub fn rebuild_if_needed(&mut self, ctx: &mut dyn Context) -> Result<()> {
        if self.need_rebuild {
            self.rebuild(ctx)?;
        }
        Ok(())
    }

    /// Пересчитывает строки текста принудительно. Не стоит вызывать этот метод слишком часто,
    /// чтобы не убить производительность (да и вообще он выполнится сам по необходимости).
    pub fn rebuild(&mut self, ctx: &mut dyn Context) -> Result<()> {
        self.clear_cache(ctx);

        let get_line_width = |s: &_| ctx.get_text_size(s, &self.font).unwrap().x;
        let line_height = ctx.get_font_line_height(&self.font)?;

        // Делим текст на строки
        let lines_tmp: Vec<_> = if self.max_width > 0.0 {
            wrap_text(&self.text, &get_line_width, self.max_width).collect()
        } else {
            self.text
                .split('\n')
                .map(|line| (get_line_width(line), line.trim().to_string()))
                .collect()
        };

        // Рисуем по текстуре для каждой строки
        // (если бэкенд не поддерживает рисование текста в текстуру, вместо текстур будет None)
        for (line_width, line) in lines_tmp {
            let font_texture =
                ctx.draw_text_to_texture(&line, &self.font, Color::WHITE, self.smooth)?;

            if line_width > self.total_size.x {
                self.total_size.x = line_width;
            }
            self.total_size.y += line_height;

            self.lines.push(LabelLine {
                line,
                size: Vec2::new(line_width, line_height),
                texture: font_texture,
            });
        }

        self.need_rebuild = false;
        Ok(())
    }

    /// Очищает кэш, в том числе текстуры для освобождения видеопамяти. Рекомендуется вызывать
    /// перед уничтожением структуры.
    pub fn clear_cache(&mut self, ctx: &mut dyn Context) {
        for l in &mut self.lines {
            if let Some(t) = l.texture.take() {
                ctx.drop_texture_if_unused(t);
            }
        }

        self.lines.clear();
        self.total_size = Vec2::new(0.0, 0.0);
        self.need_rebuild = true;
    }

    /// Текущий текст.
    pub fn get_text(&self) -> &str {
        self.text.as_str()
    }

    /// Меняет текст.
    pub fn set_text<S: ToString>(&mut self, text: S) {
        let text = text.to_string();
        if self.text != text {
            self.text = text;
            self.need_rebuild = true;
        }
    }

    /// Текущий цвет текста.
    pub fn get_color(&self) -> Color {
        self.color
    }

    /// Меняет цвет текста.
    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    /// Текущий масштаб текста.
    pub fn get_scale(&self) -> Vec2 {
        self.scale
    }

    /// Изменяет масштаб текста. Масштаб используется только при рендеринге, на расчёт строк
    /// текста он не влияет и размер в пикселях не изменяет.
    pub fn set_scale(&mut self, scale: Vec2) {
        self.scale = scale;
    }

    /// Текущая позиция текста.
    pub fn get_position(&self) -> Vec2 {
        self.position
    }

    /// Изменяет позицию текста.
    pub fn set_position(&mut self, position: Vec2) {
        self.position = position;
    }

    /// Текущая опорная точка текста.
    pub fn get_origin(&self) -> Vec2 {
        self.origin
    }

    /// Изменяет опорную точку текста (0.0 — левый/верхний угол, 1.0 — правый/нижний угол).
    pub fn set_origin(&mut self, origin: Vec2) {
        self.origin = origin;
    }

    /// Текущее выравнивание строк текста (0.0 — по левому краю, 0.5 — по центру, 1.0 — по правому
    /// краю).
    pub fn get_text_align(&self) -> f32 {
        self.text_align
    }

    /// Задаёт выравнивание строк текста (0.0 — по левому краю, 0.5 — по центру, 1.0 — по правому
    /// краю).
    pub fn set_text_align(&mut self, text_align: f32) {
        self.text_align = text_align;
    }

    /// Текущая максимальная длина текста в пикселях (без учёта `scale`). Ноль означает,
    /// что длина не ограничена.
    pub fn get_max_width(&self) -> f32 {
        self.max_width
    }

    /// Изменяет максимальную длину текста в пикселях (без учёта `scale`). Ноль означает,
    /// что длина не ограничена.
    pub fn set_max_width(&mut self, max_width: f32) {
        if self.max_width != max_width {
            self.max_width = max_width;
            self.need_rebuild = true;
        }
    }

    /// Применяется ли сглаживание при рисовании текста.
    pub fn get_smooth(&self) -> bool {
        self.smooth
    }

    /// Включает или выключает использование сглаживания при рисовании текста. Разница заметна,
    /// если [`scale`](self::Label::get_scale) отличается от единицы.
    pub fn set_smooth(&mut self, smooth: bool) {
        if smooth != self.smooth {
            self.smooth = smooth;
            self.need_rebuild = true;
        }
    }

    /// Цвет тени, `None` означает отсутствие тени.
    pub fn get_shadow_color(&self) -> Option<Color> {
        self.shadow_color
    }

    /// Устанавливает цвет тени, `None` отключает тень.
    pub fn set_shadow_color(&mut self, color: Option<Color>) {
        self.shadow_color = color;
    }

    /// Смещение тени относительно текста.
    pub fn get_shadow_offset(&self) -> Vec2 {
        self.shadow_offset
    }

    /// Устанавливает смещение тени относительно текста. Если тень отключена (цвет установлен
    /// в `None`), то ни на что не влияет.
    pub fn set_shadow_offset(&mut self, offset: Vec2) {
        self.shadow_offset = offset;
    }

    /// Метод, позволяющий одним вызовом включить тень и задать её цвет и смещение.
    pub fn set_shadow(&mut self, color: Color, offset: Vec2) {
        self.shadow_color = Some(color);
        self.shadow_offset = offset;
    }

    /// Возвращает прямоугольник, в границах которого будет рисоваться текст.
    ///
    /// Если расчёт геометрии текста ещё не выполнялся, то границы ещё не известны и метод вернёт
    /// `None`. Границы гарантированно известны непосредственно после вызова `render`, `rebuild`
    /// или `rebuild_if_needed`.
    pub fn get_bounding_rect(&self) -> Option<Rect> {
        if self.need_rebuild {
            return None;
        }
        let final_size = Vec2::new(
            self.total_size.x * self.scale.x,
            self.total_size.y * self.scale.y,
        );
        let origin_abs = Vec2::new(self.origin.x * final_size.x, self.origin.y * final_size.y);
        Some(Rect::new(
            self.position.x - origin_abs.x,
            self.position.y - origin_abs.y,
            final_size.x,
            final_size.y,
        ))
    }

    /// Рисует текст.
    pub fn render(&mut self, ctx: &mut dyn Context) -> Result<()> {
        if self.text.is_empty() {
            return Ok(());
        }

        // Если бэкенд пересоздавал окно, текстуры в кэше могли пропасть
        if !self.need_rebuild {
            self.need_rebuild = self.lines.is_empty();
            for l in &self.lines {
                if let Some(tex) = &l.texture {
                    if !ctx.is_texture_valid(tex) {
                        self.need_rebuild = true;
                        break;
                    }
                }
            }
        }
        // Перерасчёт геометрии и пересоздание кэша текстур
        if self.need_rebuild {
            self.rebuild(ctx)?;
        }

        // Опорная точка в абсолютных единицах
        let o = Vec2::new(
            self.origin.x * self.total_size.x * self.scale.x,
            self.origin.y * self.total_size.y * self.scale.y,
        );

        let offset_x = self.position.x - o.x;
        let mut offset_y = self.position.y - o.y;

        for l in &self.lines {
            if let Some(t) = &l.texture {
                // Если бэкенд поддерживает рисование текста в текстуру, то она берётся из кэша
                // и рисуется в этой ветке
                let align = offset_x
                    + (self.total_size.x - t.width as f32) * self.scale.x * self.text_align;
                // Тень текста
                if let Some(shadow_color) = self.shadow_color {
                    ctx.draw_texture_ex(
                        t,
                        DrawTextureParams {
                            position: Vec2::new(
                                (align + self.shadow_offset.x).round(),
                                (offset_y + self.shadow_offset.y).round(),
                            ),
                            scale: self.scale,
                            color: shadow_color,
                            ..Default::default()
                        },
                    )?;
                }
                // Сам текст
                ctx.draw_texture_ex(
                    t,
                    DrawTextureParams {
                        position: Vec2::new(align.round(), offset_y.round()),
                        scale: self.scale,
                        color: self.color,
                        ..Default::default()
                    },
                )?;
            } else if !l.line.is_empty() {
                // Эта ветка выполняется, если бэкенд не поддерживает рисование текста в текстуру
                let align = offset_x
                    + (self.total_size.x - l.size.x as f32) * self.scale.x * self.text_align;
                // Тень текста
                if let Some(shadow_color) = self.shadow_color {
                    ctx.draw_text(
                        &l.line,
                        &self.font,
                        shadow_color,
                        self.smooth,
                        Vec2::new(
                            (align + self.shadow_offset.x).round(),
                            (offset_y + self.shadow_offset.y).round(),
                        ),
                        self.scale,
                    )?;
                }
                // Сам текст
                ctx.draw_text(
                    &l.line,
                    &self.font,
                    self.color,
                    self.smooth,
                    Vec2::new(align.round(), offset_y.round()),
                    self.scale,
                )?;
            }
            offset_y += l.size.y * self.scale.y;
        }

        Ok(())
    }
}
