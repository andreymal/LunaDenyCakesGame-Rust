//! Информация о внутриигровом времени.
//!
//! Используйте `ctx.time()` для обращения к этому объекту.

use std::time::{Duration, Instant, SystemTime};

/// Информация о внутриигровом времени.
///
/// Используйте `ctx.time()` для обращения к этому объекту.
pub struct GameTime {
    start_date: SystemTime,
    start_time: Instant,
    current_time: Instant,
    current_frame: u64,
    last_fps_frameno: u64,
    last_fps_tick: f32,
    last_fps: u64,
}

impl GameTime {
    /// Создаёт новый экземпляр со всеми нулями по умолчанию и текущим временем в качестве
    /// точки отсчёта.
    pub fn new() -> GameTime {
        let t = Instant::now();
        GameTime {
            start_date: SystemTime::now(),
            start_time: t,
            current_time: t,
            current_frame: 0,
            last_fps_frameno: 0,
            last_fps_tick: 0.0,
            last_fps: 0,
        }
    }

    /// Сбрасывает всё. Момент сброса будет считаться новой точкой отсчёта.
    pub fn reset(&mut self) {
        let t = Instant::now();
        self.start_date = SystemTime::now();
        self.start_time = t;
        self.current_time = t;
        self.current_frame = 0;
        self.last_fps_frameno = 0;
        self.last_fps_tick = 0.0;
        self.last_fps = 0;
    }

    /// Возвращает время, прошедшее с момента запуска игры (или с момента последнего сброса
    /// времени).
    pub fn uptime(&self) -> Duration {
        self.current_time - self.start_time
    }

    /// Возвращает дату запуска игры (или дату последнего сброса времени).
    pub fn get_start_date(&self) -> SystemTime {
        self.start_date
    }

    /// Возвращает значение монотонного времени, которое было в момент запуска игры
    /// (или последнего сброса времени).
    pub fn get_start_time(&self) -> Instant {
        self.start_time
    }

    /// Возвращает текущее значение монотонного времени (на момент вызова метода `tick`).
    pub fn get_current_time(&self) -> Instant {
        self.current_time
    }

    /// Возвращает номер текущего кадра (на момент вызова метода `tick`)
    ///
    /// Поскольку бэкенды первый раз вызывают `tick` только после завершения первого рендеринга,
    /// самый первый кадр имеет значение 0.
    pub fn get_current_frame(&self) -> u64 {
        self.current_frame
    }

    /// Возвращает число кадров, отрисованных за последнюю секунду.
    pub fn get_fps(&self) -> u64 {
        self.last_fps
    }

    /// Тик.
    ///
    /// Выполняет следующие операции:
    ///
    /// * записывает новое текущее значение монотонного времени;
    /// * увеличивает номер текущего кадра на 1;
    /// * раз в секунду пересчитывает частоту кадров;
    /// * возвращает время, прошедшее с прошлого тика.
    ///
    /// Этот метод вызывается бэкендами автоматически, и вам, как правило, не нужно его трогать.
    pub fn tick(&mut self) -> f32 {
        let new_current_time = Instant::now();
        let dt = (new_current_time - self.current_time).as_secs_f32();
        self.current_time = new_current_time;
        self.current_frame = self.current_frame.wrapping_add(1);

        self.last_fps_tick += dt;
        if self.last_fps_tick >= 1.0 {
            self.last_fps = self.current_frame - self.last_fps_frameno;
            self.last_fps_frameno = self.current_frame;
            if dt >= 0.2 {
                self.last_fps_tick = 0.0;
            } else {
                self.last_fps_tick -= 1.0;
            }
        }

        dt
    }
}

/// Ограничитель частоты кадров.
pub struct FPSLimiter {
    frame_duration: Duration,
    frametime: Instant,
}

impl FPSLimiter {
    pub fn new() -> FPSLimiter {
        FPSLimiter {
            frame_duration: Duration::ZERO,
            frametime: Instant::now(),
        }
    }

    /// Установка ограничения частоты кадров, ноль снимает ограничение.
    pub fn set_fps_limit(&mut self, fps: f32) {
        if fps.is_finite() && !fps.is_nan() && fps > 0.0 {
            self.frame_duration = Duration::from_secs_f32(1.0 / fps);
        } else {
            self.frame_duration = Duration::ZERO;
        }
    }

    /// Применение ограничения. Метод усыпит поток на время, требуемое для соблюдения заданного
    /// ограничения частоты кадров. Если ограничение не задано, то ничего не делает.
    pub fn tick(&mut self) {
        if !self.frame_duration.is_zero() {
            self.frametime += self.frame_duration;
            let sleep_time = self.frametime - Instant::now();
            if sleep_time > Duration::ZERO {
                std::thread::sleep(sleep_time);
            } else {
                self.frametime = Instant::now();
            }
        }
    }
}
