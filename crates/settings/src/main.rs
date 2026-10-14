#![cfg_attr(windows, windows_subsystem = "windows")]

mod app;
mod form;
mod register;

use eframe::egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Broxer Settings")
            .with_inner_size([1040.0, 640.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Broxer Settings",
        options,
        Box::new(|_cc| Ok(Box::new(app::SettingsApp::new(broxer_platform::paths::config_path())))),
    )
}
