use crate::{
    action::Action,
    common_data::CommonData,
    data::options::OPTIONS,
    game,
    game::{Direction, Game, GameState},
    gameaction::{GAChicken, GAJump, GALaser, GAShield, GameAction},
    scene::{gameover::SceneGameOver, menu::SceneMenu},
    touchui::TouchUi,
    utils::{spr, tex},
};
use anyhow::Result;
use cake_engine::{
    audio::Sound,
    color::Color,
    context::{Context, DrawTextureParams},
    input::{Actions, Event, ScanCode},
    label::Label,
    rect::Rect,
    scene::{Scene, SceneResult},
    sprite::Sprite,
    texture::Texture,
    vec::Vec2,
};
use std::{path::Path, rc::Rc};

const MANA_COLOR: Color = Color::new(35, 20, 250, 255);
const HP_COLOR: Color = Color::new(240, 240, 240, 255);
const INDICATOR_W: f32 = 48.0;
const INDICATOR_H: f32 = 8.0;
const LASER_Y: f32 = 80.0;

const CHICKEN_Y: f32 = 30.0;

const COLORSET: [Color; 4] = [
    Color::new(255, 0, 0, 255),
    Color::new(255, 128, 0, 255),
    Color::new(255, 255, 0, 255),
    Color::new(0, 255, 0, 255),
];

const ACT_TELEPORT: usize = 0;
const ACT_LASER: usize = 1;
const ACT_CHICKEN: usize = 2;
const ACT_SHIELD: usize = 3;

const GRAVITY: f32 = 800.0;

struct FallingCelestia {
    pos: Vec2,
    vel: Vec2,
    dir: Direction,
    eating: bool,
}

struct Sounds {
    snd_galop: Rc<Sound>,
    snd_galop2: Rc<Sound>,
    snd_laser: Rc<Sound>,
    snd_teleport: Rc<Sound>,
    snd_chicken: Rc<Sound>,
}

pub struct ScenePlay {
    common_data: CommonData,
    touchui: Option<TouchUi>,
    started: bool,
    block: Rc<Texture>,
    chicken: Rc<Texture>,
    cakes: [Rc<Texture>; 3],
    celestia_walk: Sprite,
    celestia_eat: Sprite,
    luna_walk: Sprite,
    luna_wait: Sprite,
    laser: Sprite,
    shield: Sprite,
    deny: Rc<Texture>,
    sounds: Option<Sounds>,
    game: Game,
    islunawalk: bool,
    iscelestiawalk: bool,
    islaseron: bool,
    oldcelestiazoneidx: usize,
    gameactions: [Box<dyn GameAction>; 4],
    action_textures: [Rc<Texture>; 4],
    current_action_id: usize,
    used_action_id: Option<usize>,
    mana_label: Label,
    hp_label: Label,
    input_actions: Actions<Action>,
    apply_after_select: bool,
    falling_celestia: Option<FallingCelestia>,
}

