//! Контекст — через него выполняется всё взаимодействие с движком.
//!
//! Движок передаёт мутабельную ссылку на [`Context`][self::Context] в методы
//! [`process`](crate::scene::Scene::process) и [`render`](crate::scene::Scene::render) вашего
//! объекта, реализующего типаж [`Scene`](crate::scene::Scene), и через неё вы можете
//! рисовать, воспроизводить звуки, получать пользовательский ввод, смотреть частоту кадров
//! и вот это вот всё.
//!
//! В каком-то смысле контекст можно считать глобальной переменной, но так как контекст может
//! пересоздаваться вместе с пересозданием окна и вообще глобальные переменные это плохо, доступ
//! к нему осуществляется через параметр функции.
//!
//! Пример взаимодействия с контекстом смотрите в документации [`Scene`](crate::scene).

use crate::{
    audio::{Music, Sound},
    color::Color,
    font::Font,
    gametime::GameTime,
    input::Input,
    rect::Rect,
    texture::{Texture, TextureOptions, TextureSource},
    vec::Vec2,
    view::View,
};
use anyhow::Result;
use std::{path::Path, rc::Rc};

/// Параметры рисования текстуры.
pub struct DrawTextureParams {
    /// Источник — какую часть текстуры рисовать. Если не указано, то рисуется вся текстура.
    /// Можно указать отрицательные ширину и/или высоту — тогда текстура будет перевёрнута,
    /// а левый и/или верхний угол станет правым и/или нижним соответственно.
    pub src: Option<Rect>,
    /// Точка на рисуемой области текстуры, относительно которой будут выполняться преобразования.
    /// Значение 0.0 означает лево или верх, 1.0 — право или низ. Например, если указать
    /// `origin: Vec2::new(1.0, 1.0)`, то в координате `position` будет правый нижний угол,
    /// `origin: Vec2::new(0.5, 0.5)` — центр или `origin: Vec2::new(0.0, 0.0)` — левый верхний.
    pub origin: Vec2,
    /// Позиция, в которой рисовать текстуру. В зависимости от значений `origin`, `rotation`
    /// и `scale` текстура может быть смещена относительно этой позиции.
    pub position: Vec2,
    /// Поворот текстуры относительно `position` по часовой стрелке в градусах.
    pub rotation: f32,
    /// Масштабирование текстуры. Если не указано или `Vec2::new(1.0, 1.0)`, то используется
    /// размер текстуры как есть. Отрицательный размер переворачивает текстуру и изменяет
    /// направление её рисования.
    pub scale: Vec2,
    /// Цвет, на который будут умножены пиксели текстуры. Белый означает, что текстура будет
    /// отрисована как есть.
    pub color: Color,
}

impl Default for DrawTextureParams {
    fn default() -> Self {
        DrawTextureParams {
            src: None,
            origin: Vec2::new(0.0, 0.0),
            position: Vec2::new(0.0, 0.0),
            rotation: 0.0,
            scale: Vec2::new(1.0, 1.0),
            color: Color::WHITE,
        }
    }
}

/// Контекст хранит в себе всё состояние игры, и через него осуществляется всё взаимодействие
/// с движком.
///
/// Подробнее в [документации модуля](self).
pub trait Context {
    /// Название текущего бэкенда.
    fn get_backend_name(&self) -> &'static str;

    // time

    /// Объект с внутриигровым временем. Из него можно узнать, например, время запуска
    /// и частоту кадров.
    fn time(&self) -> &GameTime;

    /// Мутабельная ссылка на объект с внутриигровым временем.
    fn time_mut(&mut self) -> &mut GameTime;

    // input

    /// Объект, позволяющий узнать недавние события и нажатые клавиши.
    fn input(&self) -> &Input;

    // view

    /// Объект с информацией о системе координат игры.
    fn view(&self) -> &View;

    /// Мутабельная ссылка на объект с информацией о системе координат игры, которая позволяет
    /// вам изменить эту самую систему координат.
    fn view_mut(&mut self) -> &mut View;

    // window

    /// Возвращает коэффициент масштабирования экрана. Если HiDPI отсутствует или
    /// не поддерживается, возвращает просто число 1.
    fn get_dpi_scale(&self) -> Vec2 {
        Vec2::new(1.0, 1.0)
    }

