// src/backend/hotkeys.rs
// hotkeys.rs - uses RegisterHotKey + message loop on a thread to send simple commands

use std::sync::mpsc::Sender;
use std::thread;

use crate::backend::Command;

// windows types / functions
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    GetMessageW, TranslateMessage, DispatchMessageW, MSG, WM_HOTKEY,
    
};
//deepseek magic

use windows::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, HOT_KEY_MODIFIERS
};

//deepseek magic
/// Start a background thread that registers a few global hotkeys and sends Command messages
/// to the provided channel when they are pressed.
///
/// Hotkeys registered here (example):
///  - Ctrl+R => ToggleRecord
///  - Ctrl+P => TogglePlay
///  - Ctrl+S => Save
///  - Ctrl+L => Load
///  - Ctrl+O => Exit
pub fn start_hotkey_thread(tx: Sender<Command>) -> Result<(), String> {
    thread::spawn(move || unsafe {
        // Hotkey IDs
        const ID_REC: i32 = 1;
        const ID_PLAY: i32 = 2;
        const ID_SAVE: i32 = 3;
        const ID_LOAD: i32 = 4;
        const ID_EXIT: i32 = 5;

        // Use HOT_KEY_MODIFIERS wrapper with numeric flags.
        // 0x0002 is the Win32 MOD_CONTROL flag (Ctrl).
        let ctrl_mod = HOT_KEY_MODIFIERS(0x0002);

        // Register global hotkeys. Virtual-key codes: 'R' as u32 etc.
        // If registration fails, we log but continue (another app might have the hotkey).
        if let Err(e) = RegisterHotKey(HWND(0), ID_REC, ctrl_mod, 'R' as u32) {
            eprintln!("RegisterHotKey Ctrl+R failed: {:?}", e);
        }
        if let Err(e) = RegisterHotKey(HWND(0), ID_PLAY, ctrl_mod, 'P' as u32) {
            eprintln!("RegisterHotKey Ctrl+P failed: {:?}", e);
        }
        if let Err(e) = RegisterHotKey(HWND(0), ID_SAVE, ctrl_mod, 'S' as u32) {
            eprintln!("RegisterHotKey Ctrl+S failed: {:?}", e);
        }
        if let Err(e) = RegisterHotKey(HWND(0), ID_LOAD, ctrl_mod, 'L' as u32) {
            eprintln!("RegisterHotKey Ctrl+L failed: {:?}", e);
        }
        if let Err(e) = RegisterHotKey(HWND(0), ID_EXIT, ctrl_mod, 'O' as u32) {
            eprintln!("RegisterHotKey Ctrl+O failed: {:?}", e);
        }

        // Message loop to receive WM_HOTKEY events.
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, HWND(0), 0, 0).0 != 0 {
            // WM_HOTKEY is a plain constant; compare directly.
            if msg.message == WM_HOTKEY {
                // wParam contains the hotkey id
                let id = msg.wParam.0 as i32;
                match id {
                    ID_REC => { let _ = tx.send(Command::ToggleRecord); }
                    ID_PLAY => { let _ = tx.send(Command::TogglePlay); }
                    ID_SAVE => { let _ = tx.send(Command::Save); }
                    ID_LOAD => { let _ = tx.send(Command::Load); }
                    ID_EXIT => { let _ = tx.send(Command::Exit); }
                    _ => {}
                }
            }

            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        // Message loop ended — unregister hotkeys before thread exit
        let _ = UnregisterHotKey(HWND(0), ID_REC);
        let _ = UnregisterHotKey(HWND(0), ID_PLAY);
        let _ = UnregisterHotKey(HWND(0), ID_SAVE);
        let _ = UnregisterHotKey(HWND(0), ID_LOAD);
        let _ = UnregisterHotKey(HWND(0), ID_EXIT);
    });

    Ok(())
}
