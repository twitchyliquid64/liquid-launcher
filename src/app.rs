use egui::TextEdit;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

use crate::ext::{EquationExtra, ImmediateExtra, NumFormatExtra};

const ICON_SIZE: [usize; 2] = [32, 32];

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct Launcher {
    #[serde(skip)]
    input: String,
    #[serde(skip)]
    focus_input: bool,

    #[serde(skip)]
    applications: Arc<Mutex<Option<crate::sys_apps::AppList>>>,
    #[serde(skip)]
    matching_app_idx: Option<Vec<usize>>,
    #[serde(skip)]
    selected_idx: usize,
    #[serde(skip)]
    icons: Arc<Mutex<HashMap<String, PathBuf>>>,
    #[serde(skip)]
    matcher: SkimMatcherV2,

    #[serde(skip)]
    extras: Vec<Box<dyn ImmediateExtra>>,
}

impl Default for Launcher {
    fn default() -> Self {
        Self {
            // Example stuff:
            input: "".to_owned(),
            focus_input: true,
            applications: Arc::new(Mutex::new(None)),
            icons: Arc::new(Mutex::new(HashMap::new())),
            matcher: SkimMatcherV2::default(),
            matching_app_idx: None,
            selected_idx: 0,
            extras: vec![
                Box::new(NumFormatExtra::default()),
                Box::new(EquationExtra::default()),
            ],
        }
    }
}

impl Launcher {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        configure_text_styles(&cc.egui_ctx);
        egui_extras::install_image_loaders(&cc.egui_ctx);

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        let prev_state = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Self::default()
        };

        let applications = Arc::new(Mutex::new(None));
        let applications2 = applications.clone();
        let icons = Arc::new(Mutex::new(HashMap::with_capacity(32)));
        let icons2 = icons.clone();
        let frame = cc.egui_ctx.clone();

        // When compiling natively, implement the applications list.
        #[cfg(not(target_arch = "wasm32"))]
        {
            std::thread::spawn(move || match crate::sys_apps::AppList::new() {
                Err(e) => println!("failed to load system applications: {:?}", e),
                Ok(apps_list) => {
                    {
                        let mut data = applications2.lock().unwrap();
                        *data = Some(apps_list.clone());
                    }
                    frame.request_repaint();

                    // iterate through each app and attempt to load the icon.
                    for (i, app) in apps_list.apps.iter().enumerate() {
                        if let Some(path) = app.find_icon(ICON_SIZE[0] as u16) {
                            {
                                let mut data = icons2.lock().unwrap();
                                (*data).insert(app.name.clone(), path);
                            }
                        }
                        // request redraw after every 12 entries
                        if i % 12 == 0 {
                            frame.request_repaint();
                        }
                    }
                    frame.request_repaint();
                }
            });
        }

        Self {
            icons,
            applications,
            ..prev_state
        }
    }

    fn compute_app_indices(
        matcher: &SkimMatcherV2,
        apps_list: &crate::sys_apps::AppList,
        input: &String,
    ) -> Vec<usize> {
        if input.len() == 0 {
            apps_list
                .apps
                .iter()
                .enumerate()
                .map(|(i, _app)| i)
                .collect()
        } else {
            let mut idx_scores: Vec<(usize, i64)> = apps_list
                .apps
                .iter()
                .enumerate()
                .map(|(i, app)| match matcher.fuzzy_match(&app.name, input) {
                    Some(score) => Some((i, score)),
                    None => None,
                })
                .filter(|e| e.is_some())
                .map(|e| e.unwrap())
                .collect();

            idx_scores.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

            idx_scores.into_iter().map(|e| e.0).collect()
        }
    }

    fn handle_input_changed(&mut self) {
        self.matching_app_idx = match &*self.applications.lock().unwrap() {
            None => None,
            Some(apps_list) => Some(Launcher::compute_app_indices(
                &self.matcher,
                apps_list,
                &self.input,
            )),
        };

        if self.matching_app_idx.as_ref().map(|v| v.len()).unwrap_or(0) <= self.selected_idx {
            self.selected_idx = 0;
        }
    }

    fn ui_for_app_entry(
        &self,
        app: &crate::sys_apps::App,
        selected: bool,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        icons: &HashMap<String, PathBuf>,
    ) {
        ui.allocate_space(egui::Vec2::new(0., 2.));

        if let Some(icon_path) = icons.get(&app.name) {
            let uri = "file://".to_owned() + icon_path.to_str().unwrap();
            ui.add(
                egui::Image::from_uri(uri)
                    .fit_to_exact_size(egui::Vec2::new(ICON_SIZE[0] as f32, ICON_SIZE[1] as f32)),
            );
        } else {
            //ui.label("🔆");
            ui.add(
                egui::Image::new(egui::include_image!("../unknown-app.png"))
                    .fit_to_exact_size(egui::Vec2::new(ICON_SIZE[0] as f32, ICON_SIZE[1] as f32)),
            );
        }

        let label = ui.selectable_label(selected, app.name.clone());
        if label.clicked() {
            app.run(true);
        }
    }
}