    /// Возвращает размер окна в логических единицах (без учёта view, но с учётом HiDPI).
    fn get_logical_window_size(&self) -> Vec2 {
        let (w, h) = self.get_physical_window_size();
        let d = self.get_dpi_scale();
        Vec2::new(w as f32 / d.x, h as f32 / d.y)
    }

    /// Возвращает реальный размер окна в пикселях.
    fn get_physical_window_size(&self) -> (u32, u32);

    /// Возвращает `true`, если игра находится в полноэкранном режиме.
    fn get_fullscreen(&self) -> bool;

    /// Переход в полноэкранный режим или выход из него. В зависимости от платформы или бэкенда
    /// может не сработать или сработать криво.
    fn set_fullscreen(&mut self, fullscreen: bool) -> Result<()>;

    /// Включена ли вертикальная синхронизация.
    fn get_vsync(&self) -> bool;

    /// Включение или выключение вертикальной синхронизации. В зависимости от платформы или бэкенда
    /// может не сработать или сработать криво.
    fn set_vsync(&mut self, vsync: bool) -> Result<()>;

    /// Ограничение частоты кадров, ноль означает отсутствие ограничения.
    fn set_fps_limit(&mut self, value: f32);

    /// Возвращает `true`, если системный курсор мыши виден.
    fn get_mouse_cursor_visibility(&self) -> bool;

    /// Отображает или скрывает системный курсор мыши.
    fn set_mouse_cursor_visibility(&mut self, visible: bool) -> Result<()>;

    /// Включено ли преобразование событий касания в события мыши.
    fn is_simulating_mouse_with_touch(&self) -> bool;

    /// Включает или выключает преобразование событий касания в события мыши. Сами события касания
    /// при этом не отключаются, а сразу после них появляются симулированные события мыши, если
    /// преобразование включено.
    fn simulate_mouse_with_touch(&mut self, enabled: bool);

    // drawing

    /// Устанавливает текущий цвет рисования.
    fn set_fill_color(&mut self, color: Color);

    /// Очищает всё окно текущим цветом рисования.
    fn clear(&mut self) -> Result<()>;

    /// Рисует прямоугольник текущим цветом рисования. Допускаются отрицательные ширина
    /// и/или высота.
    fn fill_rect(&mut self, rect: Rect) -> Result<()>;

    /// Рисует текстуру в её реальном размере.
    fn draw_texture(&mut self, texture: &Texture, position: Vec2, origin: Vec2) -> Result<()> {
        let params = DrawTextureParams {
            position,
            origin,
            ..Default::default()
        };
        self.draw_texture_ex(texture, params)
    }

    /// Рисует текстуру с дополнительными параметрами.
    fn draw_texture_ex(&mut self, texture: &Texture, params: DrawTextureParams) -> Result<()>;

    /// Рисует строку текста в текстуру. Если не поддерживается бэкендом, возвращает `None`.
    fn draw_text_to_texture(
        &mut self,
        text: &str,
        font: &Font,
        color: Color,
        smooth: bool,
    ) -> Result<Option<Rc<Texture>>>;

    /// Рисует строку текста (многострочность не поддерживается).
    fn draw_text(
        &mut self,
        text: &str,
        font: &Font,
        color: Color,
        smooth: bool,
        position: Vec2,
        scale: Vec2,
    ) -> Result<()>;

    // audio

    /// Воспроизводит музыку. Одновременно может играть только одна музыка; если уже играет
    /// какая-то музыка, она будет остановлена.
    fn play_music(&mut self, music: &Music, volume: f32, looping: bool) -> Result<()>;

    /// Останавливает музыку.
    fn stop_music(&mut self) -> Result<()>;

    /// Возвращает текущую воспроизводимую музыку.
    fn get_playing_music(&self) -> Result<Option<Rc<Music>>>;

    /// Воспроизводит звук. Можно одновременно воспроизводить несколько звуков, но эти звуки
    /// должны быть разными.
    fn play_sound(&mut self, sound: &Sound, volume: f32, looping: bool) -> Result<()>;

