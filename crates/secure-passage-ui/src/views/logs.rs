use crate::theme;
use egui::RichText;

#[derive(Default)]
pub struct LogsState {
    pub open: bool,
    pub entries: Vec<String>,
    pub filter: String,
}

impl LogsState {
    pub fn push(&mut self, level: &str, message: &str) {
        let ts = chrono::Local::now().format("%H:%M:%S");
        self.entries.push(format!("[{ts}] [{level}] {message}"));
        if self.entries.len() > 2000 {
            self.entries.drain(0..500);
        }
    }
}

pub fn show(ctx: &egui::Context, state: &mut LogsState) {
    let mut open = state.open;
    egui::Window::new("Logs")
        .open(&mut open)
        .default_size([640.0, 400.0])
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Filter:");
                ui.text_edit_singleline(&mut state.filter);
                if theme::secondary_button(ui, "Clear").clicked() {
                    state.entries.clear();
                }
            });
            ui.separator();
            egui::ScrollArea::vertical()
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for line in &state.entries {
                        if state.filter.is_empty()
                            || line.to_lowercase().contains(&state.filter.to_lowercase())
                        {
                            ui.label(RichText::new(line).monospace().color(theme::TEXT));
                        }
                    }
                });
        });
    state.open = open;
}
