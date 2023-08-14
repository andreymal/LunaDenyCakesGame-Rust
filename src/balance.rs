use crate::data::options::Difficulty;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Balance {
    pub luna_vel: f32,
    pub celestia_vel: f32,
    pub shield_time: f32,
    pub chicken_vel: f32,
    pub laser_power_in_sec: f32,
    pub laser_cost_in_sec: f32,
    pub shield_cost: f32,
    pub jump_cost: f32,
    pub chicken_cost: f32,
    pub max_mana: f32,
    pub regen_mana_in_sec: f32,
    pub eat_in_sec: f32,
    pub celestia_start_hp: f32,
}

impl Balance {
    pub fn new(difficulty: Difficulty) -> Balance {
        let mut balance = Balance::default();

        if difficulty == Difficulty::Easy {
            balance.eat_in_sec = 0.1;
            balance.regen_mana_in_sec = 10.0;
            balance.jump_cost = 30.0;
            balance.chicken_cost = 15.0;
            balance.laser_cost_in_sec = 25.0;
        } else if difficulty == Difficulty::Medi {
            balance.jump_cost = 30.0;
            balance.chicken_cost = 15.0;
            balance.laser_cost_in_sec = 25.0;
        }

        balance
    }
}

impl Default for Balance {
    fn default() -> Self {
        Balance {
            luna_vel: 100.0,
            celestia_vel: 70.0,
            shield_time: 10.0,
            chicken_vel: 50.0,
            laser_power_in_sec: 1.0,
            laser_cost_in_sec: 30.0,
            shield_cost: 10.0,
            jump_cost: 40.0,
            chicken_cost: 20.0,
            max_mana: 200.0,
            regen_mana_in_sec: 5.0,
            eat_in_sec: 0.2,
            celestia_start_hp: 5.0,
        }
    }
}
