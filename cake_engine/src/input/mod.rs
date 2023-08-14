//! Штуки для работы с вводом и другими событиями, приходящими извне.

mod actions;
mod scancode;

pub use self::{actions::*, scancode::*};

use crate::vec::Vec2;
use std::collections::{HashMap, HashSet};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Кнопки мыши.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum MouseButton {
    Unknown,
    Left,
    Right,
    Middle,
    XButton1,
    XButton2,
}

/// Клавиши вместе с кнопками мыши.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Key {
    Keyboard(ScanCode),
    Mouse(MouseButton),
}

/// Стадия касания.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum TouchPhase {
    Started,
    Moved,
    Ended,
    Cancelled,
}

/// Клавиши модификаторы, которые были зажаты в момент нажатия/отпускания клавиши.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct KeyMods {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub logo: bool, // super
}

/// Событие. Набор поддерживаемых событий зависит от бэкенда.
#[derive(Clone, Copy, Debug)]
pub enum Event {
    /// Пользователь запросил выход (например, нажал Ctrl+C или кликнул в крестик у окна).
    Quit,
    /// Курсор мыши оказался над окном.
    MouseEnter,
    /// Курсор мыши оказался за пределами окна.
    MouseLeave,
    /// Курсор мыши переместился, `point` содержит новую позицию в системе координат игры
    /// (с учётом view). Если указан `touch_id`, значит событие преобразовано из события касания.
    MouseMove { point: Vec2, touch_id: Option<u64> },
    /// Прокрутка колёсиком мыши. Знак чисел указывает направление прокрутки
    /// (`delta.x` минус — влево, плюс — вправо; `delta.y` минус — вниз, плюс — вверх).
    MouseWheel { delta: Vec2 },
    /// Зажата кнопка мыши, `point` содержит позицию курсора в системе координат игры
    /// (с учётом view). Если указан `touch_id`, значит событие преобразовано из события касания.
    MouseDown {
        button: MouseButton,
        point: Vec2,
        touch_id: Option<u64>,
    },
    /// Отпущена кнопка мыши, `point` содержит позицию курсора в системе координат игры
    /// (с учётом view). Если указан `touch_id`, значит событие преобразовано из события касания.
    MouseUp {
        button: MouseButton,
        point: Vec2,
        touch_id: Option<u64>,
    },
    /// Касание сенсорного экрана.
    Touch {
        phase: TouchPhase,
        id: u64,
        point: Vec2,
    },
    /// Нажата клавиша на клавиатуре. Если пользователь долго её не отпускает, событие будет
    /// периодически повторяться с `repeat: true`. Сканкод указывает на физический код клавиши,
    /// не зависящий от текущей раскладки.
    KeyDown {
        scancode: ScanCode,
        repeat: bool,
        mods: KeyMods,
    },
    /// Отпущена клавиша на клавиатуре. Сканкод указывает на физический код клавиши, не зависящий
    /// от текущей раскладки.
    KeyUp { scancode: ScanCode, mods: KeyMods },
    /// Введён текст. В отличие от сканкодов в событиях `KeyDown` и `KeyUp`, здесь учитываются
    /// раскладка и капс.
    Character { character: char },
    /// Изменился размер окна, `logical_size` содержит новый размер в логических единицах измерения
    /// (без учёта view, но с учётом HiDPI, если он поддерживается бэкендом).
    Resize { logical_size: Vec2 },
    /// Окно получило фокус.
    FocusIn,
    /// Окно потеряло фокус.
    FocusOut,
    /// Окно свёрнуто.
    Minimized,
    /// Окно развёрнуто на весь экран.
    Maximized,
    /// Окно перешло в обычное состояние (не свёрнуто и не развёрнуто).
    Restored,
}

