use crate::{
    balance::Balance,
    data::{options::Difficulty, texts::get_text},
};
use cake_engine::vec::Vec2;
use rand::{seq::SliceRandom, Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

pub const CAKE_Y: f32 = 30.0;
pub const BLOCKW: f32 = 84.0;
pub const BLOCKH: f32 = 24.0;

pub const ZONEW: f32 = BLOCKW;
pub const ZONEH: f32 = 110.0;
pub const ZONEH1: f32 = ZONEH - BLOCKH;
pub const CAKEW: f32 = 48.0;
pub const PONYW: f32 = 30.0;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Zone {
    pub y: f32,
    pub left: f32,
    pub right: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Direction {
    No,
    Left,
    Right,
}

impl Direction {
    pub fn same_way(&self, x_from: f32, x_to: f32) -> bool {
        match self {
            Direction::Left => x_to <= x_from,
            Direction::Right => x_to >= x_from,
            Direction::No => false,
        }
    }

    pub fn sig_f(&self) -> f32 {
        match self {
            Direction::Left => -1.0,
            Direction::Right => 1.0,
            Direction::No => 0.0,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Chicken {
    pub zoneidx: usize,
    pub x: f32,
    pub vx: f32,
    pub removed: bool,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct FallingChicken {
    pub pos: Vec2,
    pub vel: Vec2,
    pub rotation: f32,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Cake {
    pub zoneidx: usize,
    pub x: f32,
    pub spriteidx: usize,
    pub hp: f32,
    pub shieldleft: f32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GameState {
    Normal,
    Win(String),
    Fail(String),
}

#[derive(Clone, Debug)]
pub struct Game {
    seed: u64,
    rng: ChaCha8Rng,
    balance: Balance,
    zones: Vec<Zone>,
    chickens: Vec<Chicken>,
    falling_chickens: Vec<FallingChicken>,
    cakes: Vec<Cake>,
    celestiax: f32,
    celestiazoneidx: usize,
    celestiadir: Direction,
    lunax: f32,
    lunazoneidx: usize,
    lunadir: Direction,
    celestiahp: f32,
    state: GameState,
    mana: f32,
    is_celestia_eating: bool,
    wintimer: f32,
    laserdir: Direction,
}

impl Game {
    pub fn new(difficulty: Difficulty) -> Game {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut rng = ChaCha8Rng::seed_from_u64(seed);

        let balance = Balance::new(difficulty);

        #[rustfmt::skip]
        let zones = vec![
            Zone { y: 90.0, left: 50.0, right: 50.0 + ZONEW * 11.0 },
            Zone { y: 90.0 + 1.0 * ZONEH, left: 50.0, right: 50.0 + ZONEW * 11.0 },
            Zone { y: 90.0 + 2.0 * ZONEH, left: 50.0, right: 50.0 + ZONEW * 11.0 },
            Zone { y: 90.0 + 3.0 * ZONEH, left: 50.0, right: 50.0 + ZONEW * 11.0 },
            Zone { y: 90.0 + 4.0 * ZONEH, left: 50.0, right: 50.0 + ZONEW * 11.0 },
            Zone { y: 90.0 + 5.0 * ZONEH, left: 50.0, right: 50.0 + ZONEW * 11.0 },
            Zone { y: 90.0 + 6.0 * ZONEH, left: 50.0, right: 50.0 + ZONEW * 11.0 },
        ];

        let mut cakes = Vec::new();
        for i in 0..zones.len() {
            for pos in [200.0, 500.0, 800.0] {
                cakes.push(Cake {
                    x: pos + rng.gen_range(-100..100) as f32,
                    zoneidx: i,
                    spriteidx: rng.gen_range(0..3),
                    hp: 1.0,
                    shieldleft: 0.0,
                });
            }
        }

        let celestiazoneidx = 0;
        let celestiax = (zones[celestiazoneidx].left + zones[celestiazoneidx].right) / 2.0;
        let celestiahp = balance.celestia_start_hp;

        let lunazoneidx = 6;
        let lunax = (zones[lunazoneidx].left + zones[lunazoneidx].right) / 2.0;
        let mana = balance.max_mana;

        Game {
            seed,
            rng,
            balance,
            zones,
            chickens: Vec::new(),
            falling_chickens: Vec::new(),
            cakes,
            celestiax,
            celestiazoneidx,
            celestiadir: Direction::No,
            lunax,
            lunazoneidx,
            lunadir: Direction::Right,
            celestiahp,
            state: GameState::Normal,
            mana,
            is_celestia_eating: false,
            wintimer: 0.0,
            laserdir: Direction::No,
        }
    }

    pub fn get_seed(&self) -> u64 {
        self.seed
    }

    pub fn zones(&self) -> &[Zone] {
        &self.zones
    }

    pub fn chickens(&self) -> &[Chicken] {
        &self.chickens
    }

    pub fn falling_chickens(&self) -> &[FallingChicken] {
        &self.falling_chickens
    }

    pub fn cakes(&self) -> &[Cake] {
        &self.cakes
    }

    pub fn get_balance(&self) -> &Balance {
        &self.balance
    }

    pub fn get_mana(&self) -> f32 {
        self.mana
    }

    pub fn dec_mana(&mut self, delta: f32) {
        if self.mana > delta {
            self.mana -= delta;
        }
    }

    pub fn get_celestia_pos(&self) -> Vec2 {
        Vec2::new(self.celestiax, self.zones[self.celestiazoneidx].y)
    }

    pub fn get_celestia_dir(&self) -> Direction {
        self.celestiadir
    }

    pub fn is_celestia_eating(&self) -> bool {
        self.is_celestia_eating
    }

    pub fn get_celestia_zone_idx(&self) -> usize {
        self.celestiazoneidx
    }

    pub fn get_celestia_hp_percent(&self) -> f32 {
        self.celestiahp * 100.0 / self.balance.celestia_start_hp
    }

    pub fn get_luna_pos(&self) -> Vec2 {
        Vec2::new(self.lunax, self.zones[self.lunazoneidx].y)
    }

    pub fn get_luna_dir(&self) -> Direction {
        self.lunadir
    }

    pub fn get_luna_zone_idx(&self) -> usize {
        self.lunazoneidx
    }

    pub fn send_luna_left(&mut self, dt: f32) -> bool {
        if self.laserdir != Direction::No {
            return false;
        }

        let newlunax = self.lunax - self.balance.luna_vel * dt;
        if newlunax >= self.zones[self.lunazoneidx].left + PONYW / 2.0 {
            self.lunax = newlunax;
            self.lunadir = Direction::Left;
            true
        } else {
            false
        }
    }

    pub fn send_luna_right(&mut self, dt: f32) -> bool {
        if self.laserdir != Direction::No {
            return false;
        }

        let newlunax = self.lunax + self.balance.luna_vel * dt;
        if newlunax <= self.zones[self.lunazoneidx].right - PONYW / 2.0 {
            self.lunax = newlunax;
            self.lunadir = Direction::Right;
            true
        } else {
            false
        }
    }

    pub fn get_zone_id_by_xy(&self, mxy: Vec2) -> Option<usize> {
        for (i, zone) in self.zones.iter().enumerate() {
            if zone.left < mxy.x && mxy.x < zone.right && zone.y > mxy.y && mxy.y > zone.y - ZONEH1
            {
                return Some(i);
            }
        }
        None
    }

    pub fn get_cake_id_at(&self, mxy: Vec2) -> Option<usize> {
        for (i, cake) in self.cakes.iter().enumerate() {
            let zone = self.zones[cake.zoneidx];
            if cake.x - CAKEW / 2.0 < mxy.x
                && cake.x + CAKEW / 2.0 > mxy.x
                && zone.y - CAKEW / 2.0 - CAKE_Y < mxy.y
                && zone.y + CAKEW / 2.0 - CAKE_Y > mxy.y
            {
                return Some(i);
            }
        }
        None
    }

    pub fn jump_luna_to_xy(&mut self, mxy: Vec2) -> bool {
        let idx = match self.get_zone_id_by_xy(mxy) {
            Some(idx) => idx,
            None => return false,
        };

        let zone = self.zones[idx];
        self.lunazoneidx = idx;
        self.lunax = mxy
            .x
            .clamp(zone.left + PONYW / 2.0, zone.right - PONYW / 2.0);
        true
    }

    pub fn jump_celestia_to_best_zone(&mut self) {
        let mut zones_for_jump = Vec::new();
        for cake in &self.cakes {
            if cake.shieldleft <= 0.0 && !zones_for_jump.contains(&cake.zoneidx) {
                zones_for_jump.push(cake.zoneidx);
            }
        }

        for chicken in &self.chickens {
            zones_for_jump.retain(|&x| x != chicken.zoneidx);
        }

        // Если подходящие зоны с кексами и без куриц не найдены, но на зоне Селестии находится курица
        if zones_for_jump.is_empty()
            && self
                .chickens
                .iter()
                .any(|c| c.zoneidx == self.celestiazoneidx)
        {
            // Добавляем все зоны, свободные от куриц
            for i in 0..self.zones.len() {
                if !self.chickens.iter().any(|c| c.zoneidx == i) {
                    zones_for_jump.push(i);
                }
            }
        }

        if !zones_for_jump.is_empty() {
            self.celestiazoneidx = *zones_for_jump.choose(&mut self.rng).unwrap();
        }
    }

    pub fn add_chicken(&mut self, mxy: Vec2) -> bool {
        let idx = match self.get_zone_id_by_xy(mxy) {
            Some(idx) => idx,
            None => return false,
        };

        let zone = &self.zones[idx];
        let chickenx = mxy
            .x
            .clamp(zone.left + PONYW / 2.0, zone.right - PONYW / 2.0);

        let vsig = if self.celestiazoneidx == idx {
            (self.celestiax - chickenx).signum()
        } else if self.rng.gen_bool(0.5) {
            1.0
        } else {
            -1.0
        };

        self.chickens.push(Chicken {
            x: chickenx,
            zoneidx: idx,
            vx: self.balance.chicken_vel * vsig,
            removed: false,
        });

        true
    }

    pub fn get_laser_dir(&self) -> Direction {
        self.laserdir
    }

    pub fn start_laser(&mut self, mxy: Vec2) {
        self.laserdir = if mxy.x > self.lunax {
            Direction::Right
        } else {
            Direction::Left
        };
        self.lunadir = self.laserdir;
    }

    pub fn finish_laser(&mut self) {
        self.laserdir = Direction::No;
    }

    pub fn set_shield_to_cake_by_xy(&mut self, mxy: Vec2) -> bool {
        let idx = match self.get_cake_id_at(mxy) {
            Some(idx) => idx,
            None => return false,
        };
        self.cakes[idx].shieldleft = self.balance.shield_time;
        true
    }

    pub fn get_state(&self) -> &GameState {
        &self.state
    }

    pub fn update(&mut self, dt: f32) {
        for cake in self.cakes.iter_mut() {
            if cake.shieldleft > 0.0 {
                cake.shieldleft -= dt;
            }
        }

        for falling_chicken in self.falling_chickens.iter_mut() {
            falling_chicken.pos.x += falling_chicken.vel.x * dt;
            falling_chicken.pos.y += falling_chicken.vel.y * dt;
            falling_chicken.rotation += 2.0 * falling_chicken.vel.x * dt;
        }

        let bottom = self.zones.last().unwrap().y + BLOCKH;
        self.falling_chickens.retain(|c| c.pos.y < bottom + 100.0);

        for chicken in self.chickens.iter_mut() {
            chicken.x += chicken.vx * dt;
            let zone = self.zones[chicken.zoneidx];
            if chicken.vx < 0.0 && chicken.x < zone.left
                || chicken.vx > 0.0 && chicken.x > zone.right
            {
                self.falling_chickens.push(FallingChicken {
                    pos: Vec2::new(chicken.x, zone.y),
                    vel: Vec2::new(chicken.vx, 200.0),
                    rotation: 0.0,
                });
                chicken.removed = true;
            }
        }

        if self.laserdir != Direction::No {
            for cake in self.cakes.iter_mut() {
                if cake.zoneidx == self.lunazoneidx
                    && cake.shieldleft <= 0.0
                    && self.laserdir.same_way(self.lunax, cake.x)
                {
                    cake.hp -= self.balance.laser_power_in_sec * dt;
                }
            }

            for chicken in self.chickens.iter_mut() {
                if chicken.zoneidx == self.lunazoneidx
                    && self.laserdir.same_way(self.lunax, chicken.x)
                {
                    chicken.removed = true;
                }
            }

            if self.celestiazoneidx == self.lunazoneidx
                && self.laserdir.same_way(self.lunax, self.celestiax)
            {
                self.state = GameState::Fail(get_text("msg_laserfail"));
            }

            self.mana -= self.balance.laser_cost_in_sec * dt;
            if self.mana <= 0.0 {
                self.mana = 0.0;
                self.laserdir = Direction::No;
            }
        } else {
            self.mana += self.balance.regen_mana_in_sec * dt;
            if self.mana >= self.balance.max_mana {
                self.mana = self.balance.max_mana;
            }
        }

        self.chickens.retain(|c| !c.removed);

        // Расчет Селестии
        if self
            .chickens
            .iter()
            .any(|c| c.zoneidx == self.celestiazoneidx)
        {
            self.jump_celestia_to_best_zone();
        }

        let mut eaten_cake_id = None;
        for (id, cake) in self.cakes.iter().enumerate() {
            if cake.zoneidx == self.celestiazoneidx
                && cake.shieldleft <= 0.0
                && (self.celestiax - cake.x).abs() < (PONYW / 2.0 + CAKEW / 2.0)
            {
                eaten_cake_id = Some(id);
                break;
            }
        }
        self.is_celestia_eating = eaten_cake_id.is_some();

        if let Some(eaten_cake_id) = eaten_cake_id {
            let cake = &mut self.cakes[eaten_cake_id];
            self.celestiadir = if cake.x - self.celestiax > 0.0 {
                Direction::Right
            } else {
                Direction::Left
            };
            let mut dh = self.balance.eat_in_sec * dt;
            if dh > cake.hp {
                dh = cake.hp;
            }
            cake.hp -= dh;
            self.celestiahp -= dh;
        } else {
            let mut near_cake_id = None;
            let mut dist = f32::INFINITY;
            for (i, cake) in self.cakes.iter().enumerate() {
                if cake.zoneidx == self.celestiazoneidx && cake.shieldleft <= 0.0 {
                    let d = (cake.x - self.celestiax).abs();
                    if d < dist {
                        near_cake_id = Some(i);
                        dist = d;
                    }
                }
            }
            if let Some(near_cake_id) = near_cake_id {
                let cake = &self.cakes[near_cake_id];
                self.celestiadir = if (cake.x - self.celestiax) > 0.0 {
                    Direction::Right
                } else {
                    Direction::Left
                };
                self.celestiax += self.balance.celestia_vel * self.celestiadir.sig_f() * dt;
            } else {
                // Не найден кексик на уровне
                self.celestiadir = Direction::No;
                self.jump_celestia_to_best_zone();
            }
        }

        self.cakes.retain(|c| c.hp > 0.0);

        if self.state == GameState::Normal {
            if self.cakes.is_empty() {
                self.wintimer += dt;
                if self.wintimer >= 2.0 {
                    self.state = GameState::Win(get_text("msg_cakeover"));
                }
            }

            if self.get_celestia_hp_percent().floor() == 0.0 {
                self.state = GameState::Fail(get_text("msg_celestiafail"));
            }
        }
    }
}
