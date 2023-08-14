use crate::{
    common_data::CommonData,
    data::{options::OPTIONS, texts::get_text},
    scene::{bench_result::SceneBenchResult, menu_misc::SceneMenuMisc},
    utils::tex,
};
use anyhow::Result;
use cake_engine::{
    color::Color,
    context::Context,
    input::{Event, ScanCode},
    label::Label,
    scene::{Scene, SceneResult},
    texture::Texture,
    vec::Vec2,
};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::{
    ops::Range,
    rc::Rc,
    time::{Duration, Instant},
};

const RANGE_X: Range<i32> = 24..1000;
const RANGE_Y: Range<i32> = 24..744;
const TARGET_FPS: u32 = 30;
const FRAMETIME_THRESHOLD: Duration = Duration::new(0, 1_000_000_000 / TARGET_FPS);

pub struct SceneBench {
    common_data: CommonData,
    cake_textures: [Rc<Texture>; 3],
    cakes: Vec<(usize, Vec2)>,
    last_cakes_count_diff: usize,
    rng: ChaCha8Rng,
    stage: u32,
    warmup: bool,
    frame_count: u32,
    frame_tm: Instant,
    fps: f32,
    stage_label: Label,
    cakes_count_label: Label,
}

impl SceneBench {
    pub fn new(common_data: CommonData, ctx: &mut dyn Context) -> Result<SceneBench> {
        let cake_textures = [
            tex!(ctx, "images/cake1.png"),
            tex!(ctx, "images/cake2.png"),
            tex!(ctx, "images/cake3.png"),
        ];
        let mut rng = ChaCha8Rng::seed_from_u64(2010_10_10);
        let mut cakes = Vec::new();
        for _ in 0..16 {
            cakes.push((
                rng.gen_range(0..cake_textures.len()),
                Vec2::new(rng.gen_range(RANGE_X) as f32, rng.gen_range(RANGE_Y) as f32),
            ));
        }

        let mut stage_label = Label::new(common_data.font_main.clone(), Color::WHITE);
        stage_label.set_shadow(Color::BLACK, Vec2::new(1.0, 1.0));
        stage_label.set_text(get_text("bench_stage1"));

        let mut cakes_count_label = Label::new(common_data.font_main.clone(), Color::WHITE);
        cakes_count_label.set_shadow(Color::BLACK, Vec2::new(1.0, 1.0));
        cakes_count_label.set_position(Vec2::new(
            0.0,
            ctx.get_font_line_height(&common_data.font_main)? + 4.0,
        ));
        cakes_count_label
            .set_text(get_text("bench_cakes_count").replace("%COUNT%", &cakes.len().to_string()));

        ctx.set_fps_limit(0.0);

        Ok(SceneBench {
            common_data,
            cake_textures,
            cakes,
            last_cakes_count_diff: 8,
            rng,
            stage: 1,
            warmup: true,
            frame_count: 0,
            frame_tm: Instant::now(),
            fps: 0.0,
            stage_label,
            cakes_count_label,
        })
    }

    fn update_cakes_count_label(&mut self) {
        self.cakes_count_label.set_text(
            get_text("bench_cakes_count").replace("%COUNT%", &self.cakes.len().to_string()),
        );
    }

    fn process_bench(&mut self) {
        if self.warmup {
            // Пропускаем один кадр после изменения состояния, чтобы не учитывать накладные расходы
            // на это самое изменение состояния
            self.frame_count = 0;
            self.frame_tm = Instant::now();
            self.warmup = false;
            return;
        }

        self.frame_count += 1;
        if self.stage < 4 && self.frame_count >= 8 {
            let frametime = (Instant::now() - self.frame_tm) / self.frame_count;

            if self.stage == 1 || self.stage == 2 {
                // Шаг 1: наваливаем тортиков от души, пока устройство не перестанет справляться
                // Шаг 2: тоже наваливаем, но медленнее
                if frametime <= FRAMETIME_THRESHOLD {
                    let k = if self.stage == 1 { 8 } else { 32 };
                    self.last_cakes_count_diff += self.last_cakes_count_diff / k * 2;
                    for _ in 0..self.last_cakes_count_diff {
                        self.cakes.push((
                            self.rng.gen_range(0..self.cake_textures.len()),
                            Vec2::new(
                                self.rng.gen_range(RANGE_X) as f32,
                                self.rng.gen_range(RANGE_Y) as f32,
                            ),
                        ));
                    }
                } else {
                    // Перестало справляться — отваливаем последнюю партию назад
                    if self.stage == 1 {
                        for _ in 0..self.last_cakes_count_diff {
                            self.cakes.pop();
                        }
                        self.last_cakes_count_diff =
                            std::cmp::max(1, self.last_cakes_count_diff / 32 * 2);
                        self.stage = 2;
                        self.stage_label.set_text(get_text("bench_stage2"));
                    } else {
                        while self.cakes.len() > 50 && self.cakes.len() % 10 != 0 {
                            self.cakes.pop(); // хочу красивое число, да
                        }
                        self.stage = 3;
                        self.stage_label.set_text(get_text("bench_stage3"));
                    }
                }
                self.update_cakes_count_label();
                self.warmup = true;
            } else if self.stage == 3 {
                // Шаг 3: постепенно отваливаем, пока устройство не станет справляться
                let k = if self.cakes.len() > 50000 { 50 } else { 10 };
                if frametime > FRAMETIME_THRESHOLD && !self.cakes.is_empty() {
                    for _ in 0..k {
                        self.cakes.pop();
                    }
                } else {
                    self.stage = 4;
                    self.stage_label.set_text(get_text("bench_stage4"));
                }
                self.update_cakes_count_label();
                self.warmup = true;
            }
        } else if self.stage == 4 && self.frame_count >= 60 {
            // Шаг 4: финальное измерение частоты кадров
            let frametime = (Instant::now() - self.frame_tm) / self.frame_count;
            self.fps = 1.0 / frametime.as_secs_f32();
            self.stage = 5;
        }
    }
}

impl Scene for SceneBench {
    fn process(
        &mut self,
        ctx: &mut dyn Context,
        _dt: f32,
        _events: &[Event],
    ) -> Result<SceneResult> {
        self.common_data.process(ctx)?;

        if ctx.input().is_key_just_pressed(ScanCode::Escape)
            || self.common_data.button_close.just_clicked()
        {
            ctx.set_fps_limit(OPTIONS.lock().unwrap().get_fps_limit());
            let s = SceneMenuMisc::new(self.common_data.clone(), ctx);
            return Ok(SceneResult::Switch(Box::new(s)));
        }

        if self.stage >= 5 {
            ctx.set_fps_limit(OPTIONS.lock().unwrap().get_fps_limit());
            let s =
                SceneBenchResult::new(self.common_data.clone(), ctx, self.cakes.len(), self.fps)?;
            return Ok(SceneResult::Switch(Box::new(s)));
        }

        self.process_bench();

        Ok(SceneResult::Normal)
    }

    fn render(&mut self, ctx: &mut dyn Context) -> Result<()> {
        self.common_data.draw_back(ctx)?;

        for (cake_id, pos) in &self.cakes {
            ctx.draw_texture(&self.cake_textures[*cake_id], *pos, Vec2::new(0.5, 0.5))?;
        }

        self.stage_label.render(ctx)?;
        self.cakes_count_label.render(ctx)?;

        self.common_data.button_close.render(ctx)?;
        // рисовать здесь fps_counter наверное плохая идея?
        self.common_data.draw_cursor(ctx)?;
        Ok(())
    }
}
