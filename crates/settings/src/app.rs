use std::path::PathBuf;

use broxer_core::{Config, ConfigError};
use broxer_platform::detect::installed_browsers;
use eframe::egui;

use crate::form::{BrowserRow, Form, RuleRow};
use crate::register;

pub struct SettingsApp {
    form: Form,
    config_path: PathBuf,
    test_url: String,
    status: String,
}

enum RuleAction {
    Up(usize),
    Down(usize),
    Remove(usize),
}

impl SettingsApp {
    pub fn new(config_path: PathBuf) -> SettingsApp {
        let (form, status) = match Config::load(&config_path) {
            Ok(config) => (Form::from_config(&config), String::new()),
            Err(ConfigError::NotFound) => (
                Form::from_detected(&installed_browsers()),
                "No config yet: installed browsers were filled in. Add rules and Save.".into(),
            ),
            Err(e) => (
                Form::from_detected(&installed_browsers()),
                format!("{e}. Saving will replace {}.", config_path.display()),
            ),
        };
        SettingsApp { form, config_path, test_url: String::new(), status }
    }

    fn save(&mut self) {
        self.status = match self
            .form
            .to_config()
            .and_then(|c| c.save(&self.config_path).map_err(|e| e.to_string()))
        {
            Ok(()) => format!("Saved to {}", self.config_path.display()),
            Err(e) => format!("Not saved: {e}"),
        };
    }

    fn register(&mut self) {
        self.status = match register::router_path().and_then(|p| register::register(&p)) {
            Ok(()) => register::AFTER_REGISTER.into(),
            Err(e) => format!("Could not register: {e}"),
        };
    }

    fn unregister(&mut self) {
        self.status = match register::unregister() {
            Ok(()) => "Unregistered.".into(),
            Err(e) => format!("Could not unregister: {e}"),
        };
    }

    fn browsers_section(&mut self, ui: &mut egui::Ui) {
        ui.heading("Browsers");
        let mut remove = None;
        egui::Grid::new("browsers").num_columns(5).striped(true).show(ui, |ui| {
            for header in ["Name", "Path", "", "Arguments", ""] {
                ui.strong(header);
            }
            ui.end_row();
            for (i, row) in self.form.browsers.iter_mut().enumerate() {
                text_field(ui, &mut row.name, 120.0);
                text_field(ui, &mut row.path, 300.0);
                if ui.button("Browse…").clicked()
                    && let Some(path) = rfd::FileDialog::new().pick_file()
                {
                    row.path = path.display().to_string();
                }
                text_field(ui, &mut row.args, 180.0);
                if ui.button("Remove").clicked() {
                    remove = Some(i);
                }
                ui.end_row();
            }
        });
        if let Some(i) = remove {
            self.form.browsers.remove(i);
        }
        if ui.button("Add browser").clicked() {
            self.form.browsers.push(BrowserRow::default());
        }
    }

    fn rules_section(&mut self, ui: &mut egui::Ui) {
        ui.heading("Rules");
        ui.label("Checked top to bottom; the first match wins. A domain also matches its subdomains.");
        let names = self.form.browser_names();
        let count = self.form.rules.len();
        let mut action = None;
        egui::Grid::new("rules").num_columns(3).striped(true).show(ui, |ui| {
            for header in ["Domains (comma-separated)", "Browser", ""] {
                ui.strong(header);
            }
            ui.end_row();
            for (i, rule) in self.form.rules.iter_mut().enumerate() {
                text_field(ui, &mut rule.domains, 320.0);
                browser_picker(ui, ("rule", i), &mut rule.browser, &names);
                ui.horizontal(|ui| {
                    if ui.add_enabled(i > 0, egui::Button::new("Up")).clicked() {
                        action = Some(RuleAction::Up(i));
                    }
                    if ui.add_enabled(i + 1 < count, egui::Button::new("Down")).clicked() {
                        action = Some(RuleAction::Down(i));
                    }
                    if ui.button("Remove").clicked() {
                        action = Some(RuleAction::Remove(i));
                    }
                });
                ui.end_row();
            }
        });
        match action {
            Some(RuleAction::Up(i)) => self.form.move_rule(i, true),
            Some(RuleAction::Down(i)) => self.form.move_rule(i, false),
            Some(RuleAction::Remove(i)) => {
                self.form.rules.remove(i);
            }
            None => {}
        }
        if ui.button("Add rule").clicked() {
            self.form.rules.push(RuleRow { domains: String::new(), browser: self.form.fallback.clone() });
        }
    }

    fn fallback_and_test(&mut self, ui: &mut egui::Ui) {
        let names = self.form.browser_names();
        ui.horizontal(|ui| {
            ui.label("Fallback browser (links no rule matches):");
            browser_picker(ui, "fallback", &mut self.form.fallback, &names);
        });
        ui.horizontal(|ui| {
            ui.label("Test a link:");
            text_field(ui, &mut self.test_url, 400.0);
        });
        if !self.test_url.trim().is_empty() {
            ui.label(match self.form.browser_for_test_link(&self.test_url) {
                Ok(browser) => format!("Opens in {browser}"),
                Err(e) => format!("Fix the settings first: {e}"),
            });
        }
    }

    fn actions(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Save").clicked() {
                self.save();
            }
            if ui.button("Register Broxer").clicked() {
                self.register();
            }
            if ui.button("Unregister").clicked() {
                self.unregister();
            }
            #[cfg(windows)]
            if ui.button("Open Default apps").clicked()
                && let Err(e) = register::open_default_apps()
            {
                self.status = format!("Could not open Default apps: {e}");
            }
        });
        if !self.status.is_empty() {
            ui.label(&self.status);
        }
    }
}

// Grid cells cap available width at last frame's column width, so desired_width alone never grows.
fn text_field(ui: &mut egui::Ui, text: &mut String, width: f32) {
    let height = ui.spacing().interact_size.y;
    ui.add_sized([width, height], egui::TextEdit::singleline(text));
}

fn browser_picker(ui: &mut egui::Ui, id: impl std::hash::Hash, selected: &mut String, names: &[String]) {
    egui::ComboBox::from_id_salt(id).selected_text(selected.as_str()).show_ui(ui, |ui| {
        for name in names {
            ui.selectable_value(selected, name.clone(), name.as_str());
        }
    });
}

impl eframe::App for SettingsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                self.browsers_section(ui);
                ui.separator();
                self.rules_section(ui);
                ui.separator();
                self.fallback_and_test(ui);
                ui.separator();
                self.actions(ui);
            });
        });
    }
}
