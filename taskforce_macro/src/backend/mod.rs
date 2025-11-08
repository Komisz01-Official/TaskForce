pub mod recorder;
pub mod player;
pub mod storage;
pub mod hotkeys;

pub use recorder::Recorder;
pub use player::Player;
pub use storage::{save_macro_file, load_macro_file};

// commands sent by hotkey thread to the UI
#[derive(Debug, Clone, Copy)]
pub enum Command {
    ToggleRecord,
    TogglePlay,
    Save,
    Load,
    Exit,
}
