use egui::{Button, Color32, Context, Response, RichText, Rounding, Ui, Widget};

#[derive(Debug, Clone, Copy)]
pub struct Timer {
    countdown_from: Option<f32>,
    running_since: Option<f32>,
    elapsed_previous: f32,
}

impl Timer {

    fn new_running(ctx: &Context) -> Self {
        Self { running_since: Some(time(ctx)), .. Self::new_paused()}
    }

    fn new_countdown_running(minutes: u32, ctx: &Context) -> Self {
        Self { running_since: Some(time(ctx)), .. Self::new_countdown_paused(minutes)}
    }

    fn new_paused() -> Self {
        Self { running_since: None, elapsed_previous: 0.0, countdown_from: None }
    }

    fn new_countdown_paused(minutes: u32) -> Self {
        Self { running_since: None, elapsed_previous: 0.0, countdown_from: Some((minutes * 60) as f32) }
    }

    fn toggle(&mut self, ctx: &Context) {
        if let Some(start) = self.running_since {
            let dt = time(ctx) - start;
            self.running_since = None;
            self.elapsed_previous += dt;
        } else {
            self.running_since = Some(time(ctx));
        }
    }

    fn pause(&mut self, ctx: &Context) {
        if self.running_since.is_some() {
            self.toggle(ctx);
        }
    }

    fn resume(&mut self, ctx: &Context) {
        if self.running_since.is_none() {
            self.toggle(ctx);
        }
    }

    fn display(&self, ctx: &Context) -> String {
        let total_elapsed = self.elapsed(ctx);
        let mut t = if let Some(start) = self.countdown_from {
            start - total_elapsed
        } else {
            total_elapsed
        } as i32;
        if t < 0 { t = - t; }
        format!("{}:{:02}", t / 60, t % 60)
    }


    fn expired(&self, ctx: &Context) -> bool {
        if let Some(start) = self.countdown_from {
            return self.elapsed(ctx) > start
        }
        false
    }

    fn elapsed(&self, ctx: &Context) -> f32 {
        self.elapsed_previous + self.running_since.map_or(0.0, |t| time(ctx) - t)
    }
}

fn time(ctx: &Context) -> f32 {
    ctx.input(|i| i.time) as _
}

enum State {
    AwaitingPlayers, // WarmUp
    WarmUp(Timer), // WarmUp finished
    Paused(Timer),     // Play, TimeOut, MedicalTimeOut
    Playing(Timer),    // Pause, TimeOut, (MedicalTimeOut)
    TimeOut{ timer: Timer, set_duration: Timer, kind: TimeOutKind }, // Resume, (Medical)
    BetweenSets(Timer), // Resume, (Finish)
}

#[derive(Debug, PartialEq, Eq)]
enum TimeOutKind { Tactical, Medical }

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TTUmpire {

    #[serde(skip)]
    state: State,

    // Example stuff:
    label: String,

    #[serde(skip)] // This how you opt-out of serialization of a field
    value: u64,
}

