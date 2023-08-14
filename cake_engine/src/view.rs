//! Управление системой координат игры.
//!
//! Под фразой «система координат» имеется в виду [прямоугольник](crate::rect::Rect), который
//! задаёт координату верхнего левого угла области рисования, ширину до правого края и высоту до
//! нижнего края. То, как этот прямоугольник будет расположен в окне, зависит от размеров окна.
//! Если он окажется меньше размеров окна, то рисовать за пределами прямоугольника (но в пределах
//! окна) тоже можно.
//!
//! [Контекст](crate::context) при рисовании использует [View](self::View), чтобы преобразовать
//! координаты, которые вы используете в своей игре, в координаты окна. По сути, это задаёт
//! видимую область игры.
//!
//! Систему координат можно изменять в любой момент в процессе рендеринга, что позволяет вам
//! проворачивать удобные штуки: например, рисовать игровой мир в его собственной системе
//! с масштабированием и перемещением по координатам «камеры», а интерфейс игры рисовать
//! в системе координат окна.
//!
//! Главное — не забывайте, что последнее заданное значение используется при обработке событий:
//! если вы меняете view на лету, после рендеринга не забудьте задать тот view, в системе координат
//! которого вы ожидаете координаты курсора мыши.
//!
//! Технические ограничения — нельзя вращать и делать неквадратное соотношение сторон.
//!
//! Рассмотрим пример: вы установили область 1024x768, и она вписывается в окно 1280x720 с режимом
//! Letterbox. Так как высота окна меньше высоты области, область немного уменьшается, а по краям
//! слева и справа остаются пустоты. Тогда координаты и размеры будут примерно такие:
//!
//! ```none
//!   X: 0 — окно  160 — окно                   1120 — окно     1280 — окно
//! -170.7 — игра  0 — игра                     1024 — игра     1194.7 — игра
//!   +--------------+----------------------------+--------------+  Y: 0 — окно
//!   |              |                            |              |     0 — игра
//!   |              |                            |              |
//!   |              |          Область           |              |
//!   |   Пустота    |       1024x768 — игра      |   Пустота    |
//!   |              |       960x720 — окно       |              |
//!   |              |                            |              |
//!   |              |                            |              |
//!   |              |                            |              |    720 — окно
//!   +--------------+----------------------------+--------------+    768 — игра
//! ```
//!
//! Узнать такие координаты в коде можно примерно так:
//!
//! ```
//! use cake_engine::{rect::Rect, vec::Vec2, view::ViewMode};
//! # use cake_engine::{conf::Conf, context::Context, dummy::DummyContext};
//! # let mut dctx = DummyContext::new(&Conf::default());
//! # let mut ctx: &mut dyn Context = &mut dctx;
//! # ctx.view_mut().set_target_size(Vec2::new(1280.0, 720.0));
//!
//! // Установка области
//! ctx.view_mut().set(Some(Rect::new(0.0, 0.0, 1024.0, 768.0)));
//! ctx.view_mut().set_mode(ViewMode::Letterbox);
//!
//! // Получение координат окна в системе координат нашей игры
//! let visible_area = ctx.view().visible_area();
//! // Левый край окна
//! assert_eq!(format!("{:.1}", visible_area.x), "-170.7");
//! // Правый край окна
//! assert_eq!(format!("{:.1}", visible_area.x + visible_area.width), "1194.7");
//!
//! // Получение позиции нашей области в системе координат окна
//! let area_in_window = ctx.view().rect_to_target(ctx.view().get().unwrap());
//! // Левый край области
//! assert_eq!(format!("{:.1}", area_in_window.x), "160.0");
//! // Правый край области
//! assert_eq!(format!("{:.1}", area_in_window.x + area_in_window.width), "1120.0");
//!
//! // Получение масштаба
//! let scale = ctx.view().get_scale();
//! // Область немного уменьшилась, чтобы вписаться в окно
//! assert_eq!(format!("{:.4}", scale.x), "0.9375");
//! ```

use crate::{rect::Rect, vec::Vec2};

/// Способ вписывания области игры в окно.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ViewMode {
    /// Растянуть на всё окно — не влезшее по краям будет не видно.
    Overscan,
    /// Вписать в окно — будут пустоты по краям (но вы можете рисовать в них).
    Letterbox,
    // Stretch убран — изменение соотношения сторон ломает вращение
}

/// Управление системой координат игры.
///
/// Подробнее в [документации модуля](self).
#[derive(Clone)]
pub struct View {
    target_size: Vec2,
    view: Option<Rect>,
    mode: ViewMode,
    kx: f32,
    ky: f32,
    offset_x: f32,
    offset_y: f32,
    changed: bool,
}

impl View {
    /// Создаёт новый объект `View`, если он вам зачем-то вдруг понадобился.
    ///
    /// `target_size` — размер области в целевой системе координат (стандартный контекст
    /// в качестве «цели» использует окно).
    pub fn new(target_size: Vec2) -> View {
        View {
            target_size,
            view: None,
            mode: ViewMode::Letterbox,
            kx: 1.0,
            ky: 1.0,
            offset_x: 0.0,
            offset_y: 0.0,
            changed: false,
        }
    }

    /// Возвращает `true`, если view был изменён только что. Среди причин может быть как изменение
    /// самого view, так и изменение размера окна.
    pub fn is_changed(&self) -> bool {
        self.changed
    }

    /// Сбрасывает флаг только что проведённого изменения view. Не вызывайте этот метод у объекта,
    /// принадлежащего контексту, иначе рендеринг может стать кривой.
    pub fn clear_changed_flag(&mut self) {
        self.changed = false;
    }