impl ScenePlay {
    pub fn new(common_data: CommonData, ctx: &mut dyn Context) -> Result<ScenePlay> {
        let options = OPTIONS.lock().unwrap();

        let game = Game::new(options.get_difficulty());
        let oldcelestiazoneidx = game.get_celestia_zone_idx();

        let mut mana_label = Label::new(common_data.font_main.clone(), MANA_COLOR);
        mana_label.set_origin(Vec2::new(0.5, 0.0));

        let mut hp_label = Label::new(common_data.font_main.clone(), HP_COLOR);
        hp_label.set_origin(Vec2::new(0.5, 0.0));

        let sounds = if options.get_soundon() {
            Some(Sounds {
                snd_galop: ctx.load_sound_file(Path::new("sounds/galop.ogg"))?,
                snd_galop2: ctx.load_sound_file(Path::new("sounds/galop.ogg"))?,
                snd_laser: ctx.load_sound_file(Path::new("sounds/laser.ogg"))?,
                snd_teleport: ctx.load_sound_file(Path::new("sounds/teleport.ogg"))?,
                snd_chicken: ctx.load_sound_file(Path::new("sounds/chicken.ogg"))?,
            })
        } else {
            None
        };

        let gameactions: [Box<dyn GameAction>; 4] = [
            Box::new(GAJump),
            Box::new(GALaser),
            Box::new(GAChicken),
            Box::new(GAShield),
        ];
        let action_textures = [
            tex!(
                ctx,
                &format!("images/action_{}.png", gameactions[ACT_TELEPORT].code())
            ),
            tex!(
                ctx,
                &format!("images/action_{}.png", gameactions[ACT_LASER].code())
            ),
            tex!(
                ctx,
                &format!("images/action_{}.png", gameactions[ACT_CHICKEN].code())
            ),
            tex!(
                ctx,
                &format!("images/action_{}.png", gameactions[ACT_SHIELD].code())
            ),
        ];

        let mut input_actions = Actions::new();
        input_actions.replace_all(options.get_keys());

        let deny = tex!(ctx, "images/deny.png");

        let touchui = if options.get_touchui() {
            Some(TouchUi::new(
                ctx,
                action_textures.clone(),
                [
                    Action::SelTeleport,
                    Action::SelLaser,
                    Action::SelChicken,
                    Action::SelShield,
                ],
                deny.clone(),
            )?)
        } else {
            None
        };

        let mut s = ScenePlay {
            common_data,
            touchui,
            started: false,
            block: tex!(ctx, "images/block.png"),
            chicken: tex!(ctx, "images/chicken.png"),
            cakes: [
                tex!(ctx, "images/cake1.png"),
                tex!(ctx, "images/cake2.png"),
                tex!(ctx, "images/cake3.png"),
            ],
            celestia_walk: spr!(ctx, "images/celestia_walk.png", 6.0, grid: (6, 1)),
            celestia_eat: spr!(ctx, "images/celestia_eat.png", 6.0, grid: (6, 1)),
            luna_walk: spr!(ctx, "images/luna_walk.png", 6.0, grid: (6, 1)),
            luna_wait: spr!(ctx, "images/luna_wait.png", 6.0, grid: (6, 1)),
            laser: spr!(ctx, "images/laser.png", 16.0, grid: (8, 1)),
            shield: spr!(ctx, "images/shield.png", 14.0, frame: (80, 80)),
            deny,
            sounds,
            game,
            islunawalk: false,
            iscelestiawalk: false,
            islaseron: false,
            oldcelestiazoneidx,
            gameactions,
            action_textures,
            current_action_id: 0,
            used_action_id: None,
            mana_label,
            hp_label,
            input_actions,
            apply_after_select: options.get_apply_after_select() && !options.get_touchui(),
            falling_celestia: None,
        };

        s.celestia_walk.set_origin(Vec2::new(0.5, 0.0));
        s.celestia_eat.set_origin(Vec2::new(0.5, 0.0));
        s.luna_walk.set_origin(Vec2::new(0.5, 0.0));
        s.luna_wait.set_origin(Vec2::new(0.5, 0.0));
        s.shield.set_origin(Vec2::new(0.5, 0.5));

        s.common_data.achievements.reset_detector();

        Ok(s)
    }

    fn draw_indicator(
        &self,
        ctx: &mut dyn Context,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        v: f32,
        colorset: &[Color],
    ) -> Result<()> {
        let v = v.clamp(0.0, 1.0);

        let dc = 1.0 / colorset.len() as f32;
        for i in (0..colorset.len()).rev() {
            if v > dc * i as f32 {
                ctx.set_fill_color(colorset[i]);
                break;
            }
        }

        let mut teksize = w * v;
        if teksize < 1.0 {
            teksize = 1.0;
        }

        ctx.fill_rect(Rect::new(x, y, teksize, h))?;
        Ok(())
    }

    fn apply_action(&mut self, ctx: &mut dyn Context, pressed: bool) -> Result<()> {
        let action = &mut self.gameactions[self.current_action_id];
        if pressed {
            let mxy = ctx.input().get_mouse_position();
            if action.apply(&mut self.game, mxy) {
                self.used_action_id = Some(self.current_action_id);
                self.handle_applied_action(ctx)?;
            }
        } else {
            if let Some(a_id) = self.used_action_id {
                self.gameactions[a_id].finish(&mut self.game);
            }
        }
        Ok(())
    }

