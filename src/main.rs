use eframe::egui;
use egui::{Color32, Id, Layout};
use algebra::syntax;

fn main() -> eframe::Result {
    // let res = syntax::SyntaxTree::parse(" a 2 ee aw (2 2) * (aa) + 2");
    // println!("Parse result: {:?}", res);

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
    variables: Vec<String>,
    vars_editing: bool,
    edit_focused: bool,
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::right(Id::new("right"))
            .resizable(false)
            .exact_size(200.0)
            .show(ui, |ui| {
                ui.heading("Side panel");
                ui.checkbox(&mut self.vars_editing, "Edit Vars");
                ui.with_layout(
                    Layout::left_to_right(egui::Align::Min).with_main_wrap(true),
                    |ui| {
                        if self.vars_editing {
                            let font_id = egui::FontId::default();
                            for var in self.variables.iter_mut() {
                                let gallery = ui.fonts_mut(|f| {
                                    f.layout_no_wrap(
                                        var.clone(),
                                        font_id.clone(),
                                        egui::Color32::BLACK,
                                    )
                                });
                                egui::TextEdit::singleline(var)
                                    .desired_width(gallery.size().length() + 1.)
                                    .show(ui);
                            }
                        } else {
                            for var in self.variables.iter() {
                                if ui.button(var).clicked() {
                                    self.text_box_text += var;
                                }
                            }
                        }
                    },
                );
            });

        let mut math_highlighter =
            |ui: &egui::Ui, string: &dyn egui::TextBuffer, wrap_width: f32| {
                let mut job = egui::text::LayoutJob::default();
                job.wrap.max_width = wrap_width;
                let mut words = string.as_str().split(|c| c == 'a');
                job.append(words.next().unwrap(), 0., egui::TextFormat::default());
                for word in words {
                    job.append(
                        "a",
                        0.,
                        egui::TextFormat {
                            color: Color32::RED,
                            ..egui::TextFormat::default()
                        },
                    );
                    job.append(word, 0., egui::TextFormat::default());
                }

                ui.fonts_mut(|f| f.layout_job(job))
            };
        let mut math_highlighter_unfocused =
            |ui: &egui::Ui, string: &dyn egui::TextBuffer, wrap_width: f32| {
                let mut job = egui::text::LayoutJob::default();
                job.wrap.max_width = wrap_width;
                let mut words = string.as_str().split(|c| c == 'a');
                job.append(words.next().unwrap(), 0., egui::TextFormat::default());
                for word in words {
                    job.append(
                        "a",
                        0.,
                        egui::TextFormat {
                            color: Color32::RED,
                            ..egui::TextFormat::default()
                        },
                    );
                    job.append("(not a)", 0., egui::TextFormat::default());
                }

                ui.fonts_mut(|f| f.layout_job(job))
            };
        egui::Panel::bottom(Id::new("bottom"))
            .exact_size(120.0)
            .show(ui, |ui| {
                ui.heading("Input");
                if ui.button("Add as variable").clicked() {
                    self.variables.push(self.text_box_text.clone());
                }
                ui.with_layout(Layout::top_down_justified(egui::Align::Center), |ui| {
                    let text_box =
                        egui::TextEdit::multiline(&mut self.text_box_text).hint_text("Math here");
                    let response = if self.edit_focused {
                        text_box.layouter(&mut math_highlighter).show(ui).response
                    } else {
                        text_box
                            .layouter(&mut math_highlighter_unfocused)
                            .show(ui)
                            .response
                    };
                    self.edit_focused = response.has_focus();
                });
            });

        egui::CentralPanel::default().frame(egui::Frame::default().fill(Color32::LIGHT_GRAY).inner_margin(egui::Margin::symmetric(5, 5))).show(ui, |ui| {
            ui.heading("Central");
            let mut job = egui::text::LayoutJob::default();
            job.append(
                "RICH TEXT",
                0.0,
                egui::TextFormat {
                    color: Color32::RED,
                    underline: egui::Stroke {
                        width: 2.,
                        color: Color32::BLACK,
                    },
                    ..egui::TextFormat::default()
                },
            );
            ui.add(egui::Label::new(job));
            ui.with_layout(Layout::top_down_justified(egui::Align::Center), |ui| {
                egui::TextEdit::multiline(&mut self.text_box_text)
                    .desired_rows(1)
                    .show(ui);
            });
        });
    }
}
