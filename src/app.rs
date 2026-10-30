use eframe::egui::{self, Color32, Frame, RichText};

use crate::canvas::Canvas;
use crate::config;
use crate::dialog;
use crate::model::Brush;
use crate::settings;
use crate::theme::Mode;
use crate::toolbar::{self, Action};
use crate::updater::{self, Status, Updater};
use crate::widgets::{self, Kind};

/// Main application state, updated every frame by egui.
pub struct Painty {
    canvas: Canvas,
    brush: Brush,
    mode: Mode,
    canvas_follows_theme: bool,
    /// Ink color used on the previous frame, to detect a background change.
    last_ink: Option<Color32>,
    /// Settings values as last written to disk, to avoid saving every frame.
    last_saved: (Mode, bool),
    settings_open: bool,
    confirm_clear: bool,
    updater: Updater,
    /// True once the automatic update check has been started.
    update_checked: bool,
}

impl Painty {
    /// Loads the saved settings and builds the initial state.
    pub fn new() -> Self {
        // Remove the executable left over from the previous update.
        updater::cleanup();

        let cfg = config::load();
        let ink = cfg.mode.colors(cfg.canvas_follows_theme).ink;

        let mut brush = Brush::default();
        brush.color = ink;

        Self {
            canvas: Canvas::default(),
            brush,
            mode: cfg.mode,
            canvas_follows_theme: cfg.canvas_follows_theme,
            last_ink: Some(ink),
            last_saved: (cfg.mode, cfg.canvas_follows_theme),
            settings_open: false,
            confirm_clear: false,
            updater: Updater::default(),
            update_checked: false,
        }
    }
}

impl eframe::App for Painty {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Silent update check on the first frame. Errors are not shown.
        if !self.update_checked {
            self.update_checked = true;
            self.updater.check(ctx, false);
        }
        self.updater.poll(ctx);

        let c = self.mode.colors(self.canvas_follows_theme);

        // When the canvas background changes, the default ink follows (black <-> white).
        if let Some(prev) = self.last_ink {
            if prev != c.ink && self.brush.color == prev {
                self.brush.color = c.ink;
            }
        }
        self.last_ink = Some(c.ink);

        // Ctrl+Z undoes the last stroke, unless a dialog is open.
        let modal_open = self.settings_open || self.confirm_clear;
        if !modal_open && ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Z)) {
            self.canvas.undo();
        }

        egui::CentralPanel::default()
            .frame(Frame::none().fill(c.canvas_bg))
            .show(ctx, |ui| self.canvas.show(ui, &self.brush, c.canvas_bg));

        match toolbar::show(ctx, &mut self.brush, &c) {
            Some(Action::Undo) => self.canvas.undo(),
            Some(Action::Clear) => {
                // Nothing to clear: skip the confirmation.
                if !self.canvas.is_empty() {
                    self.confirm_clear = true;
                }
            }
            Some(Action::Settings) => self.settings_open = !self.settings_open,
            None => {}
        }

        settings::show(
            ctx,
            &mut self.settings_open,
            &mut self.mode,
            &mut self.canvas_follows_theme,
            &mut self.updater,
            &c,
        );

        // Persist settings as soon as they change.
        let current = (self.mode, self.canvas_follows_theme);
        if current != self.last_saved {
            config::save(current.0, current.1);
            self.last_saved = current;
        }

        // "Clear all" confirmation dialog.
        let mut confirm = false;
        let mut cancel = false;
        let dismissed = dialog::show(ctx, "confirm_clear", self.confirm_clear, &c, 300.0, |ui| {
            ui.label(RichText::new("Clear everything?").size(18.0).strong());
            ui.add_space(6.0);
            ui.label(RichText::new("This drawing will be permanently deleted.").color(c.text_dim));
            ui.add_space(16.0);
            ui.horizontal(|ui| {
                if widgets::button(ui, "keep", &c, "Keep", Kind::Soft, 146.0).clicked() {
                    cancel = true;
                }
                if widgets::button(ui, "wipe", &c, "Clear all", Kind::Danger, 146.0).clicked() {
                    confirm = true;
                }
            });
        });

        if confirm && self.confirm_clear {
            self.canvas.clear();
        }
        if confirm || cancel || dismissed {
            self.confirm_clear = false;
        }

        // Update prompt: shown only when a new version is available,
        // is being installed, or failed to install.
        let status = self.updater.status.clone();
        let prompt_open = matches!(
            status,
            Status::Available(_) | Status::Downloading | Status::InstallFailed(_)
        );
        let mut install = false;
        let mut later = false;
        let dismissed_update = dialog::show(ctx, "update", prompt_open, &c, 300.0, |ui| match &status {
            Status::Available(release) => {
                ui.label(RichText::new("Update available").size(18.0).strong());
                ui.add_space(6.0);
                ui.label(
                    RichText::new(format!("Painty {} is ready to install.", release.version))
                        .color(c.text_dim),
                );
                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    if widgets::button(ui, "later", &c, "Later", Kind::Soft, 146.0).clicked() {
                        later = true;
                    }
                    if widgets::button(ui, "install", &c, "Install", Kind::Soft, 146.0).clicked() {
                        install = true;
                    }
                });
            }
            Status::Downloading => {
                ui.label(RichText::new("Downloading update…").size(18.0).strong());
            }
            Status::InstallFailed(error) => {
                ui.label(RichText::new("Update failed").size(18.0).strong());
                ui.add_space(6.0);
                ui.label(RichText::new(error.as_str()).color(c.text_dim));
                ui.add_space(16.0);
                if widgets::button(ui, "update_close", &c, "Close", Kind::Soft, 300.0).clicked() {
                    later = true;
                }
            }
            _ => {}
        });

        if install {
            if let Status::Available(release) = self.updater.status.clone() {
                self.updater.install(ctx, release);
            }
        }
        // The prompt cannot be closed while the download is running.
        if later || (dismissed_update && !matches!(self.updater.status, Status::Downloading)) {
            self.updater.dismiss();
        }
    }
}