impl Default for TTUmpire {
    fn default() -> Self {
        Self {
            state: State::AwaitingPlayers,
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 1000,
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

        Default::default()
    }
}

macro_rules! cols {
    ($a:ident          ) => { ClockColors { default: Color32::$a, expired: None } };
    ($a:ident, $b:ident) => { ClockColors { default: Color32::$a, expired: Some(Color32::$b) } };
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
            macro_rules! repaint { () => { ctx.request_repaint_after(std::time::Duration::from_millis(self.value)); }; }
            macro_rules! timeout {
                ($timer:ident $kind:ident $minutes:expr) => {
                    State::TimeOut { timer: Timer::new_countdown_running($minutes, ctx), set_duration: $timer, kind: TimeOutKind::$kind }
                };
            }

            const HEAD_CLOCK_SEP   : f32 = 30.0;
            const CLOCK_BUTTON_SEP : f32 = 30.0;
            const BUTTON_SEP       : f32 = 30.0;

            match &mut self.state {
                State::AwaitingPlayers => {
                    ui.vertical_centered(|ui| {
                        header(ui, "Awaiting players");
                        ui.add_space(50.0);
                        if button(ui, "Start warm-up").clicked() {
                            self.state = State::WarmUp(Timer::new_countdown_running(2, ctx));
                        }
                    });
                }
                State::WarmUp(timer) => {
                    let timer = *timer;
                    ui.vertical_centered(|ui| {
                        header(ui, "Warm-up");
                        ui.add_space(HEAD_CLOCK_SEP);
                        clock(ui, ctx, timer, cols!(LIGHT_BLUE, RED));
                        ui.add_space(CLOCK_BUTTON_SEP);
                        if button(ui, "Start match").clicked() {
                            self.state = State::Playing(Timer::new_running(ctx));
                        }
                        repaint!();
                    });
                }
                State::Playing(timer) => {
                    let mut pause   = false;
                    let mut timeout = false;
                    let mut medical = false;
                    let mut finish  = false;
                    ui.vertical_centered(|ui| {
                        header(ui, "Playing");
                        ui.add_space(HEAD_CLOCK_SEP);
                        clock(ui, ctx, *timer, cols!(GREEN, RED));
                        ui.add_space(CLOCK_BUTTON_SEP);
                        pause   = button(ui, "Pause")            .clicked();
                        ui.add_space(BUTTON_SEP);
                        timeout = button(ui, "Time-out")         .clicked();
                        ui.add_space(BUTTON_SEP);
                        medical = button(ui, "Medical time-out") .clicked();
                        ui.add_space(BUTTON_SEP);
                        finish  = button(ui, "Set finished")     .clicked();
                    });
                    if pause || timeout || medical {
                        timer.pause(ctx);
                    }
                    let timer = *timer;
                    if      pause   { self.state = State::Paused(timer) }
                    else if timeout { self.state = timeout!(timer Tactical 1) }
                    else if medical { self.state = timeout!(timer Medical 10) }
                    else if finish  { self.state = State::BetweenSets(Timer::new_countdown_running(1, ctx)) }
                    repaint!();
                }
                State::Paused(timer) => {
                    let mut play    = false;
                    let mut timeout = false;
                    let mut medical = false;
                    ui.vertical_centered(|ui| {
                        header(ui, "Paused");
                        ui.add_space(HEAD_CLOCK_SEP);
                        clock(ui, ctx, *timer, cols!(ORANGE));
                        ui.add_space(CLOCK_BUTTON_SEP);
                        play    = button(ui, "Play")             .clicked();
                        ui.add_space(BUTTON_SEP);
                        timeout = button(ui, "Time-out")         .clicked();
                        ui.add_space(BUTTON_SEP);
                        medical = button(ui, "Medical time-out") .clicked();
                    });
                    if play                    { timer.resume(ctx) }
                    else if timeout || medical { timer.pause (ctx) }
                    let timer = *timer;
                    if      play    { self.state = State::Playing(timer) }
                    else if timeout { self.state = timeout!(timer Tactical 1) }
                         if medical { self.state = timeout!(timer Medical 10) }
                }
                State::TimeOut { timer, kind, set_duration } => {
                    if ui.vertical_centered(|ui| {
                        header(ui, if *kind == TimeOutKind::Medical {"Medical Time-out"} else {"Time-out"});
                        ui.add_space(HEAD_CLOCK_SEP);
                        clock(ui, ctx, *timer, cols!(LIGHT_BLUE, RED));
                        ui.add_space(CLOCK_BUTTON_SEP);
                        button(ui, "Play")
                    }).inner.clicked() {
                        set_duration.resume(ctx);
                        self.state = State::Playing(*set_duration);
                    }
                    repaint!();
                }
                State::BetweenSets(timer)  => {
                    let mut play = false;
                    let mut done = false;
                    ui.vertical_centered(|ui| {
                        header(ui, "Pause between sets");
                        ui.add_space(HEAD_CLOCK_SEP);
                        clock(ui, ctx, *timer, cols!(LIGHT_BLUE, RED));
                        ui.add_space(CLOCK_BUTTON_SEP);
                        if button(ui, "Play")          .clicked() { play = true; }
                        ui.add_space(BUTTON_SEP);
                        if button(ui, "Match finished").clicked() { done = true; }
                    });
                    if play { self.state = State::Playing(Timer::new_running(ctx)) }
                    if done { self.state = State::AwaitingPlayers }
                    repaint!();
                }
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
            });
        });

    }
}

fn rich(text: impl Into<String>) -> RichText {
    RichText::new(text)
}

fn button(ui: &mut Ui, text: impl Into<String>) -> Response {
    Button::new(rich(text).size(30.0))
        .rounding(Rounding::same(13.0))
        .ui(ui)
}

fn header(ui: &mut Ui, text: impl Into<String>) -> Response {
    ui.label(rich(text).size(50.0))
}

fn clock(ui: &mut Ui, ctx: &Context, timer: Timer, cols: ClockColors) -> Response {
    let mut color = cols.default;
    if let Some(expired_color) = cols.expired {
        if timer.expired(ctx) {
            color = expired_color;
        }
    }
    let text = timer.display(ctx);
    ui.label(rich(text).size(80.0).color(color))
}

struct ClockColors {
    default: Color32,
    expired: Option<Color32>,
}
