// >:(
// https://github.com/not-fl3/macroquad/issues/1

mod scancode;

use crate::{
    input::{Event, MouseButton},
    vec::Vec2,
    view::View,
};
use macroquad::{
    input::KeyCode,
    miniquad::{KeyMods, TouchPhase},
    prelude::{is_quit_requested, screen_height, screen_width},
};

impl Into<crate::input::KeyMods> for KeyMods {
    fn into(self) -> crate::input::KeyMods {
        crate::input::KeyMods {
            shift: self.shift,
            ctrl: self.ctrl,
            alt: self.alt,
            logo: self.logo,
        }
    }
}

pub struct MiniquadEventHandler<'a> {
    events: &'a mut Vec<Event>,
    view: &'a View,
    dpi_scale: f32,
    simulate_mouse_with_touch: bool,
}

impl<'a> macroquad::miniquad::EventHandler for MiniquadEventHandler<'a> {
    fn update(&mut self) {}

    fn draw(&mut self) {}

    fn mouse_motion_event(&mut self, x: f32, y: f32) {
        let point = Vec2::new(x / self.dpi_scale, y / self.dpi_scale);
        self.events.push(Event::MouseMove {
            point: self.view.point_from_target(point),
            touch_id: None,
        });
    }

    fn mouse_wheel_event(&mut self, x: f32, y: f32) {
        self.events.push(Event::MouseWheel {
            delta: Vec2::new(x, y),
        });
    }

    fn mouse_button_down_event(&mut self, button: macroquad::input::MouseButton, x: f32, y: f32) {
        let point = Vec2::new(x / self.dpi_scale, y / self.dpi_scale);
        self.events.push(Event::MouseDown {
            button: map_mouse_button(button),
            point: self.view.point_from_target(point),
            touch_id: None,
        });
    }

    fn mouse_button_up_event(&mut self, button: macroquad::input::MouseButton, x: f32, y: f32) {
        let point = Vec2::new(x / self.dpi_scale, y / self.dpi_scale);
        self.events.push(Event::MouseUp {
            button: map_mouse_button(button),
            point: self.view.point_from_target(point),
            touch_id: None,
        });
    }

    fn touch_event(&mut self, phase: TouchPhase, id: u64, x: f32, y: f32) {
        let point = self
            .view
            .point_from_target(Vec2::new(x / self.dpi_scale, y / self.dpi_scale));

        self.events.push(Event::Touch {
            phase: match phase {
                TouchPhase::Started => crate::input::TouchPhase::Started,
                TouchPhase::Moved => crate::input::TouchPhase::Moved,
                TouchPhase::Ended => crate::input::TouchPhase::Ended,
                TouchPhase::Cancelled => crate::input::TouchPhase::Cancelled,
            },
            id,
            point,
        });

        if self.simulate_mouse_with_touch {
            let button = MouseButton::Left;
            self.events.push(match phase {
                TouchPhase::Started => Event::MouseDown {
                    button,
                    point,
                    touch_id: Some(id),
                },
                TouchPhase::Moved => Event::MouseMove {
                    point,
                    touch_id: Some(id),
                },
                _ => Event::MouseUp {
                    button,
                    point,
                    touch_id: Some(id),
                },
            });
        }
    }

    // keycode в macroquad это на самом деле scancode, то есть физический код клавиши,
    // не зависящий от текущей раскладки
    fn key_down_event(&mut self, keycode: KeyCode, keymods: KeyMods, repeat: bool) {
        self.events.push(Event::KeyDown {
            scancode: keycode.into(),
            repeat,
            mods: keymods.into(),
        });
    }

    fn key_up_event(&mut self, keycode: KeyCode, keymods: KeyMods) {
        self.events.push(Event::KeyUp {
            scancode: keycode.into(),
            mods: keymods.into(),
        });
    }

    fn char_event(&mut self, character: char, _keymods: KeyMods, _repeat: bool) {
        self.events.push(Event::Character { character });
    }

    fn resize_event(&mut self, width: f32, height: f32) {
        // Не работает, потому что macroquad не пробрасывает это событие...
        self.events.push(Event::Resize {
            logical_size: Vec2::new(width, height),
        });
    }

    fn window_minimized_event(&mut self) {
        // Работает только на Android
        // (но macroquad не пробрасывает...)
        self.events.push(Event::FocusOut);
        self.events.push(Event::Minimized);
    }

    fn window_restored_event(&mut self) {
        // Тоже Android (и тоже не пробрасывает)
        self.events.push(Event::Restored);
        self.events.push(Event::FocusIn);
    }
}

fn map_mouse_button(button: macroquad::input::MouseButton) -> MouseButton {
    match button {
        macroquad::input::MouseButton::Left => MouseButton::Left,
        macroquad::input::MouseButton::Right => MouseButton::Right,
        macroquad::input::MouseButton::Middle => MouseButton::Middle,
        macroquad::input::MouseButton::Unknown => MouseButton::Unknown,
    }
}

pub(super) fn collect_events(
    event_subscriber_id: usize,
    events: &mut Vec<Event>,
    view: &mut View,
    simulate_mouse_with_touch: bool,
) {
    if is_quit_requested() {
        events.push(Event::Quit);
    }

    // macroquad не пробрасывает resize_event из miniquad, поэтому приходится костылять
    let logical_size = Vec2::new(screen_width(), screen_height());
    if view.set_target_size(logical_size) {
        events.push(Event::Resize { logical_size });
    }

    let mut h = MiniquadEventHandler {
        events,
        view,
        dpi_scale: macroquad::miniquad::window::dpi_scale(),
        simulate_mouse_with_touch,
    };
    macroquad::input::utils::repeat_all_miniquad_input(&mut h, event_subscriber_id);
}
