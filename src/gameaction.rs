use crate::game::Game;
use cake_engine::vec::Vec2;

pub trait GameAction {
    fn code(&self) -> &'static str;
    fn is_allowed_somewhere(&self, game: &Game) -> bool {
        game.get_mana() >= self.get_cost(game)
    }
    fn is_allowed_at(&self, game: &Game, mxy: Vec2) -> bool;
    fn get_cost(&self, game: &Game) -> f32;
    fn apply(&mut self, game: &mut Game, mxy: Vec2) -> bool;
    #[allow(unused_variables)]
    fn finish(&mut self, game: &mut Game) {
        // No action default
    }
}

pub struct GAJump;

impl GameAction for GAJump {
    fn code(&self) -> &'static str {
        "jump"
    }

    fn is_allowed_at(&self, game: &Game, mxy: Vec2) -> bool {
        game.get_zone_id_by_xy(mxy).is_some() && self.is_allowed_somewhere(game)
    }

    fn get_cost(&self, game: &Game) -> f32 {
        game.get_balance().jump_cost
    }

    fn apply(&mut self, game: &mut Game, mxy: Vec2) -> bool {
        if !self.is_allowed_at(game, mxy) {
            return false;
        }
        game.dec_mana(self.get_cost(game));
        game.jump_luna_to_xy(mxy)
    }
}

pub struct GAChicken;

impl GameAction for GAChicken {
    fn code(&self) -> &'static str {
        "chicken"
    }

    fn is_allowed_at(&self, game: &Game, mxy: Vec2) -> bool {
        game.get_zone_id_by_xy(mxy).is_some() && self.is_allowed_somewhere(game)
    }

    fn get_cost(&self, game: &Game) -> f32 {
        game.get_balance().chicken_cost
    }

    fn apply(&mut self, game: &mut Game, mxy: Vec2) -> bool {
        if !self.is_allowed_at(game, mxy) {
            return false;
        }
        game.dec_mana(self.get_cost(game));
        game.add_chicken(mxy)
    }
}

pub struct GAShield;

impl GameAction for GAShield {
    fn code(&self) -> &'static str {
        "shield"
    }

    fn is_allowed_at(&self, game: &Game, mxy: Vec2) -> bool {
        game.get_cake_id_at(mxy).is_some() && self.is_allowed_somewhere(game)
    }

    fn get_cost(&self, game: &Game) -> f32 {
        game.get_balance().shield_cost
    }

    fn apply(&mut self, game: &mut Game, mxy: Vec2) -> bool {
        if !self.is_allowed_at(game, mxy) {
            return false;
        }
        game.dec_mana(self.get_cost(game));
        game.set_shield_to_cake_by_xy(mxy)
    }
}

pub struct GALaser;

impl GameAction for GALaser {
    fn code(&self) -> &'static str {
        "laser"
    }

    fn is_allowed_at(&self, game: &Game, mxy: Vec2) -> bool {
        game.get_zone_id_by_xy(mxy) == Some(game.get_luna_zone_idx())
            && self.is_allowed_somewhere(game)
    }

    fn get_cost(&self, game: &Game) -> f32 {
        game.get_balance().laser_cost_in_sec
    }

    fn apply(&mut self, game: &mut Game, mxy: Vec2) -> bool {
        if !self.is_allowed_at(game, mxy) {
            return false;
        }
        game.start_laser(mxy);
        true
    }

    fn finish(&mut self, game: &mut Game) {
        game.finish_laser();
    }
}
