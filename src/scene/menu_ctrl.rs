use crate::{
    action::Action,
    common_data::CommonData,
    data::{
        options::{key_to_human_string, OPTIONS},
        texts::get_text,
    },
    utils::{btn, btn_small},
};
use anyhow::Result;
use cake_engine::{
    button::Button,
    color::Color,
    context::Context,
    input::{Actions, Event, Key, MouseButton, ScanCode},
    label::Label,
    rect::Rect,
    scene::{Scene, SceneResult},
    vec::Vec2,
};

const TOP: f32 = 240.0;
const STEP: f32 = 54.0;

pub struct SceneMenuCtrl {
    common_data: CommonData,
    action_buttons: Vec<(Action, Button)>,
    checkbox_label: Label,
    checkbox_active: bool,
    button_back: Button,
    button_default: Button,
    actions: Actions<Action>,
    changing_action: Option<Action>,
}

impl SceneMenuCtrl {
    pub fn new(common_data: CommonData, ctx: &mut dyn Context) -> SceneMenuCtrl {
        let view = ctx.view().get_or_default();

        let mut pos = Vec2::new(view.width / 2.0, TOP);
        let mut action_buttons = Vec::new();
        for action in enum_iterator::all::<Action>() {
            let b = btn!(common_data, "", pos);
            action_buttons.push((action, b));
            pos.y += STEP;
        }

        pos.x = view.width / 2.0 + common_data.checkbox_on.width() as f32 / 2.0;
        pos.y -= 4.0;
        let mut checkbox_label = Label::new(common_data.font_small.clone(), Color::WHITE);
        checkbox_label.set_position(pos);
        checkbox_label.set_origin(Vec2::new(0.5, 0.5));
        checkbox_label.set_text(get_text("text_cb_apply_after_select"));
        let checkbox_active = OPTIONS.lock().unwrap().get_apply_after_select();
        pos.y += STEP + 8.0;

        pos.x = view.width / 2.0 - common_data.button.width() as f32 / 2.0;
        let mut button_back = btn_small!(common_data, get_text("menuback"), pos);
        button_back.set_origin(Vec2::new(0.0, 0.5));

        pos.x = view.width / 2.0 + common_data.button.width() as f32 / 2.0;
        let mut button_default = btn_small!(common_data, get_text("menudefault"), pos);
        button_default.set_origin(Vec2::new(1.0, 0.5));

        let mut actions = Actions::new();
        actions.replace_all(OPTIONS.lock().unwrap().get_keys());

        let mut menu = SceneMenuCtrl {
            common_data,
            action_buttons,
            checkbox_label,
            checkbox_active,
            button_back,
            button_default,
            actions,
            changing_action: None,
        };

        menu.update_action_buttons();

        menu
    }

    fn update_action_buttons(&mut self) {
        for (action, button) in self.action_buttons.iter_mut() {
            let name = get_text(&format!("action_{}", action.code()));
            let value = match self.actions.get_key_by_action(*action) {
                Some(key) => key_to_human_string(key),
                None => "???".to_string(),
            };
            button.set_text(format!("{} : {}", name, value));
        }
    }

    fn get_checkbox_bounding_rect(&self) -> Option<Rect> {
        if let Some(mut b) = self.checkbox_label.get_bounding_rect() {
            let cb_size = self.common_data.checkbox_on.size_vec();

            // Добавляем ширину галочки
            b.x -= cb_size.x;
            b.width += cb_size.x;

            // Добавляем высоту галочки, если она выше лейбла
            let dh = cb_size.y - b.height;
            if dh > 0.0 {
                b.height += dh;
                b.y -= dh / 2.0;
            }

            Some(b)
        } else {
            None
        }
    }

    fn is_mouse_over_checkbox(&self, ctx: &dyn Context) -> bool {
        let mxy = ctx.input().get_mouse_position();
        if let Some(b) = self.get_checkbox_bounding_rect() {
            b.contains_point(mxy)
        } else {
            false
        }
    }
}

impl Scene for SceneMenuCtrl {
    fn process(
        &mut self,
        ctx: &mut dyn Context,
        _dt: f32,
        events: &[Event],
    ) -> Result<SceneResult> {
        if ctx.input().is_quit_requested() {
            return Ok(SceneResult::Quit);
        }

        if let Some(action) = self.changing_action {
            for event in events {
                if let Event::KeyDown { scancode, .. } = event {
                    if *scancode != ScanCode::Escape {
                        self.actions.add(action, Key::Keyboard(*scancode));
                        self.changing_action = None;
                        break;
                    } else {
                        self.changing_action = None;
                        break;
                    }
                } else if let Event::MouseDown { button, .. } = event {
                    self.actions.add(action, Key::Mouse(*button));
                    self.changing_action = None;
                    break;
                }
            }
            if self.changing_action.is_none() {
                OPTIONS.lock().unwrap().set_keys(self.actions.mapping());
                self.update_action_buttons();
            }
        } else {
            self.common_data.process(ctx)?;

            for (action, button) in self.action_buttons.iter_mut() {
                button.process(ctx)?;
                if button.just_clicked() && self.changing_action.is_none() {
                    self.changing_action = Some(*action);
                    let name = get_text(&format!("action_{}", action.code()));
                    button.set_text(format!("{} :", name));
                }
            }

            self.button_back.process(ctx)?;
            self.button_default.process(ctx)?;

            if ctx.input().is_key_just_pressed(ScanCode::Escape) || self.button_back.just_clicked()
            {
                let s = crate::scene::menu_settings::SceneMenuSettings::new(
                    self.common_data.clone(),
                    ctx,
                );
                return Ok(SceneResult::Switch(Box::new(s)));
            }

            if self.button_default.just_clicked() {
                let mut options = OPTIONS.lock().unwrap();
                options.reset_controls_to_default();
                self.actions.replace_all(options.get_keys());
                self.checkbox_active = options.get_apply_after_select();
                self.update_action_buttons();
            }

            if ctx.input().is_mouse_button_just_pressed(MouseButton::Left)
                && self.is_mouse_over_checkbox(ctx)
            {
                self.checkbox_active = OPTIONS.lock().unwrap().switch_apply_after_select();
            }
        }

        Ok(SceneResult::Normal)
    }

    fn render(&mut self, ctx: &mut dyn Context) -> Result<()> {
        let view = ctx.view().get_or_default();

        self.common_data.draw_back(ctx)?;

        for (_, button) in self.action_buttons.iter_mut() {
            button.render(ctx)?;
        }

        self.checkbox_label.render(ctx)?;
        let b = self.checkbox_label.get_bounding_rect().unwrap();
        ctx.draw_texture(
            if self.checkbox_active {
                &self.common_data.checkbox_on
            } else {
                &self.common_data.checkbox_off
            },
            Vec2::new(b.x, b.y + b.height / 2.0),
            Vec2::new(1.0, 0.5),
        )?;

        self.button_back.render(ctx)?;
        self.button_default.render(ctx)?;

        ctx.draw_texture(
            &self.common_data.logo,
            Vec2::new(view.width / 2.0, 100.0),
            Vec2::new(0.5, 0.5),
        )?;

        self.common_data.draw_fps_counter(ctx)?;
        if self.changing_action.is_none() {
            self.common_data.draw_cursor(ctx)?;
        }
        Ok(())
    }
}
