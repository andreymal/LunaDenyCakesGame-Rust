use crate::{
    common_data::CommonData,
    data::{options::OPTIONS, texts::get_text},
    scene::{
        achievements::SceneAchievements, help::SceneHelp, menu_misc::SceneMenuMisc,
        menu_settings::SceneMenuSettings, play::ScenePlay,
    },
    utils::{btn, tex_lang},
};
use anyhow::Result;
use cake_engine::{
    button::Button,
    context::Context,
    input::{Event, ScanCode},
    scene::{Scene, SceneResult},
    texture::Texture,
    vec::Vec2,
};
use std::rc::Rc;

const TOP: f32 = 240.0;
const STEP: f32 = 54.0;

pub struct SceneMenu {
    common_data: CommonData,
    button_start: Button,
    button_diff: Button,
    button_achievments: Button,
    button_help: Button,
    button_settings: Button,
    button_misc: Button,
    button_lang: Button,
    button_exit: Button,
    lang_flag: Rc<Texture>,
}

impl SceneMenu {
    pub fn new(common_data: CommonData, ctx: &mut dyn Context) -> Result<SceneMenu> {
        let view = ctx.view().get_or_default();

        let mut pos = Vec2::new(view.width / 2.0, TOP);
        let button_start = btn!(common_data, get_text("menustart"), pos);

        pos.y += STEP;
        let button_diff = btn!(common_data, SceneMenu::get_diff_label(), pos);

        pos.y += STEP;
        let button_achievments = btn!(
            common_data,
            SceneMenu::get_achievements_label(&common_data),
            pos
        );

        pos.y += STEP;
        let button_help = btn!(common_data, get_text("menuhelp"), pos);

        pos.y += STEP;
        let button_settings = btn!(common_data, get_text("menusettings"), pos);

        pos.y += STEP;
        let button_misc = btn!(common_data, get_text("menumisc"), pos);

        pos.y += STEP;
        let button_lang = btn!(common_data, SceneMenu::get_lang_label(), pos);

        pos.y += STEP;
        let button_exit = btn!(common_data, get_text("menuexit"), pos);

        Ok(SceneMenu {
            common_data,
            button_start,
            button_diff,
            button_achievments,
            button_help,
            button_settings,
            button_misc,
            button_lang,
            button_exit,
            lang_flag: tex_lang!(ctx, "images/lang.png"),
        })
    }

    fn get_diff_label() -> String {
        let diff = OPTIONS.lock().unwrap().get_difficulty();
        let diff_label = get_text(&format!("text_{}", diff.code()));
        format!("{} : {}", get_text("menudiff"), diff_label)
    }

    fn get_achievements_label(c: &CommonData) -> String {
        format!(
            "{} ({}/{})",
            get_text("menuachievements"),
            c.achievements.completed_count(),
            c.achievements.count()
        )
    }

    fn get_lang_label() -> String {
        let options = OPTIONS.lock().unwrap();
        let mut lang = options.get_current_language().to_string();
        lang.make_ascii_uppercase();
        format!("{} : {}", get_text("menulang"), lang)
    }
}

impl Scene for SceneMenu {
    fn process(
        &mut self,
        ctx: &mut dyn Context,
        _dt: f32,
        _events: &[Event],
    ) -> Result<SceneResult> {
        if ctx.input().is_quit_requested() || ctx.input().is_key_just_pressed(ScanCode::Escape) {
            return Ok(SceneResult::Quit);
        }

        self.common_data.process(ctx)?;

        self.button_start.process(ctx)?;
        self.button_diff.process(ctx)?;
        self.button_achievments.process(ctx)?;
        self.button_help.process(ctx)?;
        self.button_settings.process(ctx)?;
        self.button_misc.process(ctx)?;
        self.button_lang.process(ctx)?;
        self.button_exit.process(ctx)?;

        if self.button_exit.just_clicked() {
            return Ok(SceneResult::Quit);
        }

        let mut options_changed = false;

        if self.button_start.just_clicked() {
            let s = ScenePlay::new(self.common_data.clone(), ctx)?;
            return Ok(SceneResult::Switch(Box::new(s)));
        }

        if self.button_achievments.just_clicked() {
            let s = SceneAchievements::new(self.common_data.clone(), ctx)?;
            return Ok(SceneResult::Switch(Box::new(s)));
        }

        if self.button_help.just_clicked() {
            let s = SceneHelp::new(self.common_data.clone(), ctx)?;
            return Ok(SceneResult::Switch(Box::new(s)));
        }

        if self.button_settings.just_clicked() {
            let s = SceneMenuSettings::new(self.common_data.clone(), ctx);
            return Ok(SceneResult::Switch(Box::new(s)));
        }

        if self.button_misc.just_clicked() {
            let s = SceneMenuMisc::new(self.common_data.clone(), ctx);
            return Ok(SceneResult::Switch(Box::new(s)));
        }

        if self.button_lang.just_clicked() {
            {
                let mut options = OPTIONS.lock().unwrap();
                options.switch_current_language().to_string();
                crate::data::reload_lang(&options)?;
                ctx.reload_lang_resources()?;
                if let Err(e) = options.save() {
                    cake_engine::log::error!("Failed to save options: {:?}", e);
                }
            };

            let s = SceneMenu::new(self.common_data.clone(), ctx)?;
            return Ok(SceneResult::Switch(Box::new(s)));
        }

        if self.button_diff.just_clicked() {
            OPTIONS.lock().unwrap().switch_difficulty();
            self.button_diff.set_text(SceneMenu::get_diff_label());
            options_changed = true;
        }

        if options_changed {
            if let Err(e) = OPTIONS.lock().unwrap().save() {
                cake_engine::log::error!("Failed to save options: {:?}", e);
            }
        }

        Ok(SceneResult::Normal)
    }

    fn render(&mut self, ctx: &mut dyn Context) -> Result<()> {
        let view = ctx.view().get_or_default();

        self.common_data.draw_back(ctx)?;

        self.button_start.render(ctx)?;
        self.button_diff.render(ctx)?;
        self.button_achievments.render(ctx)?;
        self.button_help.render(ctx)?;
        self.button_settings.render(ctx)?;
        self.button_misc.render(ctx)?;
        self.button_lang.render(ctx)?;
        self.button_exit.render(ctx)?;

        if let Some(l) = self.button_lang.label_mut() {
            let b = l.get_bounding_rect().unwrap();
            ctx.draw_texture(
                &self.lang_flag,
                Vec2::new(b.x + b.width + 10.0, b.y + b.height / 2.0),
                Vec2::new(0.0, 0.5),
            )?;
        }

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
