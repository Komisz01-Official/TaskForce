#![windows_subsystem = "windows"]

mod app;
mod backend;
mod models;

use app::TaskForceApp;
use backend::hotkeys;
use std::sync::mpsc;

use eframe::{NativeOptions, egui};

fn load_icon() -> std::sync::Arc<egui::IconData> {
    // Load the PNG at compile time
    let bytes = include_bytes!("../assets/icon.png");
    let img = image::load_from_memory(bytes)
        .expect("Failed to load icon PNG")
        .to_rgba8();

    let (w, h) = img.dimensions();

    std::sync::Arc::new(egui::IconData {
        rgba: img.into_raw(),
        width: w,
        height: h,
    })
}

fn main() -> eframe::Result<()> {
    // Hotkey channel
    let (tx, rx) = mpsc::channel::<backend::Command>();
    hotkeys::start_hotkey_thread(tx).expect("failed to start hotkey thread");

    // Load PNG icon at compile time
    let icon_bytes = include_bytes!("../assets/icon.png");
    let img = image::load_from_memory(icon_bytes)
        .expect("Failed to load PNG icon")
        .into_rgba8();

    let (w, h) = img.dimensions();
    let icon = egui::viewport::IconData {
        rgba: img.into_raw(),
        width: w,
        height: h,
    };

    let app = TaskForceApp::new(rx);

    let options = NativeOptions {
    viewport: egui::ViewportBuilder::default()
    .with_inner_size(egui::vec2(760.0, 420.0))
    .with_resizable(true)
    .with_title("TaskForce Macro Recorder (Windows)")
    .with_icon(load_icon()),
    ..Default::default()
    };

    eframe::run_native(
        "TaskForce Macro Recorder",
        options,
        Box::new(|_cc| Ok(Box::new(app))),
    )
}
