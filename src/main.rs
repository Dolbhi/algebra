use eframe::egui;
use egui::{Color32, Id, Layout};

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_resizable(true),
        ..Default::default()
    };
    eframe::run_native(
        "Math Manipulator",
        options,
        Box::new(|_cc| Ok(Box::<MyApp>::default())),
    )
}

#[derive(Default)]
struct MyApp {
    text_box_text: String,
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::right(Id::new("right")).exact_size(200.0).show(ui, |ui| {
            ui.heading("Side panel");
        });

        egui::Panel::bottom(Id::new("bottom")).exact_size(120.0).show(ui, |ui| {
            ui.heading("Input");
            ui.with_layout(Layout::top_down_justified(egui::Align::Center), |ui| {
                egui::TextEdit::multiline(&mut self.text_box_text).hint_text("Math here").show(ui);
            });
        });

        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Output");
            ui.label(format!("The box says: {}", self.text_box_text));
        });

        // egui::CentralPanel::default().show(ui, |ui| {
        //     ui.heading("Central Panel");
        //     ui.vertical_centered(|ui| {
        //         let mut frame = egui::Frame::default().inner_margin(4.0).begin(ui);
        //         {
        //             let response = frame.content_ui.label("Inside the frame 1");
        //             if response.hovered() {
        //                 frame.frame.fill = egui::Color32::RED;
        //             }
        //         }
        //         frame.end(ui); // Will "close" the frame.
        //         let mut frame = egui::Frame::default().inner_margin(4.0).begin(ui);
        //         {
        //             let response = frame.content_ui.label("Inside the frame 2");
        //             if response.hovered() {
        //                 frame.frame.fill = egui::Color32::GREEN;
        //             }
        //         }
        //         frame.end(ui); // Will "close" the frame.
        //     });
        // });
    }
}
