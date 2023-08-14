use crate::{
    common_data::CommonData, data::texts::get_text, scene::menu_misc::SceneMenuMisc,
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

const BUT_Y: f32 = 700.0;

pub struct SceneBenchResult {
    common_data: CommonData,
    label: Label,
    button_back: Button,
}

impl SceneBenchResult {
    pub fn new(
        common_data: CommonData,
        ctx: &mut dyn Context,
        cakes_count: usize,
        fps: f32,
    ) -> Result<SceneBenchResult> {
        let view = ctx.view().get_or_default();
        let window_size = ctx.get_physical_window_size();

        let text = get_text("bench_result").replace("%COUNT%", &cakes_count.to_string());
        let text = format!(
            "{}\n\
            {}x{}\n\
            {:.3} FPS",
            text, window_size.0, window_size.1, fps
        );

        let mut label = Label::new(common_data.font_big.clone(), Color::WHITE);
        label.set_max_width(view.width - 60.0);
        label.set_text(text);
        label.set_position(Vec2::new(view.width / 2.0, 80.0 + 520.0 / 2.0));
        label.set_origin(Vec2::new(0.5, 0.5));
        label.set_text_align(0.5);

        let mut button_back = btn_small!(
            common_data,
            get_text("menuback"),
            Vec2::new(view.width / 2.0, BUT_Y)
        );
        button_back.set_origin(Vec2::new(0.5, 0.5));

        Ok(SceneBenchResult {
            common_data,
            label,
            button_back,
        })
    }
}

impl Scene for SceneBenchResult {
    fn process(
        &mut self,
        ctx: &mut dyn Context,
        _dt: f32,
        _events: &[Event],
    ) -> Result<SceneResult> {
        self.common_data.process(ctx)?;
        self.button_back.process(ctx)?;

        if ctx.input().is_key_just_pressed(ScanCode::Escape) || self.button_back.just_clicked() {
            let s = SceneMenuMisc::new(self.common_data.clone(), ctx);
            return Ok(SceneResult::Switch(Box::new(s)));
        }

        Ok(SceneResult::Normal)
    }

    fn render(&mut self, ctx: &mut dyn Context) -> Result<()> {
        let view = ctx.view().get_or_default();

        self.common_data.draw_back(ctx)?;

        ctx.set_fill_color(Color::new(40, 40, 40, 128));
        ctx.fill_rect(Rect::new(20.0, 80.0, view.width - 40.0, 520.0))?;

        self.label.render(ctx)?;
        self.button_back.render(ctx)?;

        self.common_data.draw_fps_counter(ctx)?;
        self.common_data.draw_cursor(ctx)?;
        Ok(())
    }
}