/// Объект для удобного чтения ввода и недавних событий.
///
/// Хранит в себе недавно произошедшие события и зажатые клавиши и предоставляет набор методов,
/// позволяющих удобно проверять текущее состояние ввода без необходимости проходиться по всему
/// массиву событий и хранить всё самостоятельно.
///
/// События, произошедшие «только что», сбрасываются после завершения обработки текущего кадра.
///
/// Для управления игрой рекомендуется использовать абстракцию [`Actions`](self::Actions), чтобы
/// не хардкодить клавиши в коде игры и позволить игроку настраивать управление под себя.
///
/// # Examples
///
/// ```
/// # use cake_engine::{conf::Conf, context::Context, dummy::DummyContext};
/// # let mut dctx = DummyContext::new(&Conf::default());
/// # let mut ctx: &mut dyn Context = &mut dctx;
/// # let old_speed = Vec2::new(0.0, 0.0);
/// use cake_engine::{input::ScanCode, vec::Vec2};
///
/// let mut new_speed = Vec2::new(0.0, old_speed.y);
/// if ctx.input().is_key_pressed(ScanCode::Left) {
///     new_speed.x -= 100.0;
/// }
/// if ctx.input().is_key_pressed(ScanCode::Right) {
///     new_speed.x += 100.0;
/// }
/// if ctx.input().is_key_just_pressed(ScanCode::Space) {
///     new_speed.y -= 100.0;
/// }
/// ```
pub struct Input {
    // Штуки, происходящие в данный момент
    mouse_position: Vec2,
    mouse_entered: bool,
    touch_positions: HashMap<u64, Vec2>,
    active_touch_ids: Vec<u64>,
    pressed_mouse_buttons: HashSet<MouseButton>,
    pressed_keys: HashSet<ScanCode>,
    has_focus: bool,
    // Штуки, случившиеся «только что» (сбрасываются после завершения обработки текущего кадра)
    quit_requested: bool,
    just_pressed_mouse_buttons: HashSet<MouseButton>,
    just_released_mouse_buttons: HashSet<MouseButton>,
    just_pressed_keys: HashSet<ScanCode>,
    just_released_keys: HashSet<ScanCode>,
    just_resized_window_size: Option<Vec2>,
}

impl Input {
    pub fn new() -> Input {
        Input {
            mouse_position: Vec2::new(0.0, 0.0),
            mouse_entered: true,
            touch_positions: HashMap::new(),
            active_touch_ids: Vec::new(),
            pressed_mouse_buttons: HashSet::new(),
            pressed_keys: HashSet::new(),
            has_focus: true,
            quit_requested: false,
            just_pressed_mouse_buttons: HashSet::new(),
            just_released_mouse_buttons: HashSet::new(),
            just_pressed_keys: HashSet::new(),
            just_released_keys: HashSet::new(),
            just_resized_window_size: None,
        }
    }

    /// Возвращает `true`, если пользователь только что запросил вывод.
    pub fn is_quit_requested(&self) -> bool {
        self.quit_requested
    }

    /// Возвращает `true`, если курсор мыши в данный момент наведён на окно.
    pub fn is_mouse_entered(&self) -> bool {
        self.mouse_entered
    }

    /// Возвращает текущую позицию курсора мыши в системе координат игры (с учётом view).
    ///
    /// Если курсор на данный момент находится за пределами окна, то возвращает последнюю
    /// сохранённую позицию.
    ///
    /// Имейте в виду, что значение обновляется только в момент получения событий, связанных
    /// с курсором мыши. Если вы обновили систему координат игры, изменив view, то значение
    /// может быть устаревшим до получения следующего события мыши.
    pub fn get_mouse_position(&self) -> Vec2 {
        self.mouse_position
    }

    /// Возвращает `true`, если указанная кнопка мыши зажата в данный момент.
    pub fn is_mouse_button_pressed(&self, button: MouseButton) -> bool {
        self.pressed_mouse_buttons.contains(&button)
    }

    /// Возвращает `true`, если указанная кнопка мыши была зажата только что.
    pub fn is_mouse_button_just_pressed(&self, button: MouseButton) -> bool {
        self.just_pressed_mouse_buttons.contains(&button)
    }

    /// Возвращает `true`, если указанная кнопка мыши была отпущена только что.
    pub fn is_mouse_button_just_released(&self, button: MouseButton) -> bool {
        self.just_released_mouse_buttons.contains(&button)
    }

    /// Все кнопки мыши, зажатые только что.
    pub fn just_pressed_mouse_buttons(&self) -> &HashSet<MouseButton> {
        &self.just_pressed_mouse_buttons
    }

    /// Все зажатые в данный момент кнопки мыши.
    pub fn pressed_mouse_buttons(&self) -> &HashSet<MouseButton> {
        &self.pressed_mouse_buttons
    }

    /// Возвращает `true`, если указанная клавиша зажата в данный момент.
    pub fn is_key_pressed(&self, scancode: ScanCode) -> bool {
        self.pressed_keys.contains(&scancode)
    }

    /// Возвращает `true`, если указанная клавиша была зажата только что.
    pub fn is_key_just_pressed(&self, scancode: ScanCode) -> bool {
        self.just_pressed_keys.contains(&scancode)
    }

    /// Возвращает `true`, если указанная клавиша была отпущена только что.
    pub fn is_key_just_released(&self, scancode: ScanCode) -> bool {
        self.just_released_keys.contains(&scancode)
    }

    /// Все клавиши, зажатые только что.
    pub fn just_pressed_keys(&self) -> &HashSet<ScanCode> {
        &self.just_pressed_keys
    }

    /// Все зажатые в данный момент клавиши.
    pub fn pressed_keys(&self) -> &HashSet<ScanCode> {
        &self.pressed_keys
    }

