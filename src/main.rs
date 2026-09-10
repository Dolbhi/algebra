use eframe::egui;
use egui::Id;

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
    // show_confirmation_dialog: bool,
    // allowed_to_close: bool,
    text_box_text: String,
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::bottom(Id::new("bottom_1")).show(ui, |ui| {
            ui.heading("Bottom Panel");
            ui.text_edit_singleline(&mut self.text_box_text);
        });

        egui::Panel::bottom(Id::new("bottom_2")).show(ui, |ui| {
            ui.heading("Bottom Panel 2");
            ui.label(format!("The box says: {}", self.text_box_text));
        });

        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Central Panel");
            ui.vertical_centered(|ui| {
                let mut frame = egui::Frame::default().inner_margin(4.0).begin(ui);
                {
                    let response = frame.content_ui.label("Inside the frame 1");
                    if response.hovered() {
                        frame.frame.fill = egui::Color32::RED;
                    }
                }
                frame.end(ui); // Will "close" the frame.
                let mut frame = egui::Frame::default().inner_margin(4.0).begin(ui);
                {
                    let response = frame.content_ui.label("Inside the frame 2");
                    if response.hovered() {
                        frame.frame.fill = egui::Color32::GREEN;
                    }
                }
                frame.end(ui); // Will "close" the frame.
            });
        });

        

        // if ui.input(|i| i.viewport().close_requested()) {
        //     if self.allowed_to_close {
        //         // do nothing - we will close
        //     } else {
        //         ui.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        //         self.show_confirmation_dialog = true;
        //     }
        // }

        // if self.show_confirmation_dialog {
        //     egui::Window::new("Do you want to quit?")
        //         .collapsible(false)
        //         .resizable(false)
        //         .show(ui.ctx(), |ui| {
        //             ui.horizontal(|ui| {
        //                 if ui.button("No").clicked() {
        //                     self.show_confirmation_dialog = false;
        //                     self.allowed_to_close = false;
        //                 }

        //                 if ui.button("Yes").clicked() {
        //                     self.show_confirmation_dialog = false;
        //                     self.allowed_to_close = true;
        //                     ui.send_viewport_cmd(egui::ViewportCommand::Close);
        //                 }
        //             });
        //         });
        // }
    }
}
