use crate::models::{MacroEvent, MacroEventType, MouseButton};
use std::sync::{
    Arc,
    Mutex,
    atomic::{AtomicBool, Ordering}
};
use std::thread;
use std::time::Instant;
use std::sync::atomic::AtomicU32;

use windows::Win32::Foundation::*;
use windows::Win32::System::Threading::*;
use windows::Win32::System::LibraryLoader::*;

use windows::Win32::UI::WindowsAndMessaging::*;

pub struct Recorder {
    events: Arc<Mutex<Vec<MacroEvent>>>,
    stop_flag: Arc<AtomicBool>,
    thread_id: Arc<AtomicU32>,
}

impl Recorder {
    pub fn new(events: Arc<Mutex<Vec<MacroEvent>>>) -> Self {
        Self {
            events,
            stop_flag: Arc::new(AtomicBool::new(false)),
            thread_id: Arc::new(AtomicU32::new(0)),
        }
    }

    pub fn start(&self) {
        // Don't double start
        self.stop_flag.store(false, Ordering::SeqCst);

        let events = Arc::clone(&self.events);
        let stop_flag = Arc::clone(&self.stop_flag);
        let tid_store = Arc::clone(&self.thread_id);

        thread::spawn(move || unsafe {
            // Shared state for hook procedures
            static mut DISP_PTR: *mut Dispatcher = std::ptr::null_mut();

            let dispatcher = Box::new(Dispatcher {
                events,
                stop_flag,
                last_time: Instant::now(),
            });

            DISP_PTR = Box::into_raw(dispatcher);

            extern "system" fn kb_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
                unsafe {
                    if code >= HC_ACTION as i32{
                        let disp = &mut *DISP_PTR;
                        if !disp.stop_flag.load(Ordering::SeqCst) {
                            let kb = &*(lparam.0 as *const KBDLLHOOKSTRUCT);

                            let vk = kb.vkCode;
                            let now = Instant::now();
                            let delay = now.duration_since(disp.last_time).as_millis() as u64;
                            disp.last_time = now;

                            let ev = match wparam.0 as u32 {
                                WM_KEYDOWN => MacroEvent {
                                    ev: MacroEventType::KeyDown { vk },
                                    delay,
                                },
                                WM_KEYUP => MacroEvent {
                                    ev: MacroEventType::KeyUp { vk },
                                    delay,
                                },
                                _ => return CallNextHookEx(None, code, wparam, lparam),
                            };

                            if let Ok(mut guard) = disp.events.lock() {
                                guard.push(ev);
                            }
                        }
                    }
                    CallNextHookEx(None, code, wparam, lparam)
                }
            }

            extern "system" fn ms_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
                unsafe {
                    if code >= HC_ACTION as i32{
                        let disp = &mut *DISP_PTR;
                        if !disp.stop_flag.load(Ordering::SeqCst) {
                            let ms = &*(lparam.0 as *const MSLLHOOKSTRUCT);

                            let now = Instant::now();
                            let delay = now.duration_since(disp.last_time).as_millis() as u64;
                            disp.last_time = now;

                            let event = match wparam.0 as u32 {
                                WM_MOUSEMOVE => MacroEvent {
                                    ev: MacroEventType::MouseMove {
                                        x: ms.pt.x,
                                        y: ms.pt.y,
                                    },
                                    delay,
                                },
                                WM_LBUTTONDOWN => MacroEvent {
                                    ev: MacroEventType::MouseDown {
                                        button: MouseButton::Left,
                                    },
                                    delay,
                                },
                                WM_LBUTTONUP => MacroEvent {
                                    ev: MacroEventType::MouseUp {
                                        button: MouseButton::Left,
                                    },
                                    delay,
                                },
                                WM_RBUTTONDOWN => MacroEvent {
                                    ev: MacroEventType::MouseDown {
                                        button: MouseButton::Right,
                                    },
                                    delay,
                                },
                                WM_RBUTTONUP => MacroEvent {
                                    ev: MacroEventType::MouseUp {
                                        button: MouseButton::Right,
                                    },
                                    delay,
                                },
                                WM_MBUTTONDOWN => MacroEvent {
                                    ev: MacroEventType::MouseDown {
                                        button: MouseButton::Middle,
                                    },
                                    delay,
                                },
                                WM_MBUTTONUP => MacroEvent {
                                    ev: MacroEventType::MouseUp {
                                        button: MouseButton::Middle,
                                    },
                                    delay,
                                },
                                _ => return CallNextHookEx(None, code, wparam, lparam),
                            };

                            if let Ok(mut guard) = disp.events.lock() {
                                guard.push(event);
                            }
                        }
                    }
                    CallNextHookEx(None, code, wparam, lparam)
                }
            }

            // Install hooks
            let hmod = GetModuleHandleW(None).unwrap_or_default();
            let kb_hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(kb_proc), hmod, 0)
            .expect("Failed to install keyboard hook");

            let ms_hook = SetWindowsHookExW(WH_MOUSE_LL, Some(ms_proc), hmod, 0)
            .expect("Failed to install mouse hook");

            // Store thread ID
            tid_store.store(GetCurrentThreadId(), Ordering::SeqCst);

            // Message loop
            let mut msg = MSG::default();
            while GetMessageW(&mut msg, HWND(0), 0, 0).0 != 0 {
                if (*DISP_PTR).stop_flag.load(Ordering::SeqCst) {
                    break;
                }
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            // Unhook
            UnhookWindowsHookEx(kb_hook).ok();
            UnhookWindowsHookEx(ms_hook).ok();

            // Cleanup dispatcher
            if !DISP_PTR.is_null() {
                drop(Box::from_raw(DISP_PTR));
                DISP_PTR = std::ptr::null_mut();
            }
        });
    }

    pub fn stop(&self) {
        self.stop_flag.store(true, Ordering::SeqCst);

        let tid = self.thread_id.load(Ordering::SeqCst);
        if tid != 0 {
            unsafe {
                PostThreadMessageW(tid, WM_QUIT, WPARAM(0), LPARAM(0));
            }
        }
    }
}

// Internal shared struct used by hook procs
struct Dispatcher {
    events: Arc<Mutex<Vec<MacroEvent>>>,
    stop_flag: Arc<AtomicBool>,
    last_time: Instant,
}
