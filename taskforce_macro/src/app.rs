use crate::models::MacroEvent;
use crate::backend::{Recorder, Player, storage};
use crate::backend;
use eframe::egui;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Receiver;

use std::time::Duration;

pub struct TaskForceApp {
    events: Arc<Mutex<Vec<MacroEvent>>>,
    recorder: Recorder,
    player: Player,
    // state
    recording: bool,
    playing: bool,
    status: String,
    // playback controls
    play_count: u32,
    play_speed: f32,
    infinite_loop: bool,
    // hotkey rx
    rx: Receiver<backend::Command>,
}

impl TaskForceApp {
    pub fn new(rx: Receiver<backend::Command>) -> Self {
        let events = Arc::new(Mutex::new(Vec::new()));
        let rec = Recorder::new(Arc::clone(&events));
        let player = Player::new();
        Self {
            events,
            recorder: rec,
            player,
            recording: false,
            playing: false,
            status: "Ready.".into(),
            play_count: 1,
            play_speed: 1.0,
            infinite_loop: false,
            rx,
        }
    }

    fn toggle_record(&mut self) {
        if self.recording {
            self.recorder.stop();
            self.recording = false;
            self.status = "🛑 Recording stopped".into();
        } else {
            self.recorder.start();
            self.recording = true;
            self.status = "⏺ Recording...".into();
        }
    }

    fn toggle_play(&mut self) {
        if self.playing {
            self.player.stop();
            self.playing = false;
            self.status = "🛑 Playback stopped".into();
        } else {
            let events = { self.events.lock().unwrap().clone() };
            if events.is_empty() {
                self.status = "❌ Nothing recorded".into();
                return;
            }
            self.player.play(events, self.play_count, self.play_speed, self.infinite_loop);
            self.playing = true;
            self.status = format!("▶ Playing ({}x, {} times{})", self.play_speed, self.play_count, if self.infinite_loop { " infinite" } else { "" });
        }
    }

    fn save(&mut self) {
        let ev = { self.events.lock().unwrap().clone() };
        match storage::save_macro_file("macro_recording.json", &ev) {
            Ok(_) => self.status = "💾 Saved macro_recording.json".into(),
            Err(e) => self.status = format!("❌ Save failed: {}", e),
        }
    }

    fn load(&mut self) {
        match storage::load_macro_file("macro_recording.json") {
            Ok(vec) => {
                let mut guard = self.events.lock().unwrap();
                *guard = vec;
                self.status = "📂 Loaded macro_recording.json".into();
            }
            Err(e) => {
                self.status = format!("❌ Load failed: {}", e);
            }
        }
    }
}

impl eframe::App for TaskForceApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // consume hotkey commands
        while let Ok(cmd) = self.rx.try_recv() {
            match cmd {
                backend::Command::ToggleRecord => self.toggle_record(),
                backend::Command::TogglePlay => self.toggle_play(),
                backend::Command::Save => self.save(),
                backend::Command::Load => self.load(),
                backend::Command::Exit => std::process::exit(0),
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🎯 Komisz_01's TaskForce Macro Recorder");
            ui.label(&self.status);
            ui.separator();

            ui.horizontal(|ui| {
                if ui.button(if self.recording { "⏹ Stop Recording (Ctrl+R)" } else { "⏺ Start Recording (Ctrl+R)" }).clicked() {
                    self.toggle_record();
                }
                if ui.button(if self.playing { "⏹ Stop Playback (Ctrl+P)" } else { "▶ Start Playback (Ctrl+P)" }).clicked() {
                    self.toggle_play();
                }
            });

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                if ui.button("💾 Save (Ctrl+S)").clicked() { self.save(); }
                if ui.button("📂 Load (Ctrl+L)").clicked() { self.load(); }
            });

            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Reps:");
                ui.add(egui::DragValue::new(&mut self.play_count).range(1..=9999));
                ui.add_space(6.0);
                ui.label("Speed:");
                ui.add(egui::Slider::new(&mut self.play_speed, 0.1..=5.0).suffix("×"));
                ui.add_space(6.0);
                ui.checkbox(&mut self.infinite_loop, "♾ Infinite");
            });

            ui.separator();

            // show a short list preview of events (first 20)
            ui.label("Recorded events (preview):");
            let guard = self.events.lock().unwrap();
            egui::ScrollArea::vertical().max_height(160.0).show(ui, |ui| {
                for (i, e) in guard.iter().enumerate().take(200) {
                    ui.label(format!("{}: {:?} ({} ms)", i, e.ev, e.delay));
                }
            });
        });

        ctx.request_repaint_after(Duration::from_millis(100));
    }
}
