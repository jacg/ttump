use egui::Context;

pub enum Timer {
    Running {
        dt: f32,
        t0: f32,
    },
    Paused(f32),
}

fn time(ctx: &Context) -> f32 {
    ctx.input(|i| i.time) as _
}

impl Timer {

    fn new_running(ctx: &Context) -> Self {
        Self::Running { dt: 0.0, t0: ctx.input(|i| i.time as _)
        }
    }

    fn new_paused () -> Self { Self::Paused (0.0) }

    fn toggle(&mut self, ctx: &Context) {
        match *self {
            Timer::Running { dt, t0 } => *self = Timer::Paused(dt + time(ctx) - t0),
            Timer::Paused(dt) => *self = Timer::Running { dt, t0: time(ctx)  },
        }
    }

    fn pause(&mut self, ctx: &Context) {
        match *self {
            Timer::Paused(_) => (),
            Timer::Running { .. } => self.toggle(ctx),
        }
    }

    fn resume(&mut self, ctx: &Context) {
        match *self {
            Timer::Running{..} => (),
            Timer::Paused(_) => self.toggle(ctx),
        }
    }

    fn elapsed(&self, ctx: &Context) -> f32 {
        match *self {
            Timer::Running { dt, t0 } =>  dt + time(ctx) - t0,
            Timer::Paused (t) => t,
        }
    }


}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TTUmpire {

    #[serde(skip)]
    timer: Timer,

    // Example stuff:
    label: String,

    #[serde(skip)] // This how you opt-out of serialization of a field
    value: u64,
}

impl Default for TTUmpire {
    fn default() -> Self {
        Self {
            timer: Timer::new_paused(),
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 270,
        }
    }
}

impl TTUmpire {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Self {
            timer: Timer::new_running(&cc.egui_ctx),
            label: "Dummy text".into(),
            value: 270,
        }
    }
}

impl eframe::App for TTUmpire {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let t = self.timer.elapsed(ctx) as u32;
            if ui.label(format!("{}:{:02}", t/60, t%60)).clicked() {
                self.timer.toggle(ctx);
            }

            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("eframe template");

            ui.horizontal(|ui| {
                ui.label("Write something: ");
                ui.text_edit_singleline(&mut self.label);
            });

            ui.add(egui::Slider::new(&mut self.value, 0..=1000).text("value"));
            if ui.button("Increment").clicked() {
                self.value += 10;
            }
            ui.separator();

            ui.add(egui::github_link_file!(
                "https://github.com/emilk/eframe_template/blob/main/",
                "Source code."
            ));

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
        ctx.request_repaint_after(std::time::Duration::from_millis(self.value));
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
