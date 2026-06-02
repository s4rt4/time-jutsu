//! App struct utama + state global + window behavior (titlebar, close-to-tray).
//!
//! Layout multi-panel (terbukti pas di 340×520). Sudut window dibulatkan
//! lewat DWM (Windows) — lihat `utils::platform::round_window_corners`.
//! Kedalaman dibuat lewat beda warna (chrome SURFACE vs konten BG),
//! garis pembatas antar-panel dimatikan (`show_separator_line(false)`).

use std::time::Duration;

use eframe::CreationContext;
use egui::{vec2, Align, Context, Frame, Id, Layout, RichText, Sense};

use crate::core::alarm::AlarmState;
use crate::core::countdown::TimerState;
use crate::core::pomodoro::{PhaseSwitch, Pomodoro};
use crate::core::scheduler::Scheduler;
use crate::core::tracking::Tracker;
use crate::ui;
use crate::ui::theme;
use crate::utils;
use crate::utils::config::{BreakAlertCfg, Config, PomodoroCfg};
use crate::utils::tray::{TrayCommand, TrayState};

const TITLEBAR_H: f32 = 30.0;
const SIDEBAR_W: f32 = 44.0;

/// Tab navigasi (icon-only).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Dashboard,
    Pomodoro,
    Timer,
    Tracking,
    Scheduler,
    Alarm,
    About,
    Settings,
}

impl Tab {
    /// Tab utama (di atas sidebar). `About` di bawah sidebar, `Settings` via titlebar.
    pub const ALL: [Tab; 6] = [
        Tab::Dashboard,
        Tab::Pomodoro,
        Tab::Timer,
        Tab::Tracking,
        Tab::Scheduler,
        Tab::Alarm,
    ];

    pub fn glyph(self) -> &'static str {
        use egui_phosphor::regular as icon;
        match self {
            Tab::Dashboard => icon::GAUGE,
            Tab::Pomodoro => icon::TIMER,
            Tab::Timer => icon::HOURGLASS,
            Tab::Tracking => icon::BRIEFCASE,
            Tab::Scheduler => icon::POWER,
            Tab::Alarm => icon::BELL,
            Tab::About => icon::INFO,
            Tab::Settings => icon::GEAR,
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Tab::Dashboard => "Dashboard",
            Tab::Pomodoro => "Pomodoro",
            Tab::Timer => "Timer",
            Tab::Tracking => "Time Tracking",
            Tab::Scheduler => "Scheduler",
            Tab::Alarm => "Alarm & Reminder",
            Tab::About => "About",
            Tab::Settings => "Settings",
        }
    }
}

pub struct TimeJutsuApp {
    current_tab: Tab,
    tray: Option<TrayState>,
    /// true hanya saat user pilih Quit dari tray → izinkan window benar-benar close.
    allow_exit: bool,
    /// true setelah sudut window dibulatkan (DWM). Dicoba tiap frame sampai sukses.
    corners_rounded: bool,
    /// true saat window sedang minimized → skip render agar tidak ada frame
    /// berukuran mini yang di-stretch OS saat restore (bikin transisi kasar).
    was_minimized: bool,
    /// true bila Pomodoro di-pause otomatis karena idle (untuk auto-resume).
    idle_paused: bool,
    pomodoro: Pomodoro,
    timer: TimerState,
    scheduler: Scheduler,
    alarm: AlarmState,
    tracker: Tracker,
    config: Config,
}

impl TimeJutsuApp {
    pub fn new(cc: &CreationContext<'_>) -> Self {
        // Muat config dulu → set tema sebelum apply visuals.
        let config = Config::load();
        theme::set_theme(config.theme);
        theme::setup_fonts(&cc.egui_ctx);
        theme::apply(&cc.egui_ctx);
        egui_extras::install_image_loaders(&cc.egui_ctx);

        // Thread heartbeat: bangunkan UI tiap 1 detik supaya alarm/scheduler/
        // timer tetap berjalan & tray tetap responsif walau window di-hide ke
        // tray (saat hidden, eframe tidak repaint sendiri).
        let beat_ctx = cc.egui_ctx.clone();
        std::thread::spawn(move || loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
            beat_ctx.request_repaint();
        });

        // Terapkan config ke state runtime.
        let mut pomodoro = Pomodoro::default();
        pomodoro.work_minutes = config.pomodoro.work_minutes.max(1);
        pomodoro.break_minutes = config.pomodoro.break_minutes.max(1);
        pomodoro.long_break_minutes = config.pomodoro.long_break_minutes.max(1);
        pomodoro.sound_enabled = config.pomodoro.sound_enabled;
        pomodoro.reset();