    fn handle_action_event(
        &mut self,
        ctx: &mut dyn Context,
        action: Action,
        pressed: bool,
    ) -> Result<()> {
        if pressed && action == Action::Switch {
            self.current_action_id = (self.current_action_id + 1) % self.gameactions.len();
            return Ok(());
        }

        if self.apply_after_select {
            self.current_action_id = match action {
                Action::SelTeleport => ACT_TELEPORT,
                Action::SelLaser => ACT_LASER,
                Action::SelChicken => ACT_CHICKEN,
                Action::SelShield => ACT_SHIELD,
                _ => return Ok(()),
            };
            self.apply_action(ctx, pressed)?;
        } else if action == Action::SelTeleport && pressed {
            self.current_action_id = ACT_TELEPORT;
        } else if action == Action::SelLaser && pressed {
            self.current_action_id = ACT_LASER;
        } else if action == Action::SelChicken && pressed {
            self.current_action_id = ACT_CHICKEN;
        } else if action == Action::SelShield && pressed {
            self.current_action_id = ACT_SHIELD;
        } else if action == Action::Apply {
            self.apply_action(ctx, pressed)?;
        }

        Ok(())
    }

    pub fn handle_applied_action(&mut self, ctx: &mut dyn Context) -> Result<()> {
        if let Some(sounds) = self.sounds.as_ref() {
            if self.current_action_id == ACT_TELEPORT {
                ctx.play_sound(&sounds.snd_teleport, 1.0, false)?;
            } else if self.current_action_id == ACT_CHICKEN {
                ctx.play_sound(&sounds.snd_chicken, 1.0, false)?;
            }
        }
        Ok(())
    }