impl eframe::App for Launcher {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.allocate_space(egui::Vec2::new(0., 4.));
            ui.horizontal(|ui| {
                let fbc = ui.visuals().faint_bg_color;
                ui.visuals_mut().extreme_bg_color = fbc;
                ui.label("🔎");
                let input = ui.add_sized(
                    ui.available_size(),
                    TextEdit::multiline(&mut self.input).desired_rows(1), // .hint_text("Start typing ...")
                                                                          //.horizontal_align(egui::Align::Center)
                );
                if self.focus_input {
                    self.focus_input = false;
                    input.request_focus();
                }

                let (down, up, enter) = if input.has_focus() {
                    ui.input(|i| {
                        (
                            i.key_pressed(egui::Key::ArrowDown),
                            i.key_pressed(egui::Key::ArrowUp),
                            i.key_pressed(egui::Key::Enter),
                        )
                    })
                } else {
                    (false, false, false)
                };

                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    std::process::exit(0);
                }
                if enter && self.matching_app_idx.as_ref().map(|v| v.len()).unwrap_or(0) > 0 {
                    let apps_mutex = &*self.applications.lock().unwrap();
                    match (apps_mutex, &self.matching_app_idx) {
                        (Some(apps_list), Some(idxs)) => {
                            if self.selected_idx < idxs.len() {
                                apps_list.apps[idxs[self.selected_idx]].run(true);
                            }
                        }
                        _ => {}
                    }
                    if self.input.ends_with("\n") {
                        self.input.pop();
                    }
                } else if input.changed() || down || up {
                    self.handle_input_changed();
                }
                if down || up {
                    let length = self.matching_app_idx.as_ref().map(|v| v.len()).unwrap_or(0);
                    if length > 0 {
                        if down && self.selected_idx < length - 1 {
                            self.selected_idx += 1;
                        } else if up && self.selected_idx > 0 {
                            self.selected_idx -= 1;
                        }
                    } else {
                        self.selected_idx = 0;
                    }
                }
            });
            ui.allocate_space(egui::Vec2::new(0., 4.));
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut had_extra = false;
            for ext in self.extras.iter_mut() {
                let had_content = ext.ui(&self.input, ctx, ui);
                if had_content {
                    ui.allocate_space(egui::Vec2::new(0., 2.));
                }
                had_extra |= had_content;
            }

            if had_extra {
                ui.separator();
            }

            {
                let row_height = ui
                    .text_style_height(&egui::TextStyle::Body)
                    .max(ICON_SIZE[1] as f32);

                let apps_mutex = &*self.applications.lock().unwrap();
                let icons_mutex = &*self.icons.lock().unwrap();
                match (apps_mutex, &self.matching_app_idx) {
                    (Some(apps_list), Some(idx)) => {
                        egui::ScrollArea::vertical()
                            .auto_shrink([false; 2])
                            .show_rows(ui, row_height, idx.len(), |ui, row_range| {
                                egui::Grid::new("apps_grid").num_columns(3).show(ui, |ui| {
                                    for row in row_range {
                                        self.ui_for_app_entry(
                                            &apps_list.apps[idx[row]],
                                            self.selected_idx == row,
                                            ctx,
                                            ui,
                                            icons_mutex,
                                        );
                                        ui.end_row();
                                    }
                                });
                            });
                    }
                    (Some(apps_list), None) => {
                        egui::ScrollArea::vertical()
                            .auto_shrink([false; 2])
                            .show_rows(ui, row_height, apps_list.apps.len(), |ui, row_range| {
                                egui::Grid::new("apps_grid").num_columns(3).show(ui, |ui| {
                                    for row in row_range {
                                        self.ui_for_app_entry(
                                            &apps_list.apps[row],
                                            self.selected_idx == row,
                                            ctx,
                                            ui,
                                            icons_mutex,
                                        );
                                        ui.end_row();
                                    }
                                });
                            });
                    }

                    _ => {}
                }
            }

            egui::warn_if_debug_build(
                &mut ui.child_ui(
                    ui.max_rect()
                        .split_left_right_at_x(ui.max_rect().max.x - 155.)
                        .1,
                    egui::Layout::default(),
                ),
            );
        });
    }
}

fn configure_text_styles(ctx: &egui::Context) {
    use egui::FontFamily::{Monospace, Proportional};
    use egui::{FontId, TextStyle};

    let mut style = (*ctx.style()).clone();
    style.text_styles = [
        (TextStyle::Heading, FontId::new(28.0, Proportional)),
        (TextStyle::Body, FontId::new(20.0, Proportional)),
        (TextStyle::Monospace, FontId::new(16.0, Monospace)),
        (TextStyle::Button, FontId::new(16.0, Proportional)),
        (TextStyle::Small, FontId::new(15.0, Proportional)),
    ]
    .into();
    ctx.set_style(style);
}
