use crate::{
    common_data::CommonData,
    data::texts::get_text,
    scene::menu::SceneMenu,
    utils::{btn_small, tex},
};
use anyhow::Result;
use cake_engine::{
    button::Button,
    color::Color,
    context::Context,
    input::{Event, ScanCode},
    label::Label,
    rect::Rect,
    scene::{Scene, SceneResult},
    texture::Texture,
    vec::Vec2,
};
use std::rc::Rc;

const TOP: f32 = 250.0;
const STEP: f32 = 54.0;
const BUT_Y: f32 = 730.0;

pub struct SceneAchievements {
    common_data: CommonData,
    ok: Rc<Texture>,
    cancel: Rc<Texture>,
    button_back: Button,
    button_reset: Button,
    labels: Vec<Label>,
}

impl SceneAchievements {
    pub fn new(common_data: CommonData, ctx: &mut dyn Context) -> Result<SceneAchievements> {
        let view = ctx.view().get_or_default();

        let mut button_back = btn_small!(
            common_data,
            get_text("menuback"),
            Vec2::new(
                view.width / 2.0 - common_data.button.width() as f32 / 2.0,
                BUT_Y,
            )
        );
        button_back.set_origin(Vec2::new(0.0, 0.5));

        let mut button_reset = btn_small!(
            common_data,
            get_text("menureset"),
            Vec2::new(
                view.width / 2.0 + common_data.button.width() as f32 / 2.0,
                BUT_Y,
            )
        );
        button_reset.set_origin(Vec2::new(1.0, 0.5));

        let labels: Vec<_> = common_data
            .achievements
            .list()
            .iter()
            .enumerate()
            .map(|(i, (a, _))| {
                let mut l = Label::new(common_data.font_button.clone(), Color::WHITE);
                l.set_text(get_text(&format!("achievement_{}", a.code())));
                l.set_position(Vec2::new(view.width / 2.0, TOP + STEP * i as f32 - 2.0));
                l.set_text_align(0.5);
                l.set_origin(Vec2::new(0.5, 0.5));
                l
            })
            .collect();

        Ok(SceneAchievements {
            common_data,
            ok: tex!(ctx, "images/ok.png"),
            cancel: tex!(ctx, "images/cancel.png"),
            button_back,
            button_reset,
            labels,
        })
    }
}

impl Scene for SceneAchievements {
    fn process(
        &mut self,
        ctx: &mut dyn Context,
        _dt: f32,
        _events: &[Event],
    ) -> Result<SceneResult> {
        self.common_data.process(ctx)?;

        self.button_back.process(ctx)?;
        self.button_reset.process(ctx)?;

        if ctx.input().is_key_just_pressed(ScanCode::Escape) || self.button_back.just_clicked() {
            let s = SceneMenu::new(self.common_data.clone(), ctx)?;
            return Ok(SceneResult::Switch(Box::new(s)));
        }

        if self.button_reset.just_clicked() {
            if let Err(e) = self.common_data.achievements.reset_achievements() {
                cake_engine::log::error!("Failed to reset achievements: {:?}", e);
            }
        }

        Ok(SceneResult::Normal)
    }

    fn render(&mut self, ctx: &mut dyn Context) -> Result<()> {
        let view = ctx.view().get_or_default();

        self.common_data.draw_back(ctx)?;

        ctx.set_fill_color(Color::new(40, 40, 40, 128));
        ctx.fill_rect(Rect::new(100.0, 180.0, view.width - 200.0, 450.0))?;

        for (i, ((a, _), label)) in self
            .common_data
            .achievements
            .list()
            .iter() // (a, _)
            .zip(self.labels.iter_mut()) // ((a, _), label)
            .enumerate()
        {
            label.render(ctx)?;
            let tex = if self.common_data.achievements.is_completed(a.code()) {
                &self.ok
            } else {
                &self.cancel
            };
            ctx.draw_texture(
                tex,
                Vec2::new(
                    label.get_bounding_rect().unwrap().x - 56.0,
                    TOP + STEP * i as f32 - 28.0,
                ),
                Vec2::new(0.0, 0.0),
            )?;
        }

        self.button_back.render(ctx)?;
        self.button_reset.render(ctx)?;

        ctx.draw_texture(
            &self.common_data.logo,
            Vec2::new(view.width / 2.0, 100.0),
            Vec2::new(0.5, 0.5),
        )?;

        self.common_data.draw_fps_counter(ctx)?;
        self.common_data.draw_cursor(ctx)?;
        Ok(())
    }
}