    pub fn process_game(
        &mut self,
        ctx: &mut dyn Context,
        dt: f32,
        events: &[Event],
    ) -> Result<GameState> {
        // Пропускаем первый кадр, чтобы скомпенсировать лаг, вызванный загрузкой
        // (особенно на macroquad)
        if !self.started {
            self.started = true;
            return Ok(GameState::Normal);
        }

        // Аномально большой dt может указывать на то, что игра была приостановлена,
        // поэтому ничего не делаем до следующего кадра
        if dt > 2.0 {
            return Ok(GameState::Normal);
        }

        // Просто большие скачки времени сглаживаем
        let dt = if dt < 0.2 { dt } else { 0.2 };

        let mut newlunawalk = false;

        // Сперва обрабатываем сенсорный интерфейс, если он включен
        let mut touchui_hover = false;
        let touchui_action = if let Some(touchui) = self.touchui.as_mut() {
            let a = touchui.process(ctx)?;
            touchui_hover = touchui.is_hovered();
            a
        } else {
            None
        };

        if let Some((action, pressed)) = touchui_action {
            if action == Action::Left {
                if pressed {
                    newlunawalk = self.game.send_luna_left(dt);
                }
            } else if action == Action::Right {
                if pressed {
                    newlunawalk = self.game.send_luna_right(dt);
                }
            } else {
                self.handle_action_event(ctx, action, pressed)?;
            }
        }

        // Затем обрабатываем события клавиатуры, а вот события мыши пропускаем, если мышь
        // была наведена на тот самый сенсорный интерфейс
        for event in events {
            if let Event::MouseDown { .. } | Event::MouseUp { .. } = event {
                if touchui_hover {
                    continue;
                }
            }
            if let Some((action, pressed)) = self.input_actions.match_event(*event) {
                self.handle_action_event(ctx, action, pressed)?;
            }
        }

        // Поведение из версии 0.5
        // if self.input_actions.pressed(ctx, Action::Left) {
        //     newlunawalk = self.game.send_luna_left(dt);
        // }
        // if self.input_actions.pressed(ctx, Action::Right) {
        //     newlunawalk = self.game.send_luna_right(dt);
        // }

        // Эмуляция поведения из версии 1.0: там код спроектирован так, что одновременно может
        // быть активно только одно действие, причём проверка идёт в определённом порядке
        let active_action = [
            Action::Switch,
            Action::Apply,
            Action::SelTeleport,
            Action::SelLaser,
            Action::SelChicken,
            Action::SelShield,
            Action::Left,
            Action::Right,
        ]
        .into_iter()
        .find(|&a| self.input_actions.pressed(ctx, a));
        if active_action == Some(Action::Left) {
            newlunawalk = self.game.send_luna_left(dt);
        } else if active_action == Some(Action::Right) {
            newlunawalk = self.game.send_luna_right(dt);
        }

        if let Some(sounds) = self.sounds.as_ref() {
            if newlunawalk && !self.islunawalk {
                ctx.play_sound(&sounds.snd_galop, 1.0, true)?;
            } else if !newlunawalk && self.islunawalk {
                ctx.stop_sound(&sounds.snd_galop)?;
            }
        }
        self.islunawalk = newlunawalk;

        let newcelestiawalk =
            !self.game.is_celestia_eating() && self.game.get_celestia_dir() != Direction::No;
        if let Some(sounds) = self.sounds.as_ref() {
            if newcelestiawalk && !self.iscelestiawalk {
                ctx.play_sound(&sounds.snd_galop2, 1.0, true)?;
            } else if !newcelestiawalk && self.iscelestiawalk {
                ctx.stop_sound(&sounds.snd_galop2)?;
            }
        }
        self.iscelestiawalk = newcelestiawalk;

        if self.oldcelestiazoneidx != self.game.get_celestia_zone_idx() {
            if let Some(sounds) = self.sounds.as_ref() {
                ctx.play_sound(&sounds.snd_teleport, 1.0, false)?;
            }
            self.oldcelestiazoneidx = self.game.get_celestia_zone_idx();
        }

        let newlaseron = self.game.get_laser_dir() != Direction::No;
        if let Some(sounds) = self.sounds.as_ref() {
            if newlaseron && !self.islaseron {
                ctx.play_sound(&sounds.snd_laser, 1.0, true)?;
            } else if !newlaseron && self.islaseron {
                ctx.stop_sound(&sounds.snd_laser)?;
            }
        }
        self.islaseron = newlaseron;

        match self.game.get_state() {
            GameState::Normal => {
                self.game.update(dt);
                if let Err(e) = self.common_data.achievements.update(&self.game) {
                    cake_engine::log::error!("Failed to update achievements: {:?}", e);
                }
            }
            GameState::Win(msg) => {
                return Ok(GameState::Win(msg.clone()));
            }
            GameState::Fail(msg) => {
                if let Some(falling_celestia) = self.falling_celestia.as_mut() {
                    falling_celestia.pos.x += falling_celestia.vel.x * dt;
                    falling_celestia.pos.y += falling_celestia.vel.y * dt;
                    falling_celestia.vel.y += GRAVITY * dt;

                    let bottom = self.game.zones().last().unwrap().y + game::BLOCKH + game::PONYW;
                    if falling_celestia.pos.y > bottom {
                        return Ok(GameState::Fail(msg.clone()));
                    }
                } else {
                    let celestia_pos = self.game.get_celestia_pos();
                    let luna_pos = self.game.get_luna_pos();
                    let eating = self.game.get_celestia_hp_percent().floor() <= 0.0;
                    self.falling_celestia = Some(FallingCelestia {
                        pos: celestia_pos,
                        vel: if eating {
                            Vec2::new(0.0, 0.0)
                        } else {
                            Vec2::new(100.0 * (celestia_pos.x - luna_pos.x).signum(), -100.0)
                        },
                        dir: self.game.get_celestia_dir(),
                        eating,
                    });
                }
            }
        }

        Ok(GameState::Normal)
    }
}

