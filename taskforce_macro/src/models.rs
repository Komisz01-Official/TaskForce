use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Unknown,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum MacroEventType {
    MouseMove { x: i32, y: i32 },
    MouseDown { button: MouseButton },
    MouseUp { button: MouseButton },
    KeyDown { vk: u32 },
    KeyUp { vk: u32 },
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct MacroEvent {
    pub ev: MacroEventType,
    pub delay: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MouseMode {
    Absolute,
    Relative,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackSettings {
    pub mouse_mode: MouseMode,
    pub speed: f32,
    pub repeat_count: u32,
    pub infinite: bool,
}

impl Default for PlaybackSettings {
    fn default() -> Self {
        Self {
            mouse_mode: MouseMode::Absolute,
            speed: 1.0,
            repeat_count: 1,
            infinite: false,
        }
    }
}
