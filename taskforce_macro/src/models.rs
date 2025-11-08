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
