use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::Duration;

use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};
use windows::Win32::Foundation::POINT;
use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

use crate::models::{MacroEvent, MacroEventType, MouseButton, MouseMode};

pub struct Player {
    stop_flag: Arc<AtomicBool>,
    is_playing: Arc<AtomicBool>,
}

impl Player {
    pub fn new() -> Self {
        Self {
            stop_flag: Arc::new(AtomicBool::new(false)),
            is_playing: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn play(
        &mut self,
        events: Vec<MacroEvent>,
        repeat_count: u32,
        speed: f32,
        infinite: bool,
        mouse_mode: MouseMode,
    ) {
        if events.is_empty() {
            return;
        }

        // reset stop flag
        self.stop_flag.store(false, Ordering::SeqCst);
        self.is_playing.store(true, Ordering::SeqCst);

        let stop_flag = self.stop_flag.clone();
        let playing_flag = self.is_playing.clone();

        thread::spawn(move || {
            let speed_factor = speed.max(0.05);

            let mut loop_index = 0;

            // Get screen dimensions once
            let screen_width = unsafe { GetSystemMetrics(SM_CXSCREEN) } as i32;
            let screen_height = unsafe { GetSystemMetrics(SM_CYSCREEN) } as i32;

            // For relative mode: track the starting position to convert absolute coords
            let mut relative_start_pos: Option<(i32, i32)> = None;

            loop {
                if stop_flag.load(Ordering::SeqCst) {
                    break;
                }

                for ev in &events {
                    if stop_flag.load(Ordering::SeqCst) {
                        break;
                    }

                    let adjusted_delay = (ev.delay as f32 / speed_factor) as u64;
                    thread::sleep(Duration::from_millis(adjusted_delay));

                    unsafe {
                        match &ev.ev {
                            MacroEventType::MouseMove { x, y } => {
                                match mouse_mode {
                                    MouseMode::Absolute => {
                                        // Existing absolute movement code
                                        let abs_x = (*x * 65535) / screen_width;
                                        let abs_y = (*y * 65535) / screen_height;
                                        let input = INPUT {
                                            r#type: INPUT_MOUSE,
                                            Anonymous: INPUT_0 {
                                                mi: MOUSEINPUT {
                                                    dx: abs_x,
                                                    dy: abs_y,
                                                    mouseData: 0,
                                                    dwFlags: MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE,
                                                    time: 0,
                                                    dwExtraInfo: 0,
                                                },
                                            },
                                        };
                                        SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
                                    }
                                    MouseMode::Relative => {
                                        // Check if we need to detect if these are absolute coordinates
                                        // If coordinates are large (screen-sized), they're probably absolute
                                        let (rel_x, rel_y) = if *x > 1000 || *y > 1000 {
                                            // These are likely absolute coordinates - convert to relative
                                            if relative_start_pos.is_none() {
                                                // Get current mouse position as starting point
                                                let mut point = POINT { x: 0, y: 0 };
                                                if GetCursorPos(&mut point).is_ok() {  // FIXED: use is_ok() instead of as_bool()
                                                    relative_start_pos = Some((point.x, point.y));
                                                }
                                            }
                                            
                                            if let Some((start_x, start_y)) = relative_start_pos {
                                                // Convert absolute to relative from starting position
                                                let dx = *x - start_x;
                                                let dy = *y - start_y;
                                                (dx, dy)
                                            } else {
                                                (*x, *y) // Fallback
                                            }
                                        } else {
                                            // These are already relative coordinates
                                            (*x, *y)
                                        };

                                        let input = INPUT {
                                            r#type: INPUT_MOUSE,
                                            Anonymous: INPUT_0 {
                                                mi: MOUSEINPUT {
                                                    dx: rel_x,
                                                    dy: rel_y,
                                                    mouseData: 0,
                                                    dwFlags: MOUSEEVENTF_MOVE,  // No ABSOLUTE flag
                                                    time: 0,
                                                    dwExtraInfo: 0,
                                                },
                                            },
                                        };
                                        SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
                                    }
                                }
                            }
                            
                            MacroEventType::MouseDown { button } => {
                                let flag = match button {
                                    MouseButton::Left => MOUSEEVENTF_LEFTDOWN,
                                    MouseButton::Right => MOUSEEVENTF_RIGHTDOWN,
                                    MouseButton::Middle => MOUSEEVENTF_MIDDLEDOWN,
                                    _ => continue,
                                };
                                let input = INPUT {
                                    r#type: INPUT_MOUSE,
                                    Anonymous: INPUT_0 {
                                        mi: MOUSEINPUT {
                                            dx: 0,
                                            dy: 0,
                                            mouseData: 0,
                                            dwFlags: flag,
                                            time: 0,
                                            dwExtraInfo: 0,
                                        },
                                    },
                                };
                                SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
                            }

                            MacroEventType::MouseUp { button } => {
                                let flag = match button {
                                    MouseButton::Left => MOUSEEVENTF_LEFTUP,
                                    MouseButton::Right => MOUSEEVENTF_RIGHTUP,
                                    MouseButton::Middle => MOUSEEVENTF_MIDDLEUP,
                                    _ => continue,
                                };
                                let input = INPUT {
                                    r#type: INPUT_MOUSE,
                                    Anonymous: INPUT_0 {
                                        mi: MOUSEINPUT {
                                            dx: 0,
                                            dy: 0,
                                            mouseData: 0,
                                            dwFlags: flag,
                                            time: 0,
                                            dwExtraInfo: 0,
                                        },
                                    },
                                };
                                SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
                            }

                            MacroEventType::KeyDown { vk } => {
                                let input = INPUT {
                                    r#type: INPUT_KEYBOARD,
                                    Anonymous: INPUT_0 {
                                        ki: KEYBDINPUT {
                                            wVk: VIRTUAL_KEY(*vk as u16),
                                            wScan: 0,
                                            dwFlags: KEYBD_EVENT_FLAGS(0),
                                            time: 0,
                                            dwExtraInfo: 0,
                                        },
                                    },
                                };
                                SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
                            }

                            MacroEventType::KeyUp { vk } => {
                                let input = INPUT {
                                    r#type: INPUT_KEYBOARD,
                                    Anonymous: INPUT_0 {
                                        ki: KEYBDINPUT {
                                            wVk: VIRTUAL_KEY(*vk as u16),
                                            wScan: 0,
                                            dwFlags: KEYEVENTF_KEYUP,
                                            time: 0,
                                            dwExtraInfo: 0,
                                        },
                                    },
                                };
                                SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
                            }
                        }
                    }
                }

                if !infinite {
                    loop_index += 1;
                    if loop_index >= repeat_count {
                        break;
                    }
                }
            }

            playing_flag.store(false, Ordering::SeqCst);
        });
    }

    pub fn stop(&mut self) {
        self.stop_flag.store(true, Ordering::SeqCst);
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing.load(Ordering::SeqCst)
    }
}
