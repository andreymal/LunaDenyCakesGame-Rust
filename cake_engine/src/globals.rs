// Хранилка глобальных штук, которые одинаковые в разных бэкендах. Собраны в одну структуру
// для удобства и уменьшения копипаста между бэкендами.

use crate::{
    audio::{Music, Sound},
    font::Font,
    texture::{Texture, TextureOptions, TextureSource},
    vec::Vec2,
};
use std::{collections::HashMap, path::Path, rc::Rc};

pub(crate) struct Globals {
    pub(crate) time: crate::gametime::GameTime,
    pub(crate) input: crate::input::Input,
    pub(crate) view: crate::view::View,
    pub(crate) last_texture_id: usize,
    pub(crate) textures: HashMap<usize, Rc<crate::texture::Texture>>,
    pub(crate) last_font_id: usize,
    pub(crate) fonts: HashMap<usize, Rc<crate::font::Font>>,
    pub(crate) last_music_id: usize,
    pub(crate) music: HashMap<usize, Rc<crate::audio::Music>>,
    pub(crate) last_sound_id: usize,
    pub(crate) sounds: HashMap<usize, Rc<crate::audio::Sound>>,
}

impl Globals {
    pub fn new(view_target_size: Vec2) -> Globals {
        Globals {
            time: crate::gametime::GameTime::new(),
            input: crate::input::Input::new(),
            view: crate::view::View::new(view_target_size),
            last_texture_id: 0,
            textures: HashMap::new(),
            last_font_id: 0,
            fonts: HashMap::new(),
            last_music_id: 0,
            music: HashMap::new(),
            last_sound_id: 0,
            sounds: HashMap::new(),
        }
    }

    pub(crate) fn add_texture(
        &mut self,
        source: TextureSource,
        width: u32,
        height: u32,
        options: TextureOptions,
    ) -> Rc<Texture> {
        self.last_texture_id += 1;
        let id = self.last_texture_id;

        let t = Rc::new(Texture {
            id,
            source,
            width,
            height,
            options,
        });
        self.textures.insert(id, t.clone());
        t
    }

    #[allow(dead_code)]
    pub(crate) fn get_lang_textures(&self) -> Vec<Rc<Texture>> {
        self.textures
            .values()
            .filter(|t| matches!(t.source, TextureSource::LangFile(_)))
            .map(|t| t.clone())
            .collect()
    }

    pub(crate) fn get_unused_texture_ids(&self) -> Vec<usize> {
        self.textures
            .iter()
            .filter(|(_, t)| Rc::strong_count(t) < 2)
            .map(|(t_id, _)| *t_id)
            .collect()
    }

    pub(crate) fn add_font(&mut self, path: &Path, size: u16) -> Rc<Font> {
        self.last_font_id += 1;
        let id = self.last_font_id;

        let f = Rc::new(Font {
            id,
            path: path.to_path_buf(),
            size,
        });
        self.fonts.insert(id, f.clone());
        f
    }

    pub(crate) fn get_unused_font_ids(&self) -> Vec<usize> {
        self.fonts
            .iter()
            .filter(|(_, f)| Rc::strong_count(f) < 2)
            .map(|(f_id, _)| *f_id)
            .collect()
    }

    pub(crate) fn add_music(&mut self, path: &Path) -> Rc<Music> {
        self.last_music_id += 1;
        let id = self.last_music_id;

        let m = Rc::new(Music {
            id,
            path: path.to_path_buf(),
        });

        self.music.insert(id, m.clone());
        m
    }

    pub(crate) fn get_unused_music_ids(&self) -> Vec<usize> {
        self.music
            .iter()
            .filter(|(_, m)| Rc::strong_count(m) < 2)
            .map(|(m_id, _)| *m_id)
            .collect()
    }

    pub(crate) fn add_sound(&mut self, path: &Path) -> Rc<Sound> {
        self.last_sound_id += 1;
        let id = self.last_sound_id;

        let s = Rc::new(Sound {
            id,
            path: path.to_path_buf(),
        });

        self.sounds.insert(id, s.clone());
        s
    }

    pub(crate) fn get_unused_sound_ids(&self) -> Vec<usize> {
        self.sounds
            .iter()
            .filter(|(_, s)| Rc::strong_count(s) < 2)
            .map(|(s_id, _)| *s_id)
            .collect()
    }
}