    /// Если размер окна был изменён только что, возвращает его новый размер в логических
    /// единицах измерения (без учёта view, но с учётом HiDPI, если он поддерживается бэкендом).
    /// Если размер окна не менялся в текущем кадре, возвращает `None`.
    pub fn is_window_just_resized(&self) -> Option<Vec2> {
        self.just_resized_window_size
    }

    /// Добавляет отметку об изменившемся размере окна.
    pub fn mark_window_as_resized(&mut self, size: Vec2) {
        self.just_resized_window_size = Some(size);
    }

    /// Находится ли окно в фокусе в данный момент.
    pub fn has_focus(&self) -> bool {
        self.has_focus
    }

    /// Удаляет информацию о событиях, произошедших только что. Длительные события (например,
    /// зажатые клавиши) остаются.
    pub fn clear(&mut self) {
        self.quit_requested = false;
        self.just_pressed_mouse_buttons.clear();
        self.just_released_mouse_buttons.clear();
        self.just_pressed_keys.clear();
        self.just_released_keys.clear();
        self.just_resized_window_size = None;
    }

    /// Удаляет вообще всю информацию, кроме позиции курсора мыши. Зажатые клавиши перестанут
    /// считаться зажатыми.
    pub fn reset(&mut self) {
        self.clear();
        self.pressed_mouse_buttons.clear();
        self.pressed_keys.clear();
        self.touch_positions.clear();
        self.active_touch_ids.clear();
        self.has_focus = true;
    }

    /// Обработка событий.
    pub fn handle_events<'a, I: IntoIterator<Item = &'a Event>>(&mut self, events: I) {
        for event in events {
            match event {
                Event::MouseEnter => {
                    self.mouse_entered = true;
                }
                Event::MouseLeave => {
                    self.mouse_entered = false;
                }
                Event::MouseMove {
                    point, touch_id, ..
                } => {
                    if let Some(touch_id) = touch_id {
                        if self.active_touch_ids.last() == Some(touch_id) {
                            // Записываем координаты только последнего пальца, чтобы мультитач
                            // не вызывал скакание координат туда-сюда
                            self.mouse_position = *point;
                        }
                    } else {
                        // Настоящую мышку записываем как обычно
                        self.mouse_position = *point;
                    }
                }
                Event::MouseDown {
                    button,
                    point,
                    touch_id,
                    ..
                } => {
                    self.just_pressed_mouse_buttons.insert(*button);
                    self.pressed_mouse_buttons.insert(*button);
                    if let Some(touch_id) = touch_id {
                        if !self.active_touch_ids.contains(touch_id) {
                            self.active_touch_ids.push(*touch_id);
                        }
                        self.touch_positions.insert(*touch_id, *point);
                        if self.active_touch_ids.last() == Some(touch_id) {
                            // Записываем координаты только последнего пальца, чтобы мультитач
                            // не вызывал скакание координат туда-сюда
                            self.mouse_position = *point;
                        }
                    } else {
                        self.mouse_position = *point;
                    }
                }
                Event::MouseUp {
                    button,
                    point,
                    touch_id,
                    ..
                } => {
                    if let Some(touch_id) = touch_id {
                        if let Some(idx) = self.active_touch_ids.iter().position(|t| t == touch_id)
                        {
                            self.active_touch_ids.remove(idx);
                        }
                        self.touch_positions.remove(touch_id);
                        // Если был отпущен последний палец, то текущей позицией мышки считаем
                        // позицию предпоследнего пальца (если он есть)
                        if let Some(prev_touch_id) = self.active_touch_ids.last() {
                            self.mouse_position = self.touch_positions[prev_touch_id];
                        } else {
                            self.mouse_position = *point;
                        }
                    } else {
                        self.mouse_position = *point;
                    }

                    // Считаем левую кнопку мыши нажатой, пока в экран тыкает хотя бы один палец
                    if *button != MouseButton::Left || self.active_touch_ids.is_empty() {
                        self.just_released_mouse_buttons.insert(*button);
                        self.pressed_mouse_buttons.remove(button);
                    }
                }
                Event::KeyDown {
                    scancode, repeat, ..
                } => {
                    if !repeat {
                        self.just_pressed_keys.insert(*scancode);
                    }
                    self.pressed_keys.insert(*scancode);
                }
                Event::KeyUp { scancode, .. } => {
                    self.just_released_keys.insert(*scancode);
                    self.pressed_keys.remove(scancode);
                }
                Event::Quit => {
                    self.quit_requested = true;
                }
                Event::Resize { logical_size } => {
                    self.mark_window_as_resized(*logical_size);
                }
                Event::FocusOut => {
                    self.has_focus = false;
                }
                Event::FocusIn => {
                    self.has_focus = true;
                }
                _ => {}
            }
        }
    }
}
