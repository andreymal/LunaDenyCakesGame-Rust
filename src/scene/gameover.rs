use crate::{
    common_data::CommonData,
    data::texts::get_text,
    scene::{menu::SceneMenu, play::ScenePlay},
    utils::btn_small,
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
    vec::Vec2,
};

const BUT_Y: f32 = 730.0;

pub struct SceneGameOver {
    common_data: CommonData,
    label: Label,
    button_restart: Button,
    button_menu: Button,
}

impl SceneGameOver {
    pub fn new(
        common_data: CommonData,
        ctx: &mut dyn Context,
        iswin: bool,
        msg: String,
    ) -> SceneGameOver {
        let view = ctx.view().get_or_default();

        let mut label = Label::new(
            common_data.font_big.clone(),
            if iswin {
                Color::new(70, 255, 0, 255)
            } else {
                Color::new(255, 0, 0, 255)
            },
        );
        label.set_text_align(0.5);
        label.set_origin(Vec2::new(0.5, 0.0));
        label.set_text(format!(
            "{}\n{}",
            if iswin {
                get_text("text_win")
            } else {
                get_text("text_fail")
            },
            msg,
        ));

        let mut button_restart = btn_small!(
            common_data,
            get_text("menurestart"),
            Vec2::new(
                view.width / 2.0 - common_data.button.width() as f32 / 2.0,
                BUT_Y,
            )
        );
        button_restart.set_origin(Vec2::new(0.0, 0.5));

        let mut button_menu = btn_small!(
            common_data,
            get_text("menumenu"),
            Vec2::new(
                view.width / 2.0 + common_data.button.width() as f32 / 2.0,
                BUT_Y,
            )
        );
        button_menu.set_origin(Vec2::new(1.0, 0.5));

        SceneGameOver {
            common_data,
            label,
            button_restart,
            button_menu,
        }
    }
}

impl Scene for SceneGameOver {
    fn process(
        &mut self,
        ctx: &mut dyn Context,
        _dt: f32,
        _events: &[Event],
    ) -> Result<SceneResult> {
        self.common_data.process(ctx)?;

        self.button_restart.process(ctx)?;
        self.button_menu.process(ctx)?;

        if ctx.input().is_key_just_pressed(ScanCode::Escape) || self.button_menu.just_clicked() {
            let s = SceneMenu::new(self.common_data.clone(), ctx)?;
            return Ok(SceneResult::Switch(Box::new(s)));
        }

        if self.button_restart.just_clicked() {
            let s = ScenePlay::new(self.common_data.clone(), ctx)?;
            return Ok(SceneResult::Switch(Box::new(s)));
        }

        Ok(SceneResult::Normal)
    }

    fn render(&mut self, ctx: &mut dyn Context) -> Result<()> {
        let view = ctx.view().get_or_default();

        self.common_data.draw_back(ctx)?;

        ctx.set_fill_color(Color::new(40, 40, 40, 128));
        ctx.fill_rect(Rect::new(200.0, 100.0, view.width - 400.0, 300.0))?;

        self.label.set_max_width(view.width - 400.0);
        self.label.set_position(Vec2::new(view.width / 2.0, 150.0));
        self.label.render(ctx)?;

        self.button_restart.render(ctx)?;
        self.button_menu.render(ctx)?;

        self.common_data.draw_fps_counter(ctx)?;
        self.common_data.draw_cursor(ctx)?;
        Ok(())
    }
}
