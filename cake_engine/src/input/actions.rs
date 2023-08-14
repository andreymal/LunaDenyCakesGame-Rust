use crate::{
    context::Context,
    input::{Event, Key},
};
use std::{collections::HashMap, hash::Hash};

/// Действия — абстракция, позволяющая отвязать код игры от конкретных физических клавиш.
///
/// Структура `Actions` хранит сопоставления внутриигровых действий (движение, прыжок, стрельба
/// и т. п.) с конкретными клавишами клавиатуры или кнопками мыши (WASD, пробел, ЛКМ и т. п.).
/// Вы можете менять эти сопоставления в любой момент и таким образом, например, предоставить
/// игроку изменить управление под себя, а в коде игры не будут захардкожены конкретные клавиши.
///
/// Сопоставление выполняется один к одному: на одной клавише может быть только одно действие,
/// и на одном действии может быть только одна клавиша.
///
/// # Examples
///
/// Создайте enum, содержащий все действия вашей игры и реализующий `Copy + Eq + PartialEq + Hash`,
/// и экземпляр Actions для него и загрузите в него ваши настройки управления.
///
/// ```
/// use std::collections::HashMap;
/// use cake_engine::input::{Actions, Key, MouseButton, ScanCode};
/// # use cake_engine::{conf::Conf, context::Context, dummy::DummyContext};
/// # let mut dctx = DummyContext::new(&Conf::default());
/// # let mut ctx: &mut dyn Context = &mut dctx;
///
/// // Все возможные действия в игре
/// #[derive(Copy, Clone, Hash, PartialEq, Eq)]
/// pub enum Action {
///     Left,
///     Right,
///     Jump,
///     Fire,
/// }
///
/// // Настройки управления: какая клавиша с каким действием связана
/// let mut input_keys = HashMap::new();
/// input_keys.insert(Action::Left, Key::Keyboard(ScanCode::A));
/// input_keys.insert(Action::Right, Key::Keyboard(ScanCode::D));
/// input_keys.insert(Action::Jump, Key::Keyboard(ScanCode::Space));
/// input_keys.insert(Action::Fire, Key::Mouse(MouseButton::Left));
///
/// let mut actions: Actions<Action> = Actions::new();
/// actions.replace_all(&input_keys);
///
/// // Теперь внутри метода process вашей сцены вы можете проверять,
/// // какому действию соответствует событие
/// # use cake_engine::input::{Event, KeyMods};
/// # let events_arr = [Event::KeyDown { scancode: ScanCode::Space, mods: KeyMods::default(), repeat: false }];
/// # let events = &events_arr;
/// for event in events {
///     match actions.match_event(*event) {
///         Some((Action::Left, pressed)) => {
///             if pressed {
///                 println!("Начали двигаться влево");
///             } else {
///                 println!("Закончили двигаться влево");
///             }
///         }
///         _ => { /* ... */ }
///     }
/// }
///
/// // Или проверять, активно ли интересующее действие в данный момент
/// if actions.pressed(ctx, Action::Left) {
///     println!("Двигаемся влево");
/// } else if actions.just_released(ctx, Action::Left) {
///     println!("Закончили двигаться влево");
/// }
/// ```
#[derive(Clone, Debug)]
pub struct Actions<T: Copy + Eq + PartialEq + Hash> {
    mapping: HashMap<T, Key>,
    keys_mapping: HashMap<Key, T>,
}

impl<T: Copy + Eq + PartialEq + Hash> Actions<T> {
    /// Создаёт новый экземпляр `Actions`.
    pub fn new() -> Actions<T> {
        Actions {
            mapping: HashMap::new(),
            keys_mapping: HashMap::new(),
        }
    }

    /// Текущее сопоставление действий с клавишами.
    pub fn mapping(&self) -> &HashMap<T, Key> {
        &self.mapping
    }

    /// Текущее сопоставление клавиш с действиями.
    pub fn keys_mapping(&self) -> &HashMap<Key, T> {
        &self.keys_mapping
    }

