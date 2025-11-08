use crate::models::MacroEvent;
use std::fs;
use std::path::Path;

pub fn save_macro_file(path: impl AsRef<Path>, events: &Vec<MacroEvent>) -> Result<(), String> {
    match serde_json::to_string_pretty(events) {
        Ok(json) => fs::write(path, json).map_err(|e| format!("io error: {}", e)),
        Err(e) => Err(format!("serialize error: {}", e)),
    }
}

pub fn load_macro_file(path: impl AsRef<Path>) -> Result<Vec<MacroEvent>, String> {
    let s = fs::read_to_string(path).map_err(|e| format!("io error: {}", e))?;
    serde_json::from_str(&s).map_err(|e| format!("parse error: {}", e))
}
