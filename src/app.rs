use egui::TextEdit;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

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
    icons: Arc<Mutex<HashMap<String, PathBuf>>>,
    #[serde(skip)]
    empty_icon: Option<egui::load::SizedTexture>,
}

impl Default for Launcher {
    fn default() -> Self {
        Self {
            // Example stuff:
            input: "".to_owned(),
            focus_input: true,
            applications: Arc::new(Mutex::new(None)),
            icons: Arc::new(Mutex::new(HashMap::new())),
            empty_icon: None,
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
                        // request redraw after every 8 entries
                        if i % 8 == 0 {
                            frame.request_repaint();
                        }
                    }
                    frame.request_repaint();
                }
            });
        }

        let empty_icon = Some(egui::load::SizedTexture::from_handle(
            &cc.egui_ctx.load_texture(
                "empty",
                egui::ColorImage::new(ICON_SIZE, egui::Color32::from_rgba_unmultiplied(0, 0, 0, 0)),
                Default::default(),
            ),
        ));

        Self {
            icons,
            applications,
            empty_icon,
            ..prev_state
        }
    }

    fn ui_for_app_entry(
        &self,
        app: &crate::sys_apps::App,
        ctx: &egui::Context,
        ui: &mut egui::Ui,
        icons: &HashMap<String, PathBuf>,
    ) {
        ui.allocate_space(egui::Vec2::new(0., 2.));

        if let Some(icon_path) = icons.get(&app.name) {
            let uri = "file://".to_owned() + icon_path.to_str().unwrap();
            ui.add(egui::Image::from_uri(uri).max_height(ICON_SIZE[0] as f32));
        } else {
            let tex_info = self.empty_icon.unwrap();
            use egui::widgets::ImageSource;
            ui.add(egui::Image::new(ImageSource::Texture(tex_info)));
        }

        ui.label(app.name.clone());
    }
}

impl eframe::App for Launcher {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

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
            });
            ui.allocate_space(egui::Vec2::new(0., 4.));
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            {
                let apps_mutex = &*self.applications.lock().unwrap();
                let icons_mutex = &*self.icons.lock().unwrap();
                match apps_mutex {
                    Some(apps_list) => {
                        ui.separator();

                        let row_height = ui
                            .text_style_height(&egui::TextStyle::Body)
                            .min(ICON_SIZE[1] as f32);
                        egui::ScrollArea::vertical()
                            .auto_shrink([false; 2])
                            .show_rows(ui, row_height, apps_list.apps.len(), |ui, row_range| {
                                egui::Grid::new("apps_grid").num_columns(3).show(ui, |ui| {
                                    for row in row_range {
                                        self.ui_for_app_entry(
                                            &apps_list.apps[row],
                                            ctx,
                                            ui,
                                            icons_mutex,
                                        );
                                        ui.end_row();
                                    }
                                });
                            });
                    }
                    None => {}
                }
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
            });
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
        (TextStyle::Small, FontId::new(12.0, Proportional)),
    ]
    .into();
    ctx.set_style(style);
}
