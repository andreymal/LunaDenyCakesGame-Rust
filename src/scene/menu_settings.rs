use crate::{
    common_data::CommonData,
    data::{options::OPTIONS, texts::get_text},
    scene::{menu::SceneMenu, menu_ctrl::SceneMenuCtrl},
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

pub struct SceneMenuSettings {
    common_data: CommonData,
    button_ctrl: Button,
    button_sound: Button,
    button_music: Button,
    button_fullscreen: Button,
    button_vsync: Option<Button>,
    button_fps_limit: Button,
    button_fps_counter: Button,
    button_touchui: Button,
    button_back: Button,
}

impl SceneMenuSettings {
    pub fn new(common_data: CommonData, ctx: &mut dyn Context) -> SceneMenuSettings {
        let view = ctx.view().get_or_default();

        let mut pos = Vec2::new(view.width / 2.0, TOP);
        let button_ctrl = btn!(common_data, get_text("menuctrl"), pos);

        pos.y += STEP;
        let button_sound = btn!(common_data, SceneMenuSettings::get_sound_label(), pos);

        pos.y += STEP;
        let button_music = btn!(common_data, SceneMenuSettings::get_music_label(), pos);

        pos.y += STEP;
        let button_fullscreen = btn!(common_data, SceneMenuSettings::get_fullscreen_label(), pos);

        let button_vsync = if cfg!(feature = "macroquad") {
            None
        } else {
            pos.y += STEP;
            Some(btn!(common_data, SceneMenuSettings::get_vsync_label(), pos))
        };

        pos.y += STEP;
        let button_fps_limit = btn!(common_data, SceneMenuSettings::get_fps_limit_label(), pos);

        pos.y += STEP;
        let button_fps_counter = btn!(common_data, SceneMenuSettings::get_fps_counter_label(), pos);

        pos.y += STEP;
        let button_touchui = btn!(common_data, SceneMenuSettings::get_touchui_label(), pos);

        pos.y += STEP;
        let button_back = btn!(common_data, get_text("menuback"), pos);

        SceneMenuSettings {
            common_data,
            button_ctrl,
            button_sound,
            button_music,
            button_fullscreen,
            button_vsync,
            button_fps_limit,
            button_fps_counter,
            button_touchui,
            button_back,
        }
    }

    fn get_sound_label() -> String {
        let options = OPTIONS.lock().unwrap();
        format!(
            "{} : {}",
            get_text("menusound"),
            get_text(if options.get_soundon() {
                "text_on"
            } else {
                "text_off"
            }),
        )
    }

    fn get_music_label() -> String {
        let options = OPTIONS.lock().unwrap();
        format!(
            "{} : {}",
            get_text("menumusic"),
            get_text(if options.get_musicon() {
                "text_on"
            } else {
                "text_off"
            }),
        )
    }

    fn get_fullscreen_label() -> String {
        let options = OPTIONS.lock().unwrap();
        format!(
            "{} : {}",
            get_text("menufullscreen"),
            get_text(if options.get_fullscreen() {
                "text_on"
            } else {
                "text_off"
            }),
        )
    }

    fn get_vsync_label() -> String {
        let options = OPTIONS.lock().unwrap();
        format!(
            "{} : {}",
            get_text("menuvsync"),
            get_text(if options.get_vsync() {
                "text_on"
            } else {
                "text_off"
            }),
        )
    }

    fn get_fps_limit_label() -> String {
        let fps_limit = OPTIONS.lock().unwrap().get_fps_limit();
        format!(
            "{} : {}",
            get_text("menufpslimit"),
            if fps_limit.is_finite() && !fps_limit.is_nan() && fps_limit > 0.0 {
                (fps_limit as i64).to_string()
            } else {
                "∞".to_string()
            }
        )
    }

    fn get_fps_counter_label() -> String {
        let options = OPTIONS.lock().unwrap();
        format!(
            "{} : {}",
            get_text("menufpscounter"),
            get_text(if options.get_show_fps_counter() {
                "text_on"
            } else {
                "text_off"
            }),
        )
    }

    fn get_touchui_label() -> String {
        let options = OPTIONS.lock().unwrap();
        format!(
            "{} : {}",
            get_text("menutouchui"),
            get_text(if options.get_touchui() {
                "text_on"
            } else {
                "text_off"
            }),
        )
    }
}

impl Scene for SceneMenuSettings {
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

        self.button_ctrl.process(ctx)?;
        self.button_sound.process(ctx)?;
        self.button_music.process(ctx)?;
        self.button_fullscreen.process(ctx)?;
        if let Some(b) = self.button_vsync.as_mut() {
            b.process(ctx)?;
        }
        self.button_fps_limit.process(ctx)?;
        self.button_fps_counter.process(ctx)?;
        self.button_touchui.process(ctx)?;
        self.button_back.process(ctx)?;

        if ctx.input().is_key_just_pressed(ScanCode::Escape) || self.button_back.just_clicked() {
            let s = SceneMenu::new(self.common_data.clone(), ctx)?;
            return Ok(SceneResult::Switch(Box::new(s)));
        }

        if self.button_ctrl.just_clicked() {
            let s = SceneMenuCtrl::new(self.common_data.clone(), ctx);
            return Ok(SceneResult::Switch(Box::new(s)));
        }

        if self.button_sound.just_clicked() {
            OPTIONS.lock().unwrap().invert_soundon();
            self.button_sound
                .set_text(SceneMenuSettings::get_sound_label());
        }

        if self.button_music.just_clicked() {
            let musicon = OPTIONS.lock().unwrap().invert_musicon();
            if musicon {
                self.common_data.play_music(ctx)?; // это подвисает в macroquad
            } else {
                ctx.stop_music()?;
            }
            self.button_music
                .set_text(SceneMenuSettings::get_music_label());
        }

        if self.button_fullscreen.just_clicked() {
            let fullscreen = OPTIONS.lock().unwrap().invert_fullscreen();
            ctx.set_fullscreen(fullscreen)?;
            self.button_fullscreen
                .set_text(SceneMenuSettings::get_fullscreen_label());
        }

        if let Some(b) = self.button_vsync.as_mut() {
            if b.just_clicked() {
                let vsync = OPTIONS.lock().unwrap().invert_vsync();
                ctx.set_vsync(vsync)?;
                b.set_text(SceneMenuSettings::get_vsync_label());
            }
        }

        if self.button_fps_limit.just_clicked() {
            let fps = OPTIONS.lock().unwrap().switch_fps_limit();
            ctx.set_fps_limit(fps);
            self.button_fps_limit
                .set_text(SceneMenuSettings::get_fps_limit_label());
        }

        if self.button_fps_counter.just_clicked() {
            let show = OPTIONS.lock().unwrap().invert_show_fps_counter();
            self.common_data.draw_fps_counter = show;
            self.button_fps_counter
                .set_text(SceneMenuSettings::get_fps_counter_label());
        }

        if self.button_touchui.just_clicked() {
            OPTIONS.lock().unwrap().invert_touchui();
            self.button_touchui
                .set_text(SceneMenuSettings::get_touchui_label());
        }

        Ok(SceneResult::Normal)
    }

    fn render(&mut self, ctx: &mut dyn Context) -> Result<()> {
        let view = ctx.view().get_or_default();

        self.common_data.draw_back(ctx)?;

        self.button_ctrl.render(ctx)?;
        self.button_sound.render(ctx)?;
        self.button_music.render(ctx)?;
        self.button_fullscreen.render(ctx)?;
        if let Some(b) = self.button_vsync.as_mut() {
            b.render(ctx)?;
        }
        self.button_fps_limit.render(ctx)?;
        self.button_fps_counter.render(ctx)?;
        self.button_touchui.render(ctx)?;
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

    fn stop(&mut self, _ctx: &mut dyn Context) -> Result<()> {
        if let Err(e) = OPTIONS.lock().unwrap().save() {
            cake_engine::log::error!("Failed to save options: {:?}", e);
        }
        Ok(())
    }
}
