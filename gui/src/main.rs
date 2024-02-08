//! This is (or will be) a graphical frontend for Milton.

// Hide console window on Windows.
#![windows_subsystem = "windows"]

use eframe::{egui, NativeOptions};

fn main() -> Result<(), eframe::Error> {
    eframe::run_native(
        "Milton",
        NativeOptions::default(),
        Box::new(|_| Box::<Milton>::default()),
    )
}

#[derive(Default)]
struct Milton {}

impl eframe::App for Milton {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menubar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("TMS1100", |ui| {
                    let _ = ui.button("Foo");
                    let _ = ui.button("Bar");
                });
            });
        });
    }
}
