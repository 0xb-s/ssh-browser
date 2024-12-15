mod ssh;
mod ui;

use eframe::egui;
use ssh::SSHConnection;
use ui::{render_ui, UIState};

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "SSH File Manager",
        options,
        Box::new(|_cc| Ok(Box::new(App::default()))),
    )
}

#[derive(Default)]
struct App {
    state: UIState,
    connection: Option<SSHConnection>,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            render_ui(ui, &mut self.state, &mut self.connection);
        });
    }
}
