#![allow(dead_code)]
#![allow(unused_imports)]

mod app;
mod editor;
mod languages;
mod ai;
mod low_level;
mod ui;

use app::PhobosApp;
use eframe::egui;

fn main() -> eframe::Result {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("💀 Phobos IDE — The God of Fear"),
        ..Default::default()
    };

    eframe::run_native(
        "Phobos IDE",
        options,
        Box::new(|_cc| Ok(Box::new(PhobosApp::new()))),
    )
}
