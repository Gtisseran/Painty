use eframe::egui::{self, Color32, Frame, Key, RichText};

use crate::canvas::Canvas;
use crate::config;
use crate::dialog;
use crate::export;
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
    /// True while the "quit without saving?" confirmation is shown.
    confirm_exit: bool,
    /// True once the window may close without asking (user confirmed or update restart).
    close_confirmed: bool,
    /// Message of the last failed export, shown in a dialog.
    export_error: Option<String>,
    updater: Updater,
    /// True once the automatic update check has been started.
    update_checked: bool,
}

impl Painty {
    /// Loads the saved settings and builds the initial state.
    pub fn new() -> Self {
        // Remove files left over from the previous update.
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
            confirm_exit: false,
            close_confirmed: false,
            export_error: None,
            updater: Updater::default(),
            update_checked: false,
        }
    }

    /// Asks where to save, then writes the drawing as a PNG.
    fn export(&mut self, ctx: &egui::Context, bg: Color32) {
        let Some(rect) = self.canvas.rect() else {
            return;
        };
        // Export the whole drawing, even when it no longer fits in the window.
        // An empty canvas falls back to the visible area.
        let (origin, size) = export::bounds(self.canvas.lines())
            .map(|bounds| (bounds.min, bounds.size()))
            .unwrap_or((rect.min, rect.size()));
        let Some(path) = rfd::FileDialog::new()
            .add_filter("PNG image", &["png"])
            .set_file_name("painty.png")
            .save_file()
        else {
            // Dialog cancelled.
            return;
        };
        // Add the extension if the user typed a name without one.
        let path = if path.extension().is_none() {
            path.with_extension("png")
        } else {
            path
        };

        if let Err(error) = export::save_png(
            &path,
            self.canvas.lines(),
            origin,
            size,
            bg,
            ctx.pixels_per_point(),
        ) {
            self.export_error = Some(error);
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

        // Ask before closing when the drawing would be lost. A restart after an
        // update bypasses the question: the new version is already running.
        let close_requested = ctx.input(|i| i.viewport().close_requested());
        if close_requested {
            if !self.close_confirmed && !self.canvas.is_empty() {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                self.confirm_exit = true;
                // The quit prompt takes over any other dialog.
                self.settings_open = false;
                self.confirm_clear = false;
                self.export_error = None;
            }
        } else {
            self.close_confirmed = false;
        }
        if self.updater.restart {
            self.updater.restart = false;
            self.close_confirmed = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        let c = self.mode.colors(self.canvas_follows_theme);

        // When the canvas background changes, the default ink follows (black <-> white).
        if let Some(prev) = self.last_ink {
            if prev != c.ink && self.brush.color == prev {
                self.brush.color = c.ink;
            }
        }
        self.last_ink = Some(c.ink);

        // The update prompt is modal too: block shortcuts while it is shown.
        let status = self.updater.status.clone();
        let update_prompt = matches!(
            status,
            Status::Available(_) | Status::Downloading(_) | Status::InstallFailed(_)
        );

        // Keyboard shortcuts, unless a dialog is open.
        let modal_open = self.settings_open
            || self.confirm_clear
            || self.confirm_exit
            || self.export_error.is_some()
            || update_prompt;
        if !modal_open {
            let (cmd, undo, redo, save) = ctx.input(|i| {
                (
                    i.modifiers.command,
                    i.key_pressed(Key::Z),
                    i.key_pressed(Key::Y),
                    i.key_pressed(Key::S),
                )
            });
            if cmd && undo {
                self.canvas.undo();
            }
            if cmd && redo {
                self.canvas.redo();
            }
            if cmd && save {
                self.export(ctx, c.canvas_bg);
            }
        }

        egui::CentralPanel::default()
            .frame(Frame::none().fill(c.canvas_bg))
            .show(ctx, |ui| self.canvas.show(ui, &self.brush, c.text_dim));

        match toolbar::show(ctx, &mut self.brush, &c) {
            Some(Action::Undo) => self.canvas.undo(),
            Some(Action::Clear) => {
                // Nothing to clear: skip the confirmation.
                if !self.canvas.is_empty() {
                    self.confirm_clear = true;
                }
            }
            Some(Action::Settings) => self.settings_open = !self.settings_open,
            Some(Action::Export) => self.export(ctx, c.canvas_bg),
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

        // Export error dialog.
        let error = self.export_error.clone().unwrap_or_default();
        let mut error_close = false;
        let error_dismissed = dialog::show(
            ctx,
            "export_error",
            self.export_error.is_some(),
            &c,
            300.0,
            |ui| {
                ui.label(RichText::new("Export failed").size(18.0).strong());
                ui.add_space(6.0);
                ui.label(RichText::new(error.as_str()).color(c.text_dim));
                ui.add_space(16.0);
                if widgets::button(ui, "export_error_close", &c, "Close", Kind::Soft, 300.0)
                    .clicked()
                {
                    error_close = true;
                }
            },
        );
        if error_close || error_dismissed {
            self.export_error = None;
        }

        // Update prompt: shown when a new version is available, while it downloads,
        // or when the install failed. It never stacks over another dialog: wait
        // until the others have fully faded out.
        let has_drawing = !self.canvas.is_empty();
        let other_dialog_visible = self.confirm_exit
            || dialog::is_visible(ctx, "settings")
            || dialog::is_visible(ctx, "confirm_clear")
            || dialog::is_visible(ctx, "confirm_exit")
            || dialog::is_visible(ctx, "export_error");
        let show_update = update_prompt && !other_dialog_visible;
        let mut install = false;
        let mut later = false;
        let dismissed_update = dialog::show(ctx, "update", show_update, &c, 300.0, |ui| {
            match &status {
                Status::Available(release) => {
                    ui.label(RichText::new("Update available").size(18.0).strong());
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new(format!("Painty {} is ready to install.", release.version))
                            .color(c.text_dim),
                    );
                    if has_drawing {
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(
                                "Your current drawing will be lost when Painty restarts.",
                            )
                            .color(c.danger)
                            .size(13.0),
                        );
                    }
                    ui.add_space(16.0);
                    ui.horizontal(|ui| {
                        if widgets::button(ui, "later", &c, "Later", Kind::Soft, 146.0).clicked() {
                            later = true;
                        }
                        let install_label = if has_drawing { "Install anyway" } else { "Install" };
                        if widgets::button(ui, "install", &c, install_label, Kind::Soft, 146.0)
                            .clicked()
                        {
                            install = true;
                        }
                    });
                }
                Status::Downloading(progress) => {
                    ui.label(RichText::new("Downloading update…").size(18.0).strong());
                    ui.add_space(12.0);
                    let fraction = progress.fraction();
                    // Indeterminate (animated) bar when the size is unknown.
                    let mut bar = egui::ProgressBar::new(fraction.unwrap_or(0.0))
                        .desired_width(300.0)
                        .animate(fraction.is_none());
                    if fraction.is_some() {
                        bar = bar.show_percentage();
                    }
                    ui.add(bar);
                }
                Status::InstallFailed(error) => {
                    ui.label(RichText::new("Update failed").size(18.0).strong());
                    ui.add_space(6.0);
                    ui.label(RichText::new(error.as_str()).color(c.text_dim));
                    ui.add_space(16.0);
                    if widgets::button(ui, "update_close", &c, "Close", Kind::Soft, 300.0)
                        .clicked()
                    {
                        later = true;
                    }
                }
                _ => {}
            }
        });

        if install {
            if let Status::Available(release) = self.updater.status.clone() {
                self.updater.install(ctx, release);
            }
        }
        // The prompt cannot be closed while the download is running.
        if later || (dismissed_update && !matches!(self.updater.status, Status::Downloading(_))) {
            self.updater.dismiss();
        }

        // "Quit without saving?" confirmation, when the drawing is not empty.
        let mut quit = false;
        let mut keep = false;
        let dismissed_exit = dialog::show(ctx, "confirm_exit", self.confirm_exit, &c, 300.0, |ui| {
            ui.label(RichText::new("Quit Painty?").size(18.0).strong());
            ui.add_space(6.0);
            ui.label(
                RichText::new("Your drawing will be lost. Export it first if you want to keep it.")
                    .color(c.text_dim),
            );
            ui.add_space(16.0);
            ui.horizontal(|ui| {
                if widgets::button(ui, "quit_cancel", &c, "Keep drawing", Kind::Soft, 146.0)
                    .clicked()
                {
                    keep = true;
                }
                if widgets::button(ui, "quit_confirm", &c, "Quit", Kind::Danger, 146.0).clicked() {
                    quit = true;
                }
            });
        });

        if quit {
            self.confirm_exit = false;
            self.close_confirmed = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if keep || dismissed_exit {
            self.confirm_exit = false;
        }
    }
}