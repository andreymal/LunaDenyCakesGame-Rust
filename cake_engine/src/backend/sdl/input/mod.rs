mod scancode;

use crate::{
    input::{Event, KeyMods, MouseButton, TouchPhase},
    vec::Vec2,
    view::View,
};
use sdl2::{keyboard::Mod, EventPump};

impl Into<KeyMods> for Mod {
    fn into(self) -> KeyMods {
        KeyMods {
            shift: self.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD),
            ctrl: self.intersects(Mod::LCTRLMOD | Mod::RCTRLMOD),
            alt: self.intersects(Mod::LALTMOD | Mod::RALTMOD),
            logo: self.intersects(Mod::LGUIMOD | Mod::RGUIMOD),
        }
    }
}

pub(super) fn collect_events(
    event_pump: &mut EventPump,
    events: &mut Vec<Event>,
    view: &View,
    simulate_mouse_with_touch: bool,
) {
    for event in event_pump.poll_iter() {
        match event {
            sdl2::event::Event::Quit { .. } => {
                events.push(Event::Quit);
            }

            sdl2::event::Event::MouseMotion { x, y, .. } => {
                events.push(Event::MouseMove {
                    point: view.point_from_target(Vec2::new(x as f32, y as f32)),
                    touch_id: None,
                });
            }

            sdl2::event::Event::MouseWheel {
                precise_x,
                precise_y,
                ..
            } => {
                events.push(Event::MouseWheel {
                    delta: Vec2::new(precise_x, precise_y),
                });
            }

            sdl2::event::Event::MouseButtonDown {
                mouse_btn, x, y, ..
            } => {
                events.push(Event::MouseDown {
                    button: map_mouse_button(mouse_btn),
                    point: view.point_from_target(Vec2::new(x as f32, y as f32)),
                    touch_id: None,
                });
            }

            sdl2::event::Event::MouseButtonUp {
                mouse_btn, x, y, ..
            } => {
                events.push(Event::MouseUp {
                    button: map_mouse_button(mouse_btn),
                    point: view.point_from_target(Vec2::new(x as f32, y as f32)),
                    touch_id: None,
                });
            }

            sdl2::event::Event::FingerDown {
                finger_id, x, y, ..
            } => {
                let point = view.point_from_target(Vec2::new(x, y));
                events.push(Event::Touch {
                    phase: TouchPhase::Started,
                    id: finger_id as u64,
                    point,
                });

                if simulate_mouse_with_touch {
                    events.push(Event::MouseDown {
                        button: MouseButton::Left,
                        point,
                        touch_id: Some(finger_id as u64),
                    });
                }
            }

            sdl2::event::Event::FingerMotion {
                finger_id, x, y, ..
            } => {
                let point = view.point_from_target(Vec2::new(x, y));
                events.push(Event::Touch {
                    phase: TouchPhase::Moved,
                    id: finger_id as u64,
                    point,
                });

                if simulate_mouse_with_touch {
                    events.push(Event::MouseMove {
                        point,
                        touch_id: Some(finger_id as u64),
                    });
                }
            }

            sdl2::event::Event::FingerUp {
                finger_id, x, y, ..
            } => {
                let point = view.point_from_target(Vec2::new(x, y));
                events.push(Event::Touch {
                    phase: TouchPhase::Ended,
                    id: finger_id as u64,
                    point,
                });

                if simulate_mouse_with_touch {
                    events.push(Event::MouseUp {
                        button: MouseButton::Left,
                        point,
                        touch_id: Some(finger_id as u64),
                    });
                }
            }

            sdl2::event::Event::KeyDown {
                scancode,
                repeat,
                keymod,
                ..
            } => {
                if let Some(s) = scancode {
                    events.push(Event::KeyDown {
                        scancode: s.into(),
                        repeat,
                        mods: keymod.into(),
                    });
                }
            }

            sdl2::event::Event::KeyUp {
                scancode, keymod, ..
            } => {
                if let Some(s) = scancode {
                    events.push(Event::KeyUp {
                        scancode: s.into(),
                        mods: keymod.into(),
                    });
                }
            }

            sdl2::event::Event::TextInput { text, .. } => {
                for character in text.chars() {
                    events.push(Event::Character { character });
                }
            }

            sdl2::event::Event::Window { win_event, .. } => match win_event {
                sdl2::event::WindowEvent::SizeChanged(width, height) => {
                    events.push(Event::Resize {
                        logical_size: Vec2::new(width as f32, height as f32),
                    });
                }
                sdl2::event::WindowEvent::Resized(_, _) => {}
                sdl2::event::WindowEvent::Enter => {
                    events.push(Event::MouseEnter);
                }
                sdl2::event::WindowEvent::Leave => {
                    events.push(Event::MouseLeave);
                }
                sdl2::event::WindowEvent::FocusGained => {
                    events.push(Event::FocusIn);
                }
                sdl2::event::WindowEvent::FocusLost => {
                    events.push(Event::FocusOut);
                }
                sdl2::event::WindowEvent::TakeFocus => {
                    // Не уверен, что оно надо
                }
                sdl2::event::WindowEvent::Minimized => {
                    events.push(Event::Minimized);
                }
                sdl2::event::WindowEvent::Maximized => {
                    events.push(Event::Maximized);
                }
                sdl2::event::WindowEvent::Restored => {
                    events.push(Event::Restored);
                }
                sdl2::event::WindowEvent::Moved(_, _) => {}
                _ => {}
            },

            _ => {}
        }
    }
}

fn map_mouse_button(button: sdl2::mouse::MouseButton) -> MouseButton {
    match button {
        sdl2::mouse::MouseButton::Left => MouseButton::Left,
        sdl2::mouse::MouseButton::Right => MouseButton::Right,
        sdl2::mouse::MouseButton::Middle => MouseButton::Middle,
        sdl2::mouse::MouseButton::X1 => MouseButton::XButton1,
        sdl2::mouse::MouseButton::X2 => MouseButton::XButton2,
        sdl2::mouse::MouseButton::Unknown => MouseButton::Unknown,
    }
}
