use crate::{
    input::{Event, Input, KeyMods, MouseButton, TouchPhase},
    vec::Vec2,
    view::View,
};
use sfml::graphics::RenderWindow;

mod scancode;

pub(super) fn collect_events(
    window: &mut RenderWindow,
    events: &mut Vec<Event>,
    input: &Input,
    view: &View,
    simulate_mouse_with_touch: bool,
) {
    while let Some(event) = window.poll_event() {
        match event {
            sfml::window::Event::Closed => {
                events.push(Event::Quit);
            }

            sfml::window::Event::MouseEntered => {
                events.push(Event::MouseEnter);
            }

            sfml::window::Event::MouseLeft => {
                events.push(Event::MouseLeave);
            }

            sfml::window::Event::MouseMoved { x, y } => {
                events.push(Event::MouseMove {
                    point: view.point_from_target(Vec2::new(x as f32, y as f32)),
                    touch_id: None,
                });
            }

            sfml::window::Event::MouseWheelScrolled { wheel, delta, .. } => {
                let (dx, dy) = match wheel {
                    sfml::window::mouse::Wheel::VerticalWheel => (0.0, delta),
                    sfml::window::mouse::Wheel::HorizontalWheel => (delta, 0.0),
                };
                events.push(Event::MouseWheel {
                    delta: Vec2::new(dx, dy),
                });
            }

            sfml::window::Event::MouseButtonPressed { button, x, y } => {
                let b = match button {
                    sfml::window::mouse::Button::Left => MouseButton::Left,
                    sfml::window::mouse::Button::Right => MouseButton::Right,
                    sfml::window::mouse::Button::Middle => MouseButton::Middle,
                    sfml::window::mouse::Button::XButton1 => MouseButton::XButton1,
                    sfml::window::mouse::Button::XButton2 => MouseButton::XButton2,
                };
                events.push(Event::MouseDown {
                    button: b,
                    point: view.point_from_target(Vec2::new(x as f32, y as f32)),
                    touch_id: None,
                });
            }

            sfml::window::Event::MouseButtonReleased { button, x, y } => {
                let b = match button {
                    sfml::window::mouse::Button::Left => MouseButton::Left,
                    sfml::window::mouse::Button::Right => MouseButton::Right,
                    sfml::window::mouse::Button::Middle => MouseButton::Middle,
                    sfml::window::mouse::Button::XButton1 => MouseButton::XButton1,
                    sfml::window::mouse::Button::XButton2 => MouseButton::XButton2,
                };
                events.push(Event::MouseUp {
                    button: b,
                    point: view.point_from_target(Vec2::new(x as f32, y as f32)),
                    touch_id: None,
                });
            }

            sfml::window::Event::TouchBegan { finger, x, y } => {
                let point = view.point_from_target(Vec2::new(x as f32, y as f32));
                events.push(Event::Touch {
                    phase: TouchPhase::Started,
                    id: finger as u64,
                    point,
                });

                if simulate_mouse_with_touch {
                    events.push(Event::MouseDown {
                        button: MouseButton::Left,
                        point,
                        touch_id: Some(finger as u64),
                    });
                }
            }

            sfml::window::Event::TouchMoved { finger, x, y } => {
                let point = view.point_from_target(Vec2::new(x as f32, y as f32));
                events.push(Event::Touch {
                    phase: TouchPhase::Moved,
                    id: finger as u64,
                    point,
                });

                if simulate_mouse_with_touch {
                    events.push(Event::MouseMove {
                        point,
                        touch_id: Some(finger as u64),
                    });
                }
            }

            sfml::window::Event::TouchEnded { finger, x, y } => {
                let point = view.point_from_target(Vec2::new(x as f32, y as f32));
                events.push(Event::Touch {
                    phase: TouchPhase::Ended,
                    id: finger as u64,
                    point,
                });

                if simulate_mouse_with_touch {
                    events.push(Event::MouseUp {
                        button: MouseButton::Left,
                        point,
                        touch_id: Some(finger as u64),
                    });
                }
            }

            sfml::window::Event::KeyPressed {
                code,
                alt,
                ctrl,
                shift,
                system,
                ..
            } => {
                let scancode = code.into();
                events.push(Event::KeyDown {
                    scancode,
                    repeat: input.is_key_pressed(scancode),
                    mods: KeyMods {
                        shift,
                        ctrl,
                        alt,
                        logo: system,
                    },
                });
            }

            sfml::window::Event::KeyReleased {
                code,
                alt,
                ctrl,
                shift,
                system,
                ..
            } => {
                events.push(Event::KeyUp {
                    scancode: code.into(),
                    mods: KeyMods {
                        shift,
                        ctrl,
                        alt,
                        logo: system,
                    },
                });
            }

            sfml::window::Event::TextEntered { unicode, .. } => {
                events.push(Event::Character { character: unicode });
            }

            sfml::window::Event::Resized { width, height } => {
                events.push(Event::Resize {
                    logical_size: Vec2::new(width as f32, height as f32),
                });
            }

            sfml::window::Event::GainedFocus => {
                events.push(Event::FocusIn);
            }

            sfml::window::Event::LostFocus => {
                events.push(Event::FocusOut);
            }

            _ => {}
        }
    }
}
