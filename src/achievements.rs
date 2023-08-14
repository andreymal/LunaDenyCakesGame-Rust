use crate::{
    data::options::{Difficulty, OPTIONS},
    game::{Game, GameState},
};
use anyhow::Result;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

#[derive(Clone)]
pub struct AchievementStore {
    completed: HashSet<String>,
    detectors: Vec<(Box<dyn Achievement>, bool)>,
    storefile: Option<PathBuf>,
}

impl AchievementStore {
    pub fn new() -> AchievementStore {
        AchievementStore {
            completed: HashSet::new(),
            detectors: vec![
                (Box::new(AchievementWinEasy), false),
                (Box::new(AchievementWinMedi), false),
                (Box::new(AchievementWinHard), false),
                (Box::new(AchievementWinMediChicken::new()), false),
                (Box::new(AchievementWinMediShield::new()), false),
                (Box::new(AchievementWinMedi50), false),
                (Box::new(AchievementWinEasy75), false),
            ],
            storefile: None,
        }
    }

    pub fn reset_achievements(&mut self) -> Result<()> {
        self.completed.clear();
        self.save()?;
        Ok(())
    }

    pub fn reset_detector(&mut self) {
        for (item, ingame_completed) in self.detectors.iter_mut() {
            *ingame_completed = false;
            item.reset();
        }
    }

    pub fn update(&mut self, game: &Game) -> Result<()> {
        let mut changed = false;
        for (item, ingame_completed) in self.detectors.iter_mut() {
            if *ingame_completed {
                continue;
            }
            if item.update(game) {
                *ingame_completed = true;
                if self.completed.insert(item.code().to_string()) {
                    changed = true;
                }
            }
        }
        if changed {
            self.save()?;
        }
        Ok(())
    }

    pub fn count(&self) -> usize {
        self.detectors.len()
    }

    pub fn completed_count(&self) -> usize {
        self.completed.len()
    }

    pub fn list(&self) -> &[(Box<dyn Achievement>, bool)] {
        &self.detectors
    }

    pub fn is_completed(&self, code: &str) -> bool {
        self.completed.contains(code)
    }

    pub fn load(&mut self, path: &Path) -> Result<()> {
        self.storefile = Some(path.to_path_buf());
        let serialized = std::fs::read_to_string(path)?;
        self.completed = serde_json::from_str(&serialized)?;
        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        if let Some(path) = self.storefile.as_ref() {
            let serialized = serde_json::to_string(&self.completed)?;
            std::fs::write(path, serialized)?;
        }
        Ok(())
    }
}

pub trait Achievement: dyn_clone::DynClone {
    fn code(&self) -> &'static str;
    fn update(&mut self, game: &Game) -> bool;
    fn reset(&mut self) {}
}

dyn_clone::clone_trait_object!(Achievement);

#[derive(Clone)]
pub struct AchievementWinEasy;

impl Achievement for AchievementWinEasy {
    fn code(&self) -> &'static str {
        "win_easy"
    }

    fn update(&mut self, game: &Game) -> bool {
        if let GameState::Win(_) = game.get_state() {
            let options = OPTIONS.lock().unwrap();
            options.get_difficulty() == Difficulty::Easy
        } else {
            false
        }
    }
}

#[derive(Clone)]
pub struct AchievementWinMedi;

impl Achievement for AchievementWinMedi {
    fn code(&self) -> &'static str {
        "win_medi"
    }

    fn update(&mut self, game: &Game) -> bool {
        if let GameState::Win(_) = game.get_state() {
            let options = OPTIONS.lock().unwrap();
            options.get_difficulty() == Difficulty::Medi
        } else {
            false
        }
    }
}

#[derive(Clone)]
pub struct AchievementWinHard;

impl Achievement for AchievementWinHard {
    fn code(&self) -> &'static str {
        "win_hard"
    }

    fn update(&mut self, game: &Game) -> bool {
        if let GameState::Win(_) = game.get_state() {
            let options = OPTIONS.lock().unwrap();
            options.get_difficulty() == Difficulty::Hard
        } else {
            false
        }
    }
}

#[derive(Clone)]
pub struct AchievementWinMediChicken {
    chicken_used: bool,
}

impl AchievementWinMediChicken {
    pub fn new() -> AchievementWinMediChicken {
        AchievementWinMediChicken {
            chicken_used: false,
        }
    }
}

impl Achievement for AchievementWinMediChicken {
    fn code(&self) -> &'static str {
        "win_medi_chicken"
    }

    fn update(&mut self, game: &Game) -> bool {
        if self.chicken_used {
            return false;
        }

        if game.chickens().len() > 0 {
            self.chicken_used = true;
            return false;
        }

        if let GameState::Win(_) = game.get_state() {
            let options = OPTIONS.lock().unwrap();
            options.get_difficulty() == Difficulty::Medi
        } else {
            false
        }
    }

    fn reset(&mut self) {
        self.chicken_used = false;
    }
}

#[derive(Clone)]
pub struct AchievementWinMediShield {
    shield_used: bool,
}

impl AchievementWinMediShield {
    pub fn new() -> AchievementWinMediShield {
        AchievementWinMediShield { shield_used: false }
    }
}

impl Achievement for AchievementWinMediShield {
    fn code(&self) -> &'static str {
        "win_medi_shield"
    }

    fn update(&mut self, game: &Game) -> bool {
        if self.shield_used {
            return false;
        }

        for cake in game.cakes() {
            if cake.shieldleft > 0.0 {
                self.shield_used = true;
                return false;
            }
        }

        if let GameState::Win(_) = game.get_state() {
            let options = OPTIONS.lock().unwrap();
            options.get_difficulty() == Difficulty::Medi
        } else {
            false
        }
    }

    fn reset(&mut self) {
        self.shield_used = false;
    }
}

#[derive(Clone)]
pub struct AchievementWinMedi50;

impl Achievement for AchievementWinMedi50 {
    fn code(&self) -> &'static str {
        "win_medi_50"
    }

    fn update(&mut self, game: &Game) -> bool {
        if let GameState::Win(_) = game.get_state() {
            if game.get_celestia_hp_percent() < 50.0 {
                return false;
            }
            let options = OPTIONS.lock().unwrap();
            options.get_difficulty() == Difficulty::Medi
        } else {
            false
        }
    }
}

#[derive(Clone)]
pub struct AchievementWinEasy75;

impl Achievement for AchievementWinEasy75 {
    fn code(&self) -> &'static str {
        "win_easy_75"
    }

    fn update(&mut self, game: &Game) -> bool {
        if let GameState::Win(_) = game.get_state() {
            if game.get_celestia_hp_percent() < 75.0 {
                return false;
            }
            let options = OPTIONS.lock().unwrap();
            options.get_difficulty() == Difficulty::Easy
        } else {
            false
        }
    }
}

#[derive(Clone)]
pub struct AchievementRewriteItInRust;

impl Achievement for AchievementRewriteItInRust {
    fn code(&self) -> &'static str {
        "rewrite_it_in_rust"
    }

    fn update(&mut self, _game: &Game) -> bool {
        true
    }
}