    /// Останавливает звук.
    fn stop_sound(&mut self, sound: &Sound) -> Result<()>;

    // resources

    /// Выгружает все неиспользуемые ресурсы (текстуры, шрифты, музыку и звуки).
    ///
    /// Ресурсы считаются неиспользуемыми, если на них нет ссылок, помимо внутренних ссылок движка.
    /// Воспроизводящиеся звуки, не имеющие ссылок, тоже считаются неиспользуемыми, поэтому они
    /// будут остановлены и выгружены.
    fn drop_unused_resources(&mut self) {
        self.drop_unused_textures();
        self.drop_unused_fonts();
        self.drop_unused_music();
        self.drop_unused_sounds();
    }

    /// Перезагружает все ресурсы, загруженные с учётом языка (на данный момент это только
    /// текстуры).
    fn reload_lang_resources(&mut self) -> Result<()> {
        self.reload_lang_textures()?;
        Ok(())
    }

    // resources - textures

    /// Проверяет, загружена ли текстура и пригодна ли для использования.
    ///
    /// Текстуры могут пропадать при пересоздании окна. Текстуры из файлов бэкенд может
    /// перезагрузить сам, а вот текстуры, созданные на лету (например, из `draw_text_to_texture`),
    /// придётся пересоздать заново.
    fn is_texture_valid(&self, texture: &Texture) -> bool;

    /// Загружает текстуру.
    fn load_texture(
        &mut self,
        source: TextureSource,
        options: TextureOptions,
    ) -> Result<Rc<Texture>>;

    /// Загружает текстуру из файла (обёртка над `load_texture` для упрощения кода).
    fn load_texture_file(&mut self, path: &Path, options: TextureOptions) -> Result<Rc<Texture>> {
        self.load_texture(TextureSource::File(path.to_path_buf()), options)
    }

    /// Выгружает текстуру из памяти, если она не используется. Возвращает `true`, если текстура
    /// действительно была выгружена.
    fn drop_texture_if_unused(&mut self, texture: Rc<Texture>) -> bool;

    /// Выгружает все неиспользуемые текстуры.
    fn drop_unused_textures(&mut self);

    /// Перезагружает текстуры, загруженные с учётом языка. Можно использовать для загрузки других
    /// текстур после переключения языка.
    ///
    /// Размер всех вариантов текстуры должен быть одинаковым, так как при перезагрузке он
    /// не обновляется.
    fn reload_lang_textures(&mut self) -> Result<()>;

    // resources - fonts

    /// Загружает TTF-шрифт из файла.
    fn load_ttf_file(&mut self, path: &Path, size: u16) -> Result<Rc<Font>>;

    /// Выгружает все неиспользуемые шрифты.
    fn drop_unused_fonts(&mut self);

    /// Возвращает размер указанной строки текста в пикселях.
    fn get_text_size(&self, text: &str, font: &Font) -> Result<Vec2>;

    /// Возвращает ascent (высота от верхнего края строки текста до базовой линии) и descent
    /// (высота от базовой линии до нижнего края строки текста).
    fn get_font_metrics(&self, font: &Font) -> Result<(f32, f32)>;

    /// Возвращает рекомендуемую высоту строки текста. В зависимости от шрифта и бэкенда может
    /// как совпадать с суммой ascent и descent, так и отличаться.
    fn get_font_line_height(&self, font: &Font) -> Result<f32> {
        let (ascent, descent) = self.get_font_metrics(font)?;
        Ok(ascent + descent)
    }

    // resources - audio

    /// Загружает музыку из файла. Некоторые бэкенды умеют подгружать музыку на лету во время
    /// воспроизведения, поэтому файл должен оставаться доступен в течение работы игры.
    fn load_music_file(&mut self, path: &Path) -> Result<Rc<Music>>;

    /// Останавливает и выгружает неиспользуемую музыку.
    fn drop_unused_music(&mut self);

    /// Загружает звук из файла.
    fn load_sound_file(&mut self, path: &Path) -> Result<Rc<Sound>>;

    /// Останавливает и выгружает все неиспользуемые звуки.
    fn drop_unused_sounds(&mut self);
}