    /// Очищает все имеющиеся сопоставления и загружает новые из указанного HashMap.
    pub fn replace_all(&mut self, mapping: &HashMap<T, Key>) {
        self.mapping.clear();
        self.keys_mapping.clear();
        for (action, key) in mapping {
            if let Some(old_key) = self.mapping.insert(*action, *key) {
                self.keys_mapping.remove(&old_key);
            }
            self.keys_mapping.insert(*key, *action);
        }
    }

    /// Добавляет новое сопоставление между клавишей и действием. Если клавиша и/или сопоставление
    /// уже используются, старое сопоставление будет удалено.
    pub fn add(&mut self, action: T, key: Key) {
        if self.mapping.get(&action) != Some(&key) {
            if let Some(old_key) = self.mapping.insert(action, key) {
                self.keys_mapping.remove(&old_key);
            }
            if let Some(old_action) = self.keys_mapping.insert(key, action) {
                self.mapping.remove(&old_action);
            }
        }
    }

    /// Удаляет действие, освобождая связанную с ним клавишу.
    pub fn remove_action(&mut self, action: T) {
        if let Some(old_key) = self.mapping.remove(&action) {
            self.keys_mapping.remove(&old_key);
        }
    }

    /// Удаляет клавишу и связанное с ней действие.
    pub fn remove_key(&mut self, key: Key) {
        if let Some(old_action) = self.keys_mapping.remove(&key) {
            self.mapping.remove(&old_action);
        }
    }

    /// Какая клавиша назначена на указанное действие.
    pub fn get_key_by_action(&self, action: T) -> Option<Key> {
        self.mapping.get(&action).copied()
    }

    /// Какое действие связано с указанной клавишей.
    pub fn get_action_by_key(&self, key: Key) -> Option<T> {
        self.keys_mapping.get(&key).copied()
    }

    /// Возвращает `true`, если действие только что началось (то есть клавиша была нажата
    /// только что).
    pub fn just_pressed(&self, ctx: &mut dyn Context, action: T) -> bool {
        let key = match self.mapping.get(&action) {
            Some(key) => key,
            None => return false,
        };
        match key {
            Key::Keyboard(scancode) => ctx.input().is_key_just_pressed(*scancode),
            Key::Mouse(button) => ctx.input().is_mouse_button_just_pressed(*button),
        }
    }

    /// Возвращает `true`, если действие только что прекратилось (то есть клавиша была отпущена
    /// только что).
    pub fn just_released(&self, ctx: &mut dyn Context, action: T) -> bool {
        let key = match self.mapping.get(&action) {
            Some(key) => key,
            None => return false,
        };
        match key {
            Key::Keyboard(scancode) => ctx.input().is_key_just_released(*scancode),
            Key::Mouse(button) => ctx.input().is_mouse_button_just_released(*button),
        }
    }

    /// Возвращает `true`, если действие активно в данный момент (то есть клавиша зажата).
    pub fn pressed(&self, ctx: &mut dyn Context, action: T) -> bool {
        let key = match self.mapping.get(&action) {
            Some(key) => key,
            None => return false,
        };
        match key {
            Key::Keyboard(scancode) => ctx.input().is_key_pressed(*scancode),
            Key::Mouse(button) => ctx.input().is_mouse_button_pressed(*button),
        }
    }

    /// Для событий клавиатуры и мыши возвращает связанное с ними действие, если оно есть, а также
    /// `true`, если действие началось, или `false`, если действие закончилось.
    pub fn match_event(&self, event: Event) -> Option<(T, bool)> {
        match event {
            Event::KeyDown { scancode, .. } => {
                if let Some(action) = self.keys_mapping.get(&Key::Keyboard(scancode)).copied() {
                    Some((action, true))
                } else {
                    None
                }
            }
            Event::KeyUp { scancode, .. } => {
                if let Some(action) = self.keys_mapping.get(&Key::Keyboard(scancode)).copied() {
                    Some((action, false))
                } else {
                    None
                }
            }
            Event::MouseDown { button, .. } => {
                if let Some(action) = self.keys_mapping.get(&Key::Mouse(button)).copied() {
                    Some((action, true))
                } else {
                    None
                }
            }
            Event::MouseUp { button, .. } => {
                if let Some(action) = self.keys_mapping.get(&Key::Mouse(button)).copied() {
                    Some((action, false))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}
