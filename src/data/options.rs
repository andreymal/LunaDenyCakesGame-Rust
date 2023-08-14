use crate::{action::Action, data::texts::get_text};
use anyhow::Result;
use cake_engine::{
    input::{Key, MouseButton, ScanCode},
    vec::Vec2,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf, sync::Mutex};

pub static OPTIONS: Lazy<Mutex<Options>> = Lazy::new(|| Mutex::new(Options::default()));

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum Difficulty {
    Easy,
    Medi,
    Hard,
}

impl Difficulty {
    pub fn code(&self) -> &'static str {
        match self {
            Difficulty::Easy => "easy",
            Difficulty::Medi => "medi",
            Difficulty::Hard => "hard",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Options {
    window_size: Vec2,
    soundon: bool,
    musicon: bool,
    fullscreen: bool,
    vsync: bool,
    fps_limit: f32,
    show_fps_counter: bool,
    touchui: bool,
    currentlang: String,
    #[serde(skip)]
    languages: Vec<String>,
    difficulty: Difficulty,
    apply_after_select: bool,
    keys: HashMap<Action, Key>,
}

impl Options {
    pub fn get_system_language() -> String {
        match sys_locale::get_locale() {
            Some(mut l) => {
                if let Some(idx) = l.find('-') {
                    l.truncate(idx);
                }
                l
            }
            None => "en".to_string(),
        }
    }

    pub fn get_default_keys() -> HashMap<Action, Key> {
        let mut keys = HashMap::new();

        keys.insert(Action::Switch, Key::Mouse(MouseButton::Right));
        keys.insert(Action::Apply, Key::Mouse(MouseButton::Left));
        keys.insert(Action::SelTeleport, Key::Keyboard(ScanCode::Num1));
        keys.insert(Action::SelLaser, Key::Keyboard(ScanCode::Num2));
        keys.insert(Action::SelChicken, Key::Keyboard(ScanCode::Num3));
        keys.insert(Action::SelShield, Key::Keyboard(ScanCode::Num4));
        keys.insert(Action::Left, Key::Keyboard(ScanCode::A));
        keys.insert(Action::Right, Key::Keyboard(ScanCode::D));

        keys
    }
}

impl Default for Options {
    fn default() -> Self {
        Options {
            window_size: Vec2::new(1024.0, 768.0),
            soundon: true,
            musicon: !cfg!(feature = "macroquad"), // macroquad очень долго грузит музыку, поэтому отключаем по умолчанию
            fullscreen: cfg!(target_os = "android"),
            vsync: true,
            fps_limit: 0.0,
            show_fps_counter: false,
            touchui: cfg!(target_os = "android"),
            currentlang: "".to_string(),
            languages: Vec::new(),
            difficulty: Difficulty::Easy,
            apply_after_select: false,
            keys: Options::get_default_keys(),
        }
    }
}

impl Options {
    fn check_lang(&mut self) {
        // Если текущий язык неизвестный, ставим системный язык
        if !self.languages.iter().any(|l| l == &self.currentlang) {
            self.currentlang = Options::get_system_language();
        }

        // Если системный язык тоже неизвестный, то ставим первый известный язык
        if !self.languages.iter().any(|l| l == &self.currentlang) {
            self.currentlang = match self.languages.first() {
                Some(l) => l.clone(),
                None => "".to_string(),
            };
        }
    }

    pub fn path() -> Option<PathBuf> {
        Some(crate::data::data_dir()?.join("options.json"))
    }

    pub fn load(&mut self) -> Result<()> {
        let path = match Options::path() {
            Some(p) => p,
            None => return Err(anyhow::anyhow!("No data directory")),
        };

        // serde затрёт список доступных языков значением по умолчанию, поэтому бэкапим
        let languages = self.languages.clone();

        let serialized = std::fs::read_to_string(path)?;
        *self = serde_json::from_str(&serialized)?;
        self.languages = languages; // возвращаем из бэкапа
        self.check_lang();
        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        let path = match Options::path() {
            Some(p) => p,
            None => return Err(anyhow::anyhow!("No data directory")),
        };

        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }

        let serialized = serde_json::to_string_pretty(&self)?;
        std::fs::write(path, &serialized)?;
        Ok(())
    }

    pub fn get_window_size(&self) -> Vec2 {
        self.window_size
    }

    pub fn set_window_size(&mut self, size: Vec2) {
        self.window_size = size;
    }

    pub fn get_soundon(&self) -> bool {
        self.soundon
    }

    pub fn set_soundon(&mut self, soundon: bool) {
        self.soundon = soundon;
    }

    pub fn invert_soundon(&mut self) -> bool {
        self.soundon = !self.soundon;
        self.soundon
    }

    pub fn get_musicon(&self) -> bool {
        self.musicon
    }

    pub fn set_musicon(&mut self, musicon: bool) {
        self.musicon = musicon;
    }

    pub fn invert_musicon(&mut self) -> bool {
        self.musicon = !self.musicon;
        self.musicon
    }

    pub fn get_fullscreen(&self) -> bool {
        self.fullscreen
    }

    pub fn set_fullscreen(&mut self, fullscreen: bool) {
        self.fullscreen = fullscreen;
    }

    pub fn invert_fullscreen(&mut self) -> bool {
        self.fullscreen = !self.fullscreen;
        self.fullscreen
    }

    pub fn get_vsync(&self) -> bool {
        self.vsync
    }

    pub fn set_vsync(&mut self, vsync: bool) {
        self.vsync = vsync;
    }

    pub fn invert_vsync(&mut self) -> bool {
        self.vsync = !self.vsync;
        self.vsync
    }

    pub fn get_fps_limit(&self) -> f32 {
        self.fps_limit
    }

    pub fn set_fps_limit(&mut self, value: f32) {
        if value.is_finite() && !value.is_nan() && value > 0.0 {
            self.fps_limit = value;
        } else {
            self.fps_limit = 0.0;
        }
    }

    pub fn switch_fps_limit(&mut self) -> f32 {
        for next_value in [
            30.0, 45.0, 60.0, 75.0, 90.0, 120.0, 144.0, 180.0, 240.0, 300.0, 500.0, 750.0, 1000.0,
        ] {
            if self.fps_limit < next_value {
                self.fps_limit = next_value;
                return self.fps_limit;
            }
        }
        self.fps_limit = 0.0;
        self.fps_limit
    }

    pub fn get_show_fps_counter(&self) -> bool {
        self.show_fps_counter
    }

    pub fn set_show_fps_counter(&mut self, value: bool) {
        self.show_fps_counter = value;
    }

    pub fn invert_show_fps_counter(&mut self) -> bool {
        self.show_fps_counter = !self.show_fps_counter;
        self.show_fps_counter
    }

    pub fn get_touchui(&self) -> bool {
        self.touchui
    }

    pub fn set_touchui(&mut self, touchui: bool) {
        self.touchui = touchui;
    }

    pub fn invert_touchui(&mut self) -> bool {
        self.touchui = !self.touchui;
        self.touchui
    }

    pub fn get_available_languages(&self) -> &[String] {
        &self.languages
    }

    pub fn set_available_languages(&mut self, languages: &[String]) {
        self.languages.clear();
        for l in languages {
            self.languages.push(l.clone());
        }
    }

    pub fn set_current_language(&mut self, lang: String) -> bool {
        if self.languages.contains(&lang) {
            self.currentlang = lang;
            true
        } else {
            false
        }
    }

    pub fn switch_current_language(&mut self) -> &str {
        if self.languages.is_empty() {
            return &self.currentlang;
        }
        let mut idx = self
            .languages
            .iter()
            .position(|l| l == &self.currentlang)
            .unwrap_or(self.languages.len() - 1);
        idx = (idx + 1) % self.languages.len();
        self.currentlang = self.languages[idx].clone();
        &self.currentlang
    }

    pub fn get_current_language(&self) -> &str {
        &self.currentlang
    }

    pub fn get_difficulty(&self) -> Difficulty {
        self.difficulty
    }

    pub fn set_difficulty(&mut self, difficulty: Difficulty) {
        self.difficulty = difficulty;
    }

    pub fn switch_difficulty(&mut self) -> Difficulty {
        self.difficulty = match self.difficulty {
            Difficulty::Easy => Difficulty::Medi,
            Difficulty::Medi => Difficulty::Hard,
            Difficulty::Hard => Difficulty::Easy,
        };
        self.difficulty
    }

    pub fn get_keys(&self) -> &HashMap<Action, Key> {
        &self.keys
    }

    pub fn get_keys_mut(&mut self) -> &HashMap<Action, Key> {
        &mut self.keys
    }

    pub fn set_keys(&mut self, keys: &HashMap<Action, Key>) {
        self.keys.clear();
        self.keys.extend(keys);
    }

    pub fn reset_controls_to_default(&mut self) {
        self.keys = Options::get_default_keys();
        self.apply_after_select = false;
    }

    pub fn get_apply_after_select(&self) -> bool {
        self.apply_after_select
    }

    pub fn set_apply_after_select(&mut self, value: bool) {
        self.apply_after_select = value;
    }

    pub fn switch_apply_after_select(&mut self) -> bool {
        self.apply_after_select = !self.apply_after_select;
        self.apply_after_select
    }
}

pub fn key_to_human_string(key: Key) -> String {
    match key {
        Key::Keyboard(scancode) => serde_plain::to_string(&scancode).unwrap(),
        Key::Mouse(button) => match button {
            MouseButton::Left => get_text("text_lcm"),
            MouseButton::Right => get_text("text_rcm"),
            MouseButton::Middle => get_text("text_mcm"),
            _ => serde_plain::to_string(&button).unwrap(),
        },
    }
}