impl Scene for ScenePlay {
    fn process(&mut self, ctx: &mut dyn Context, dt: f32, events: &[Event]) -> Result<SceneResult> {
        if ctx.input().is_quit_requested() {
            return Ok(SceneResult::Quit);
        }

        self.common_data.process(ctx)?;

        if ctx.input().is_key_just_pressed(ScanCode::Escape)
            || self.touchui.is_some() && self.common_data.button_close.just_clicked()
        {
            let menu_scene = SceneMenu::new(self.common_data.clone(), ctx)?;
            return Ok(SceneResult::Switch(Box::new(menu_scene)));
        }

        let prev_action_id = self.current_action_id;

        match self.process_game(ctx, dt, events)? {
            GameState::Normal => {}
            GameState::Win(msg) => {
                let s = SceneGameOver::new(self.common_data.clone(), ctx, true, msg.clone());
                return Ok(SceneResult::Switch(Box::new(s)));
            }
            GameState::Fail(msg) => {
                let s = SceneGameOver::new(self.common_data.clone(), ctx, false, msg.clone());
                return Ok(SceneResult::Switch(Box::new(s)));
            }
        }

        self.celestia_walk.process(dt);
        self.celestia_eat.process(dt);
        self.luna_walk.process(dt);
        self.luna_wait.process(dt);
        self.laser.process(dt);
        self.shield.process(dt);

        self.mana_label
            .set_text(self.game.get_mana().floor() as i32);
        self.hp_label
            .set_text(self.game.get_celestia_hp_percent().floor() as i32);

        if self.current_action_id != prev_action_id {
            if let Some(touchui) = self.touchui.as_mut() {
                touchui.update_current_action(self.current_action_id);
            }
        }

        Ok(SceneResult::Normal)
    }