        let mut alarm = AlarmState::default();
        alarm.alarms = config.alarms.clone();
        alarm.break_alert.enabled = config.break_alert.enabled;
        alarm.break_alert.interval_minutes = config.break_alert.interval_minutes.max(1);
        alarm.sound = config.alarm_sound;

        let mut timer = TimerState::default();
        timer.deadlines = config.deadlines.clone();

        let mut tracker = Tracker::default();
        tracker.projects = config.projects.clone();

        Self {
            current_tab: Tab::Pomodoro,
            tray: TrayState::new(&cc.egui_ctx),
            allow_exit: false,
            corners_rounded: false,
            was_minimized: false,
            idle_paused: false,
            pomodoro,
            timer,
            scheduler: Scheduler::default(),
            alarm,
            tracker,
            config,
        }
    }

    /// Salin state runtime → config lalu tulis ke disk.
    fn save_config(&mut self) {
        self.config.pomodoro = PomodoroCfg {
            work_minutes: self.pomodoro.work_minutes,
            break_minutes: self.pomodoro.break_minutes,
            long_break_minutes: self.pomodoro.long_break_minutes,
            sound_enabled: self.pomodoro.sound_enabled,
        };
        self.config.alarms = self.alarm.alarms.clone();
        self.config.break_alert = BreakAlertCfg {
            enabled: self.alarm.break_alert.enabled,
            interval_minutes: self.alarm.break_alert.interval_minutes,
        };
        self.config.alarm_sound = self.alarm.sound;
        self.tracker.flush(); // commit sesi tracking yang berjalan
        self.config.projects = self.tracker.projects.clone();
        self.config.deadlines = self.timer.deadlines.clone();
        self.config.save();
    }

    /// Catat satu sesi fokus selesai ke daily log hari ini.
    fn record_focus_session(&mut self) {
        let key = chrono::Local::now().format("%Y-%m-%d").to_string();
        let entry = self.config.daily_log.entry(key).or_default();
        entry.pomodoro_sessions += 1;
        entry.focus_minutes += self.pomodoro.work_minutes;
    }

    /// Data 7 hari terakhir (label hari, menit fokus) + streak hari beruntun.
    fn week_stats(&self) -> (Vec<(String, u32)>, u32) {
        use chrono::{Datelike, Duration, Local, Weekday};
        let day_lbl = |w: Weekday| match w {
            Weekday::Mon => "Sn",
            Weekday::Tue => "Sl",
            Weekday::Wed => "Rb",
            Weekday::Thu => "Km",
            Weekday::Fri => "Jm",
            Weekday::Sat => "Sb",
            Weekday::Sun => "Mg",
        };
        let sessions_on = |key: &str| {
            self.config
                .daily_log
                .get(key)
                .map(|d| d.pomodoro_sessions)
                .unwrap_or(0)
        };

        let today = Local::now().date_naive();
        let mut week = Vec::with_capacity(7);
        for i in (0..7).rev() {
            let d = today - Duration::days(i);
            let key = d.format("%Y-%m-%d").to_string();
            let mins = self
                .config
                .daily_log
                .get(&key)
                .map(|x| x.focus_minutes)
                .unwrap_or(0);
            week.push((day_lbl(d.weekday()).to_string(), mins));
        }

        // streak: hari beruntun ada sesi (hari ini boleh belum diisi).
        let mut streak = 0;
        let mut d = today;
        if sessions_on(&today.format("%Y-%m-%d").to_string()) == 0 {
            d -= Duration::days(1);
        }
        while sessions_on(&d.format("%Y-%m-%d").to_string()) > 0 {
            streak += 1;
            d -= Duration::days(1);
        }
        (week, streak)
    }

    /// Ringkasan hari ini (sessions, focus_minutes).
    fn today_log(&self) -> (u32, u32) {
        let key = chrono::Local::now().format("%Y-%m-%d").to_string();
        self.config
            .daily_log
            .get(&key)
            .map(|d| (d.pomodoro_sessions, d.focus_minutes))
            .unwrap_or((0, 0))
    }

    /// Tick countdown timer & scheduler; picu notifikasi/aksi sistem.
    fn tick_timers(&mut self) {
        for label in self.timer.tick() {
            utils::notifier::notify("Time-Jutsu", &format!("Timer selesai: {label}"));
            if self.pomodoro.sound_enabled {
                utils::sound::play(self.alarm.sound);
            }
        }
        if let Some(action) = self.scheduler.tick() {
            utils::notifier::notify("Time-Jutsu", &format!("Menjalankan: {action}"));
        }
    }

    /// Maju-kan pomodoro; bila fase berganti, picu notifikasi + suara.
    fn tick_pomodoro(&mut self) {
        if let Some(ev) = self.pomodoro.tick() {
            let body = match ev {
                PhaseSwitch::ToBreak => "Sesi fokus selesai! Saatnya istirahat.",
                PhaseSwitch::ToFocus => "Istirahat selesai. Kembali fokus!",
            };
            utils::notifier::notify("Time-Jutsu", body);
            if self.pomodoro.sound_enabled {
                utils::sound::play(self.alarm.sound);
            }
            // Fokus selesai → catat ke daily log & simpan.
            if ev == PhaseSwitch::ToBreak {
                self.record_focus_session();
                self.save_config();
            }
        }
    }

    /// Auto-pause Pomodoro (fase Fokus) saat user idle > ambang, resume saat aktif.
    fn handle_idle(&mut self) {
        const IDLE_LIMIT: u64 = 120; // detik
        if !self.config.idle_autopause {
            return;
        }
        let idle = utils::platform::idle_seconds();
        let in_focus = self.pomodoro.phase() == crate::core::pomodoro::Phase::Focus;
        if self.pomodoro.is_running() && in_focus && idle >= IDLE_LIMIT {
            self.pomodoro.pause();
            self.idle_paused = true;
        } else if self.idle_paused && idle < 3 {
            // user kembali → lanjutkan
            self.pomodoro.start();
            self.idle_paused = false;
        }
    }

    /// Tick alarm + break alert; picu notifikasi/suara.
    fn tick_alarms(&mut self) {
        let (fired, break_due) = self.alarm.tick();
        let sound = self.alarm.sound;
        for label in fired {
            utils::notifier::notify("⏰ Alarm", &label);
            utils::sound::play(sound);
        }
        if break_due {
            utils::notifier::notify("Break Alert", "Saatnya istirahat sejenak 👀");
            utils::sound::play(sound);
        }
    }

    fn handle_tray(&mut self, ctx: &Context) {
        if self.tray.is_none() {
            return;
        }
        for cmd in TrayState::drain() {
            let show = |ctx: &Context| {
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            };
            match cmd {
                TrayCommand::Show => show(ctx),
                TrayCommand::OpenPomodoro => {
                    self.current_tab = Tab::Pomodoro;
                    show(ctx);
                }
                TrayCommand::OpenTracking => {
                    self.current_tab = Tab::Tracking;
                    show(ctx);
                }
                TrayCommand::OpenDashboard => {
                    self.current_tab = Tab::Dashboard;
                    show(ctx);
                }
                TrayCommand::Quit => {
                    self.allow_exit = true;
                    self.save_config();
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
    }

    /// Close → ke tray jika `tray_on_close`, selain itu benar-benar keluar.
    fn handle_close(&mut self, ctx: &Context) {
        if ctx.input(|i| i.viewport().close_requested()) {
            let to_tray = self.tray.is_some() && self.config.tray_on_close && !self.allow_exit;
            self.save_config();
            if !to_tray {
                return; // benar-benar tutup
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        }
    }

    /// Custom titlebar: drag area + judul + tombol settings/minimize/close.
    fn titlebar(&mut self, ctx: &Context) {
        use egui_phosphor::regular as icon;
        let tray_on_minimize = self.tray.is_some() && self.config.tray_on_minimize;
        let tab_title = self.current_tab.title();
        let mut open_settings = false;
        let mut do_minimize = false;
        let mut do_close = false;

        egui::TopBottomPanel::top("titlebar")
            .exact_height(TITLEBAR_H)
            .frame(Frame::new().fill(theme::surface()))
            .show_separator_line(false)
            .show(ctx, |ui| {
                let drag =
                    ui.interact(ui.max_rect(), Id::new("titlebar_drag"), Sense::click_and_drag());
                if drag.is_pointer_button_down_on() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                }

                ui.horizontal_centered(|ui| {
                    ui.add_space(10.0);
                    ui.label(
                        RichText::new("Time-Jutsu")
                            .color(theme::text())
                            .strong()
                            .size(13.0),
                    );
                    ui.label(
                        RichText::new(format!("|  {tab_title}"))
                            .color(theme::muted())
                            .size(12.0),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.add_space(4.0);
                        if title_button(ui, icon::X).on_hover_text("Tutup").clicked() {
                            do_close = true;
                        }
                        if title_button(ui, icon::MINUS).on_hover_text("Minimize").clicked() {
                            do_minimize = true;
                        }
                        if title_button(ui, icon::GEAR).on_hover_text("Pengaturan").clicked() {
                            open_settings = true;
                        }
                    });
                });
            });

        if open_settings {
            self.current_tab = Tab::Settings;
        }
        if do_minimize {
            if tray_on_minimize {
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            } else {
                ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
            }
        }
        if do_close {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}

impl eframe::App for TimeJutsuApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        theme::bg().to_normalized_gamma_f32()
    }

    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Bulatkan sudut window via DWM (sekali, ulangi sampai window ketemu).
        if !self.corners_rounded {
            self.corners_rounded = utils::platform::round_window_corners("Time-Jutsu");
        }

        self.handle_tray(ctx);
        self.handle_close(ctx);
        self.handle_idle();
        self.tick_pomodoro();
        self.tick_timers();
        self.tick_alarms();

        // Saat minimized: jangan render panel (hindari frame mini yang
        // di-stretch OS saat restore). Tetap polling tray via repaint lambat.
        let minimized = ctx.input(|i| i.viewport().minimized).unwrap_or(false);
        if minimized {
            self.was_minimized = true;
            ctx.request_repaint_after(Duration::from_millis(300));
            return;
        }
        if self.was_minimized {
            // baru saja restore → paksa frame penuh yang crisp sekarang juga.
            self.was_minimized = false;
            ctx.request_repaint();
        }

        self.titlebar(ctx);

        egui::SidePanel::left("sidebar")
            .exact_width(SIDEBAR_W)
            .resizable(false)
            .frame(Frame::new().fill(theme::surface()))
            .show_separator_line(false)
            .show(ctx, |ui| {
                ui::sidebar::render(ui, &mut self.current_tab);
            });

        let mut cfg_changed = false;
        let mut theme_changed = false;
        egui::CentralPanel::default()
            .frame(Frame::new().fill(theme::bg()).inner_margin(12.0))
            .show(ctx, |ui| match self.current_tab {
                Tab::Pomodoro => ui::tab_pomodoro::render(ui, &mut self.pomodoro),
                Tab::Timer => {
                    cfg_changed = ui::tab_timer::render(ui, &mut self.timer);
                }
                Tab::Tracking => {
                    cfg_changed = ui::tab_tracking::render(ui, &mut self.tracker);
                }
                Tab::Scheduler => ui::tab_scheduler::render(ui, &mut self.scheduler),
                Tab::Alarm => {
                    cfg_changed = ui::tab_alarm::render(ui, &mut self.alarm);
                }
                Tab::Dashboard => {
                    let log = self.today_log();
                    let (week, streak) = self.week_stats();
                    ui::tab_dashboard::render(
                        ui,
                        &self.pomodoro,
                        &self.timer,
                        &self.scheduler,
                        &self.alarm,
                        &self.tracker,
                        log,
                        &week,
                        streak,
                    );
                }
                Tab::About => ui::tab_about::render(ui),
                Tab::Settings => {
                    let res = ui::tab_settings::render(
                        ui,
                        &mut self.alarm.sound,
                        &mut self.config.theme,
                        &mut self.config.tray_on_close,
                        &mut self.config.tray_on_minimize,
                        &mut self.pomodoro.long_break_minutes,
                        &mut self.config.idle_autopause,
                    );
                    cfg_changed = res.changed;
                    theme_changed = res.theme_changed;
                }
            });

        if theme_changed {
            theme::set_theme(self.config.theme);
            theme::apply(ctx);
        }
        if cfg_changed || theme_changed {
            self.save_config();
        }

        // Repaint: cepat saat timer jalan (arc mulus), lambat saat idle
        // (cukup untuk polling tray walau window di-hide). Wall-clock based.
        let interval = if self.timer.stopwatch.is_running() {
            33 // stopwatch: ~30fps utk centisecond yang mulus
        } else if self.pomodoro.is_running() || self.timer.any_running() || self.tracker.any_active()
        {
            100
        } else {
            250
        };
        ctx.request_repaint_after(Duration::from_millis(interval));
    }
}

fn title_button(ui: &mut egui::Ui, glyph: &str) -> egui::Response {
    let btn = egui::Button::new(RichText::new(glyph).color(theme::muted()).size(13.0))
        .frame(false)
        .min_size(vec2(26.0, 22.0));
    ui.add(btn)
}