    fn recalc_scale(&mut self) {
        let view = match self.view {
            Some(v) => v,
            None => {
                self.offset_x = 0.0;
                self.offset_y = 0.0;
                self.kx = 1.0;
                self.ky = 1.0;
                return;
            }
        };

        let kx = self.target_size.x / view.width;
        let ky = self.target_size.y / view.height;

        match self.mode {
            ViewMode::Letterbox => {
                let k = if kx.abs() < ky.abs() { kx } else { ky };
                self.kx = k;
                self.ky = k;
            }
            ViewMode::Overscan => {
                let k = if kx.abs() > ky.abs() { kx } else { ky };
                self.kx = k;
                self.ky = k;
            }
            // ViewMode::Stretch => {
            //     self.kx = kx;
            //     self.ky = ky;
            // },
        }

        let w = view.width * self.kx;
        let h = view.height * self.ky;
        self.offset_x = (self.target_size.x - w) / 2.0;
        self.offset_y = (self.target_size.y - h) / 2.0;
    }

    /// Возвращает размер цели (контекст в качестве «цели» использует окно).
    pub fn get_target_size(&self) -> Vec2 {
        self.target_size
    }

    /// Устанавливает размер цели. Возвращает `true`, если размер изменился, или `false`, если нет.
    ///
    /// Не вызывайте этот метод у объекта, принадлежащего контексту: он предполагает, что размер
    /// цели совпадает с размером окна, и если вы это измените, то рендеринг может стать кривой.
    pub fn set_target_size(&mut self, size: Vec2) -> bool {
        // TODO: approx_eq
        if self.target_size.x == size.x && self.target_size.y == size.y {
            return false;
        }
        self.target_size = size;
        self.recalc_scale();
        self.changed = true;
        true
    }

    /// Возвращает текущую систему координат. Если её нет, значит преобразование координат
    /// не выполняется.
    pub fn get(&self) -> Option<Rect> {
        self.view
    }

    /// Возвращает текущую систему координат. Если её нет, то возвращает систему координат
    /// цели (если речь о view из контекста, то это будет размер окна).
    pub fn get_or_default(&self) -> Rect {
        match self.view {
            Some(v) => v,
            None => Rect::new(0.0, 0.0, self.target_size.x, self.target_size.y),
        }
    }

    /// Устанавливает новую систему координат.
    pub fn set(&mut self, view: Option<Rect>) {
        self.view = view;
        self.changed = true;
        self.recalc_scale();
    }

    /// Текущий режим вписывания системы координат в цель.
    pub fn get_mode(&self) -> ViewMode {
        self.mode
    }

    /// Меняет режим вписывания системы координат в цель.
    pub fn set_mode(&mut self, mode: ViewMode) {
        self.mode = mode;
        self.changed = true;
        self.recalc_scale();
    }

    /// Возвращает текущий коэффициент масштабирования из текущей системы координат в систему
    /// координат цели. Если ширина и высота совпадают, то масштаб будет единица.
    pub fn get_scale(&self) -> Vec2 {
        Vec2::new(self.kx, self.ky)
    }

    /// Переводит точку из своей системы координат в систему координат цели (с учётом смещения).
    pub fn point_to_target(&self, point: Vec2) -> Vec2 {
        let v = match self.view {
            Some(v) => v,
            None => return point,
        };
        Vec2::new(
            (point.x - v.x) * self.kx + self.offset_x,
            (point.y - v.y) * self.ky + self.offset_y,
        )
    }

    /// Переводит размер из своей системы координат в систему координат цели (без учёта смещения).
    pub fn size_to_target(&self, size: Vec2) -> Vec2 {
        if let None = self.view {
            return size;
        }
        Vec2::new(size.x * self.kx, size.y * self.ky)
    }

    /// Переводит прямоугольник из своей системы координат в систему координат цели.
    pub fn rect_to_target(&self, rect: Rect) -> Rect {
        let v = match self.view {
            Some(v) => v,
            None => return rect,
        };
        Rect::new(
            (rect.x - v.x) * self.kx + self.offset_x,
            (rect.y - v.y) * self.ky + self.offset_y,
            rect.width * self.kx,
            rect.height * self.ky,
        )
    }

    /// Переводит точку из системы координат цели в свою систему координат (с учётом смещения).
    pub fn point_from_target(&self, point: Vec2) -> Vec2 {
        let v = match self.view {
            Some(v) => v,
            None => return point,
        };
        Vec2::new(
            ((point.x - self.offset_x) / self.kx) + v.x,
            ((point.y - self.offset_y) / self.ky) + v.y,
        )
    }

    /// Переводит размер из системы координат цели в свою систему координат (без учёта смещения).
    pub fn size_from_target(&self, size: Vec2) -> Vec2 {
        if let None = self.view {
            return size;
        }
        Vec2::new(size.x / self.kx, size.y / self.ky)
    }

    /// Переводит прямоугольник из системы координат цели в свою систему координат.
    pub fn rect_from_target(&self, rect: Rect) -> Rect {
        let v = match self.view {
            Some(v) => v,
            None => return rect,
        };
        Rect::new(
            ((rect.x - self.offset_x) / self.kx) + v.x,
            ((rect.y - self.offset_y) / self.ky) + v.y,
            rect.width / self.kx,
            rect.height / self.ky,
        )
    }

    /// Возвращает область, которая фактически видна в цели, в своей системе координат.
    pub fn visible_area(&self) -> Rect {
        self.rect_from_target(Rect::new(0.0, 0.0, self.target_size.x, self.target_size.y))
    }
}
