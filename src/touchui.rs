use crate::{action::Action, game::Game, gameaction::GameAction, utils::tex};
use anyhow::Result;
use cake_engine::{
    button::Button,
    color::Color,
    context::{Context, DrawTextureParams},
    input::MouseButton,
    texture::Texture,
    vec::Vec2,
};
use std::rc::Rc;

const COLOR_INACTIVE: Color = Color::new(255, 255, 255, 160);
const COLOR_ACTIVE: Color = Color::WHITE;
const BUTTON_SIZE: f32 = 40.0;
const SCALE: f32 = 3.0;
const ACTION_SCALE: f32 = 2.0;

pub struct TouchUi {
    button_tex: Rc<Texture>,
    button_active_tex: Rc<Texture>,
    deny: Rc<Texture>,
    arrow_tex: Rc<Texture>,
    action_textures: [Rc<Texture>; 4],
    action_buttons: [Button; 4],
    button_left: Button,
    button_right: Button,
    current_action_id: usize,
    input_actions: [Action; 4],
    action_hover: Option<Action>,
}

impl TouchUi {
    pub fn new(
        ctx: &mut dyn Context,
        action_textures: [Rc<Texture>; 4],
        input_actions: [Action; 4],
        deny: Rc<Texture>,
    ) -> Result<TouchUi> {
        let button_tex = tex!(ctx, "images/touchui_button.png");
        let button_active_tex = tex!(ctx, "images/touchui_button_active.png");

        let mut button_base = Button::new(button_tex.clone(), Vec2::new(0.0, 0.0));
        button_base.set_color(COLOR_INACTIVE);
        button_base.set_color_hover(COLOR_ACTIVE);
        button_base.set_scale(Vec2::new(SCALE, SCALE));

        let mut touchui = TouchUi {
            button_tex,
            button_active_tex,
            deny,
            arrow_tex: tex!(ctx, "images/touchui_arrow.png"),
            action_buttons: [
                button_base.clone(),
                button_base.clone(),
                button_base.clone(),
                button_base.clone(),
            ],
            button_left: button_base.clone(),
            button_right: button_base.clone(),
            action_textures,
            current_action_id: 0,
            input_actions,
            action_hover: None,
        };
        touchui.resize(ctx)?;
        touchui.update_current_action(touchui.current_action_id);
        Ok(touchui)
    }

    pub fn resize(&mut self, ctx: &mut dyn Context) -> Result<()> {
        let view = ctx.view().visible_area();
        let left = view.x + 64.0;
        let right = view.x + view.width - 64.0;
        let middle = view.y + view.height / 2.0;

        self.button_left.set_origin(Vec2::new(0.0, 0.0));
        self.button_left.set_position(Vec2::new(left, middle));

        self.button_right.set_origin(Vec2::new(1.0, 0.0));
        self.button_right.set_position(Vec2::new(right, middle));

        for (i, b) in self.action_buttons.iter_mut().enumerate() {
            let y = middle
                - BUTTON_SIZE * SCALE
                - 64.0
                - if i % 2 == 0 {
                    // Нечётные действия наверху
                    BUTTON_SIZE * SCALE + 16.0
                } else {
                    // Чётные действия внизу
                    0.0
                };

            if i % 4 < 2 {
                // Первые два действия слева
                b.set_origin(Vec2::new(0.0, 0.0));
                b.set_position(Vec2::new(left, y));
            } else {
                // Последние два действия справа
                b.set_origin(Vec2::new(1.0, 0.0));
                b.set_position(Vec2::new(right, y));
            }
        }

        Ok(())
    }

    pub fn update_current_action(&mut self, new_action_id: usize) {
        let old_button = &mut self.action_buttons[self.current_action_id];
        old_button.set_texture(self.button_tex.clone());
        old_button.set_texture_hover(self.button_tex.clone());

        let new_button = &mut self.action_buttons[new_action_id];
        new_button.set_texture(self.button_active_tex.clone());
        new_button.set_texture_hover(self.button_active_tex.clone());

        self.current_action_id = new_action_id;
    }

    pub fn is_hovered(&self) -> bool {
        self.action_hover.is_some()
    }

    pub fn process(&mut self, ctx: &mut dyn Context) -> Result<Option<(Action, bool)>> {
        if ctx.view().is_changed() {
            self.resize(ctx)?;
        }

        self.button_left.process(ctx)?;
        self.button_right.process(ctx)?;
        for btn in self.action_buttons.iter_mut() {
            btn.process(ctx)?;
        }

        let prev_action_hover = self.action_hover;

        // Ищем, на какую кнопку наведена мышка/палец
        self.action_hover = None;
        if self.button_left.is_hovered() {
            self.action_hover = Some(Action::Left);
        } else if self.button_right.is_hovered() {
            self.action_hover = Some(Action::Right);
        } else {
            for (btn, act) in self.action_buttons.iter_mut().zip(self.input_actions) {
                if btn.is_hovered() {
                    self.action_hover = Some(act);
                    break;
                }
            }
        }

        // Если наведено и нажато, то запускаем/продолжаем действие
        if let Some(act) = self.action_hover {
            if ctx.input().is_mouse_button_pressed(MouseButton::Left) {
                return Ok(Some((act, true)));
            }
        }

        // Если не наведено или не нажато, то останавливаем ранее запущенное действие
        if let Some(act) = prev_action_hover {
            if ctx.input().is_mouse_button_just_released(MouseButton::Left) {
                return Ok(Some((act, false)));
            }
        }

        Ok(None)
    }

    pub fn render(
        &mut self,
        ctx: &mut dyn Context,
        game: &Game,
        gameactions: &[Box<dyn GameAction>],
    ) -> Result<()> {
        self.button_left.render(ctx)?;
        ctx.draw_texture_ex(
            &self.arrow_tex,
            DrawTextureParams {
                origin: Vec2::new(0.5, 0.5),
                position: self.button_left.get_bounding_rect().get_center(),
                scale: Vec2::new(-SCALE, SCALE),
                color: if self.button_left.is_hovered() {
                    COLOR_ACTIVE
                } else {
                    COLOR_INACTIVE
                },
                ..Default::default()
            },
        )?;

        self.button_right.render(ctx)?;
        ctx.draw_texture_ex(
            &self.arrow_tex,
            DrawTextureParams {
                origin: Vec2::new(0.5, 0.5),
                position: self.button_right.get_bounding_rect().get_center(),
                scale: Vec2::new(SCALE, SCALE),
                color: if self.button_right.is_hovered() {
                    COLOR_ACTIVE
                } else {
                    COLOR_INACTIVE
                },
                ..Default::default()
            },
        )?;

        for (i, (btn, act)) in self.action_buttons.iter_mut().zip(gameactions).enumerate() {
            btn.render(ctx)?;

            let action_tex = &self.action_textures[i];
            let action_pos = btn.get_bounding_rect().get_center();
            if !act.is_allowed_somewhere(game) {
                ctx.draw_texture_ex(
                    &self.deny,
                    DrawTextureParams {
                        origin: Vec2::new(0.5, 0.5),
                        position: action_pos,
                        scale: Vec2::new(ACTION_SCALE, ACTION_SCALE),
                        ..Default::default()
                    },
                )?;
            }
            ctx.draw_texture_ex(
                action_tex,
                DrawTextureParams {
                    origin: Vec2::new(0.5, 0.5),
                    position: action_pos,
                    scale: Vec2::new(ACTION_SCALE, ACTION_SCALE),
                    ..Default::default()
                },
            )?;
        }

        Ok(())
    }
}
