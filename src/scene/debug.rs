use crate::{
    common_data::CommonData,
    data::{
        options::{Options, OPTIONS},
        texts::get_text,
    },
    dvd::Dvd,
    utils::tex,
};
use anyhow::Result;
use cake_engine::{
    color::Color,
    context::Context,
    input::{Event, ScanCode},
    label::Label,
    scene::{Scene, SceneResult},
    vec::Vec2,
};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

const MARGIN: f32 = 8.0;

pub struct SceneDebug {
    common_data: CommonData,
    prev_scene: Option<Box<dyn Scene>>,
    sys_label: Label,
    size_label: Label,
    events_label: Label,
    events_list_labels: Vec<Label>,
    cake: Dvd,
}

impl SceneDebug {
    pub fn new(common_data: CommonData, ctx: &mut dyn Context) -> Result<SceneDebug> {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut rng = ChaCha8Rng::seed_from_u64(seed);

        let backend_info = format!("{}: {}", get_text("debug_backend"), ctx.get_backend_name());

        let lang_info = {
            let options = OPTIONS.lock().unwrap();
            format!(
                "{}: {} / {:?}\n\
                {}: {}",
                get_text("debug_syslang"),
                Options::get_system_language(),
                sys_locale::get_locale(),
                get_text("debug_gamelang"),
                options.get_current_language(),
            )
        };

        let fs_info = if let Some(data_dir) = crate::data::data_dir() {
            format!(
                "{}: {:?}\n\
                {}: {}\n\
                {}: {}",
                get_text("debug_storagepath"),
                &data_dir,
                get_text("debug_storagepath_exists"),
                &match data_dir.try_exists() {
                    Ok(true) => get_text("debug_storagepath_exists_yes"),
                    Ok(false) => get_text("debug_storagepath_exists_no"),
                    Err(e) => format!("{}: {:?}", get_text("debug_storagepath_exists_error"), e),
                },
                get_text("debug_storagepath_available"),
                &match std::fs::read_dir(&data_dir) {
                    Ok(_) => get_text("debug_storagepath_available_yes"),
                    Err(e) => format!("{}: {:?}", get_text("debug_storagepath_available_no"), e),
                },
            )
        } else {
            format!(
                "{}: {}",
                get_text("debug_storagepath"),
                get_text("debug_storagepath_none")
            )
        };

        let mut sys_label = Label::new(common_data.font_small.clone(), Color::BLACK);
        sys_label.set_text(format!("{}\n{}\n{}", backend_info, lang_info, fs_info));

        let size_label = Label::new(common_data.font_small.clone(), Color::BLACK);

        let mut events_label = Label::new(common_data.font_small.clone(), Color::BLACK);
        events_label.set_text(get_text("debug_lastevents"));

        let cake_textures = vec![
            tex!(ctx, "images/cake1.png"),
            tex!(ctx, "images/cake2.png"),
            tex!(ctx, "images/cake3.png"),
        ];

        let cake = Dvd::new(
            cake_textures,
            Vec2::new(
                rng.gen_range(300..=600) as f32,
                rng.gen_range(150..=300) as f32,
            ),
            Vec2::new(300.0, 150.0),
        );

        Ok(SceneDebug {
            common_data,
            prev_scene: None,
            sys_label,
            size_label,
            events_label,
            events_list_labels: Vec::new(),
            cake,
        })
    }
}

impl Scene for SceneDebug {
    fn start(&mut self, _ctx: &mut dyn Context, prev_scene: Option<Box<dyn Scene>>) -> Result<()> {
        self.prev_scene = prev_scene;
        Ok(())
    }

    fn process(&mut self, ctx: &mut dyn Context, dt: f32, events: &[Event]) -> Result<SceneResult> {
        for event in events {
            let event_string = format!("[{:.3}] {:?}", ctx.time().uptime().as_secs_f32(), event);

            let mut l = Label::new(self.common_data.font_small.clone(), Color::BLACK);
            l.set_text(event_string);

            self.events_list_labels.push(l);

            if self.events_list_labels.len() > 100 {
                let mut l = self.events_list_labels.remove(0);
                l.clear_cache(ctx);
            }
        }

        self.common_data.process(ctx)?;

        if ctx.input().is_quit_requested() {
            return Ok(SceneResult::Quit);
        }

        if ctx.input().is_key_just_pressed(ScanCode::Escape)
            || self.common_data.button_close.just_clicked()
        {
            if let Some(s) = self.prev_scene.take() {
                return Ok(SceneResult::Switch(s));
            }
            return Ok(SceneResult::Quit);
        }

        let physical_size = ctx.get_physical_window_size();
        let logical_size = ctx.get_logical_window_size();
        let dpi_scale = ctx.get_dpi_scale();
        let mpos = ctx.input().get_mouse_position();

        let size_text = format!(
            "{}: {}x{}\n\
            {}: {}x{}\n\
            {}: {}x{}\n\
            {}: {:?}\n\
            {}: {:?}\n\
            {}: {:.3}x{:.3}",
            get_text("debug_physicalsize"),
            physical_size.0,
            physical_size.1,
            get_text("debug_logicalsize"),
            logical_size.x,
            logical_size.y,
            get_text("debug_scale"),
            dpi_scale.x,
            dpi_scale.y,
            get_text("debug_view"),
            ctx.view().get(),
            get_text("debug_visiblearea"),
            ctx.view().visible_area(),
            get_text("debug_mouseposition"),
            mpos.x,
            mpos.y,
        );

        self.size_label.set_text(size_text);

        self.cake.process(ctx, dt)?;

        Ok(SceneResult::Normal)
    }

    fn render(&mut self, ctx: &mut dyn Context) -> Result<()> {
        let v = ctx.view().visible_area();

        self.common_data.draw_back(ctx)?;

        self.cake.render(ctx)?;

        self.sys_label.set_position(Vec2::new(MARGIN, MARGIN));
        self.sys_label.render(ctx)?;
        let mut g = self.sys_label.get_bounding_rect().unwrap();

        self.size_label
            .set_position(Vec2::new(MARGIN, g.y + g.height + MARGIN));
        self.size_label.render(ctx)?;
        g = self.size_label.get_bounding_rect().unwrap();

        self.events_label
            .set_position(Vec2::new(MARGIN, g.y + g.height + MARGIN));
        self.events_label.render(ctx)?;
        g = self.events_label.get_bounding_rect().unwrap();

        for l in self.events_list_labels.iter_mut().rev() {
            l.set_position(Vec2::new(MARGIN, g.y + g.height));
            l.render(ctx)?;
            g = l.get_bounding_rect().unwrap();
            if g.y > v.height {
                break;
            }
        }

        self.common_data.button_close.render(ctx)?;
        self.common_data.draw_fps_counter(ctx)?;
        self.common_data.draw_cursor(ctx)?;
        Ok(())
    }
}