    fn render(&mut self, ctx: &mut dyn Context) -> Result<()> {
        let view = ctx.view().get_or_default(); // игровая область (по умолчанию 1024x768)
        let area = ctx.view().visible_area(); // вся область, видимая в окне

        self.common_data.draw_back(ctx)?;

        for zone in self.game.zones() {
            let n = (zone.right - zone.left).round() as i32 / game::BLOCKW as i32;
            for j in 0..n {
                ctx.draw_texture(
                    &self.block,
                    Vec2::new(zone.left + j as f32 * game::BLOCKW, zone.y),
                    Vec2::new(0.0, 0.0),
                )?;
            }
        }

        for falling_chicken in self.game.falling_chickens() {
            let mut src = self.chicken.rect();
            if falling_chicken.vel.x < 0.0 {
                src.flip_x();
            }
            ctx.draw_texture_ex(
                &self.chicken,
                DrawTextureParams {
                    src: Some(src),
                    origin: Vec2::new(0.5, 0.5),
                    position: Vec2::new(falling_chicken.pos.x, falling_chicken.pos.y - CHICKEN_Y),
                    rotation: falling_chicken.rotation,
                    ..Default::default()
                },
            )?;
        }

        for chicken in self.game.chickens() {
            let zone = self.game.zones()[chicken.zoneidx];
            let mut src = self.chicken.rect();
            if chicken.vx < 0.0 {
                src.flip_x();
            }
            ctx.draw_texture_ex(
                &self.chicken,
                DrawTextureParams {
                    src: Some(src),
                    origin: Vec2::new(0.5, 0.5),
                    position: Vec2::new(chicken.x, zone.y - CHICKEN_Y),
                    ..Default::default()
                },
            )?;
        }

        if let Some(falling_celestia) = self.falling_celestia.as_ref() {
            let celestia = if falling_celestia.eating {
                &mut self.celestia_eat
            } else {
                &mut self.celestia_walk
            };
            let mut p = falling_celestia.pos;
            p.y -= 128.0;
            celestia.set_position(p);
            celestia.set_flip_x(falling_celestia.dir == Direction::Left);
            celestia.render(ctx)?;
        } else {
            let celestia = if self.game.is_celestia_eating()
                || self.game.get_celestia_dir() == Direction::No
            {
                &mut self.celestia_eat
            } else {
                &mut self.celestia_walk
            };
            let mut p = self.game.get_celestia_pos();
            p.y -= 128.0;
            celestia.set_position(p);
            celestia.set_flip_x(self.game.get_celestia_dir() == Direction::Left);
            celestia.render(ctx)?;
        }

        let luna = if self.islunawalk {
            &mut self.luna_walk
        } else {
            &mut self.luna_wait
        };
        let mut p = self.game.get_luna_pos();
        p.y -= 126.0;
        luna.set_position(p);
        luna.set_flip_x(self.game.get_luna_dir() == Direction::Left);
        luna.render(ctx)?;

        for cake in self.game.cakes() {
            let zone = &self.game.zones()[cake.zoneidx];
            let cake_pos = Vec2::new(cake.x, zone.y - game::CAKE_Y);
            ctx.draw_texture(&self.cakes[cake.spriteidx], cake_pos, Vec2::new(0.5, 0.5))?;

            if cake.shieldleft > 0.0 {
                self.shield.set_position(cake_pos);
                self.shield.render(ctx)?;
            }

            if cake.hp < 1.0 {
                self.draw_indicator(
                    ctx,
                    cake.x - INDICATOR_W / 2.0,
                    zone.y,
                    INDICATOR_W,
                    INDICATOR_H,
                    cake.hp,
                    &COLORSET,
                )?;
            }
        }

        let luna_pos = self.game.get_luna_pos();
        let laser_width = self.laser.get_absolute_size().x;
        match self.game.get_laser_dir() {
            Direction::Right => {
                let mut start = luna_pos.x + 30.0;
                while start < area.x + area.width {
                    self.laser
                        .set_position(Vec2::new(start, luna_pos.y - LASER_Y));
                    self.laser.render(ctx)?;
                    start += laser_width;
                }
            }
            Direction::Left => {
                let mut start = luna_pos.x - 30.0 - laser_width;
                while start > area.x - laser_width {
                    self.laser
                        .set_position(Vec2::new(start, luna_pos.y - LASER_Y));
                    self.laser.render(ctx)?;
                    start -= laser_width;
                }
            }
            Direction::No => {}
        }

        // Резервирование места под кнопку закрытия в правом верхнем углу
        let indic_y = if self.touchui.is_some()
            && area.width < view.width + 128.0
            && area.height < view.height + 128.0
        {
            64.0
        } else {
            0.0
        };
        let indic_height = 700.0 - indic_y;

        self.mana_label
            .set_position(Vec2::new(view.width - 25.0, indic_y + 5.0));
        self.mana_label.render(ctx)?;
        self.hp_label.set_position(Vec2::new(25.0, indic_y + 5.0));
        self.hp_label.render(ctx)?;

        let mana_height =
            indic_height * self.game.get_mana().floor() / self.game.get_balance().max_mana;
        ctx.set_fill_color(MANA_COLOR);
        ctx.fill_rect(Rect::new(
            view.width - 40.0,
            view.height - mana_height,
            30.0,
            mana_height,
        ))?;

        let hp_height = indic_height * self.game.get_celestia_hp_percent().floor() / 100.0;
        ctx.set_fill_color(HP_COLOR);
        ctx.fill_rect(Rect::new(10.0, view.height - hp_height, 30.0, hp_height))?;

        if let Some(touchui) = self.touchui.as_mut() {
            touchui.render(ctx, &self.game, &self.gameactions)?;
            self.common_data.button_close.render(ctx)?;
        }

        self.common_data.draw_fps_counter(ctx)?;

        if self.touchui.is_some() && self.common_data.button_close.is_hovered() {
            self.common_data.draw_cursor(ctx)?;
        } else if self.common_data.render_cursor && ctx.input().is_mouse_entered() {
            let mxy = ctx.input().get_mouse_position();
            let action = &self.gameactions[self.current_action_id];
            if !action.is_allowed_at(&self.game, mxy) {
                ctx.draw_texture(&self.deny, mxy, Vec2::new(0.5, 0.5))?;
            }
            ctx.draw_texture(
                &self.action_textures[self.current_action_id],
                mxy,
                Vec2::new(0.5, 0.5),
            )?;
        }

        Ok(())
    }
}
