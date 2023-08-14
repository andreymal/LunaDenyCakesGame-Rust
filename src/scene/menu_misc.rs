use crate::{
    common_data::CommonData,
    data::texts::get_text,
    scene::{about::SceneAbout, bench::SceneBench, debug::SceneDebug, menu::SceneMenu},
    utils::btn,
};
use anyhow::Result;
use cake_engine::{
    button::Button,
    context::Context,
    input::{Event, ScanCode},
    scene::{Scene, SceneResult},
    vec::Vec2,
};

const TOP: f32 = 240.0;
const STEP: f32 = 54.0;

pub struct SceneMenuMisc {
    common_data: CommonData,
    button_bench: Button,
    button_debug: Button,
    button_about: Button,
    button_back: Button,
}

impl SceneMenuMisc {
    pub fn new(common_data: CommonData, ctx: &mut dyn Context) -> SceneMenuMisc {
        let view = ctx.view().get_or_default();

        let mut pos = Vec2::new(view.width / 2.0, TOP);
        let button_bench = btn!(common_data, get_text("menubench"), pos);

        pos.y += STEP;
        let button_debug = btn!(common_data, get_text("menudebug"), pos);

        pos.y += STEP;
        let button_about = btn!(common_data, get_text("menuabout"), pos);

        pos.y += STEP;
        let button_back = btn!(common_data, get_text("menuback"), pos);

        SceneMenuMisc {
            common_data,
            button_bench,
            button_debug,
            button_about,
            button_back,
        }
    }
}

impl Scene for SceneMenuMisc {
    fn process(
        &mut self,
        ctx: &mut dyn Context,
        _dt: f32,
        _events: &[Event],
    ) -> Result<SceneResult> {
        if ctx.input().is_quit_requested() {
            return Ok(SceneResult::Quit);
        }

        self.common_data.process(ctx)?;

        self.button_bench.process(ctx)?;
        self.button_debug.process(ctx)?;
        self.button_about.process(ctx)?;
        self.button_back.process(ctx)?;

        if ctx.input().is_key_just_pressed(ScanCode::Escape) || self.button_back.just_clicked() {
            let s = SceneMenu::new(self.common_data.clone(), ctx)?;
            return Ok(SceneResult::Switch(Box::new(s)));
        }

        if self.button_bench.just_clicked() {
            let s = SceneBench::new(self.common_data.clone(), ctx)?;
            return Ok(SceneResult::Switch(Box::new(s)));
        }

        if self.button_debug.just_clicked() {
            let s = SceneDebug::new(self.common_data.clone(), ctx)?;
            return Ok(SceneResult::Switch(Box::new(s)));
        }

        if self.button_about.just_clicked() {
            let s = SceneAbout::new(self.common_data.clone(), ctx)?;
            return Ok(SceneResult::Switch(Box::new(s)));
        }

        Ok(SceneResult::Normal)
    }

    fn render(&mut self, ctx: &mut dyn Context) -> Result<()> {
        let view = ctx.view().get_or_default();

        self.common_data.draw_back(ctx)?;

        self.button_bench.render(ctx)?;
        self.button_debug.render(ctx)?;
        self.button_about.render(ctx)?;
        self.button_back.render(ctx)?;

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
