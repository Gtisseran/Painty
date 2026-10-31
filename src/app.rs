use eframe::egui::{self, Color32, Frame, Key, RichText};
use std::time::{Duration, Instant};

use crate::canvas::Canvas;
use crate::config::{self, Config};
use crate::dialog;
use crate::export::{self, Format};
use crate::i18n::{self, t, tf, Lang};
use crate::model::Brush;
use crate::settings;
use crate::theme::{self, CanvasColor, Mode};
use crate::toolbar::{self, Action};
use crate::updater::{self, Status, Updater};
use crate::widgets::{self, Kind};

/// Export options chosen in the export popup.
#[derive(Clone)]
struct ExportForm {
    name: String,
    format: Format,
}

impl Default for ExportForm {
    fn default() -> Self {
        Self {
            name: "painty".to_owned(),
            format: Format::Png,
        }
    }
}

/// The single modal dialog shown at a time. `Update` stays status-driven (see `update`).
#[derive(Clone, Copy, PartialEq, Eq)]
enum Modal {
    None,
    Settings,
    ClearConfirm,
    Export,
    Error,
    Quit,
}

/// True on the first frame a dialog is open, used to put focus on its safe action.
fn opening_frame(ctx: &egui::Context, id: &str, open: bool) -> bool {
    let key = egui::Id::new(("dialog_opening", id));
    let was_open = ctx.data(|d| d.get_temp::<bool>(key)).unwrap_or(false);
    ctx.data_mut(|d| d.insert_temp(key, open));
    open && !was_open
}

/// Main application state, updated every frame by egui.
pub struct Painty {
    canvas: Canvas,
    brush: Brush,
    mode: Mode,
    /// Background of the drawing area, independent from the interface theme.
    canvas_color: CanvasColor,
    /// True when the theme was chosen in the settings; false follows the system.
    theme_override: bool,
    /// Last time the system theme was read, while following it.
    last_system_check: Instant,
    language: Lang,
    /// Ink color used on the previous frame, to detect a background change.
    last_ink: Option<Color32>,
    /// Settings as last written to disk, to avoid saving every frame.
    last_saved: Config,
    /// The modal dialog currently requested.
    modal: Modal,
    /// Kept outside `Modal` so the dialog content can still be drawn while it fades out.
    export_form: ExportForm,
    /// Message shown by the error dialog, kept for the same reason.
    error_text: String,
    /// True once the window may close without asking (user confirmed or update restart).
    close_confirmed: bool,
    updater: Updater,
    /// True once the automatic update check has been started.
    update_checked: bool,
    /// True once the window has been centered on the monitor.
    centered: bool,
}

impl Painty {
    /// Loads the saved settings and builds the initial state.
    pub fn new() -> Self {
        // Remove files left over from the previous update.
        updater::cleanup();

        let cfg = config::load();
        i18n::set_lang(cfg.language);
        let mode = cfg.theme.unwrap_or_else(theme::system_mode);
        let ink = mode.colors(cfg.canvas).ink;

        let mut brush = Brush::default();
        brush.color = ink;

        Self {
            canvas: Canvas::default(),
            brush,
            mode,
            canvas_color: cfg.canvas,
            theme_override: cfg.theme.is_some(),
            last_system_check: Instant::now(),
            language: cfg.language,
            last_ink: Some(ink),
            last_saved: cfg,
            modal: Modal::None,
            export_form: ExportForm::default(),
            error_text: String::new(),
            close_confirmed: false,
            updater: Updater::default(),
            update_checked: false,
            centered: false,
        }
    }

    /// Opens the export popup, unless there is nothing to draw on yet.
    fn open_export(&mut self) {
        if self.canvas.rect().is_some() {
            self.export_form = ExportForm::default();
            self.modal = Modal::Export;
        }
    }

    /// Asks where to save, then writes the drawing in the chosen format.
    /// Returns the error message if the export failed.
    fn save_export(&mut self, ctx: &egui::Context, form: ExportForm, bg: Color32) -> Option<String> {
        let rect = self.canvas.rect()?;
        // Export the whole drawing, even when it no longer fits in the window.
        // An empty canvas falls back to the visible area.
        let (origin, size) = export::bounds(self.canvas.lines())
            .map(|bounds| (bounds.min, bounds.size()))
            .unwrap_or((rect.min, rect.size()));

        // Remove characters that are not allowed in file names.
        let name: String = form
            .name
            .trim()
            .chars()
            .map(|c| if "\\/:*?\"<>|".contains(c) { '_' } else { c })
            .collect();
        let name = if name.is_empty() { "painty".to_owned() } else { name };
        let ext = form.format.ext();

        // Dialog cancelled: nothing to do.
        let mut path = rfd::FileDialog::new()
            .add_filter(form.format.label(), &[ext])
            .set_file_name(format!("{name}.{ext}"))
            .save_file()?;

        // Add the extension if the user typed a name without the right one.
        let has_ext = path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| e.eq_ignore_ascii_case(ext));
        if !has_ext {
            let mut os = path.into_os_string();
            os.push(format!(".{ext}"));
            path = os.into();
        }

        export::save(
            &path,
            form.format,
            self.canvas.lines(),
            origin,
            size,
            bg,
            ctx.pixels_per_point(),
        )
        .err()
    }
}

impl eframe::App for Painty {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Center the window once, at a reasonable size (at most 900x600, 70 % of the screen).
        if !self.centered {
            let monitor = ctx.input(|i| i.viewport().monitor_size);
            if let Some(monitor) = monitor {
                let size = egui::vec2(900.0, 600.0).min(monitor * 0.7);
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
                ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(
                    ((monitor - size) * 0.5).to_pos2(),
                ));
                self.centered = true;
            }
        }

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
                self.modal = Modal::Quit;
            }
        } else {
            self.close_confirmed = false;
        }
        if self.updater.restart {
            self.updater.restart = false;
            self.close_confirmed = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        // Follow the Windows theme while no explicit choice was saved. The
        // periodic repaint also notices a theme change made outside the app.
        if !self.theme_override {
            ctx.request_repaint_after(Duration::from_secs(2));
            if self.last_system_check.elapsed() >= Duration::from_secs(2) {
                self.last_system_check = Instant::now();
                self.mode = theme::system_mode();
            }
        }

        let canvas = self.canvas_color;
        let c = self.mode.colors(canvas);

        // When the canvas background changes, the default ink follows (black <-> white).
        if let Some(prev) = self.last_ink {
            if prev != c.ink && self.brush.color == prev {
                self.brush.color = c.ink;
            }
        }
        self.last_ink = Some(c.ink);

        // The update prompt is shown from the updater status, not from `Modal`.
        let status = self.updater.status.clone();
        let update_prompt = matches!(
            status,
            Status::Available(_) | Status::Downloading(_) | Status::InstallFailed(_)
        );

        // Keyboard shortcuts, unless a dialog is open.
        let modal_open = self.modal != Modal::None || update_prompt;
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
                self.open_export();
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
                    self.modal = Modal::ClearConfirm;
                }
            }
            Some(Action::Settings) => {
                self.modal = if self.modal == Modal::Settings {
                    Modal::None
                } else {
                    Modal::Settings
                };
            }
            Some(Action::Export) => self.open_export(),
            None => {}
        }

        // Settings dialog. It keeps its own open flag, synced with `Modal`.
        let mut settings_open = self.modal == Modal::Settings;
        let previous_mode = self.mode;
        settings::show(
            ctx,
            &mut settings_open,
            &mut self.mode,
            &mut self.canvas_color,
            &mut self.language,
            &mut self.updater,
            &c,
        );
        // Picking a theme stops following the system, and is saved.
        if self.mode != previous_mode {
            self.theme_override = true;
        }
        if !settings_open && self.modal == Modal::Settings {
            self.modal = Modal::None;
        }

        // Persist settings as soon as they change.
        let current = Config {
            theme: if self.theme_override {
                Some(self.mode)
            } else {
                None
            },
            canvas: self.canvas_color,
            language: self.language,
        };
        if current != self.last_saved {
            config::save(&current);
            self.last_saved = current;
        }

        // "Delete drawing" confirmation dialog: 340 px card, safe action focused first.
        let mut confirm = false;
        let mut cancel = false;
        let clear_open = self.modal == Modal::ClearConfirm;
        let focus_cancel = opening_frame(ctx, "confirm_clear", clear_open);
        let dismissed_clear = dialog::show(ctx, "confirm_clear", clear_open, &c, 340.0, c.surface, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
            widgets::title(ui, &c, t("dialog.clear.title"));
            ui.add_space(10.0);
            ui.label(RichText::new(t("dialog.clear.body")).size(14.0).color(c.text_dim));
            ui.add_space(24.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                let keep = widgets::button(ui, "keep", &c, t("button.cancel"), Kind::Soft, 141.0);
                if focus_cancel {
                    keep.request_focus();
                }
                if keep.clicked() {
                    cancel = true;
                }
                if widgets::button(ui, "wipe", &c, t("button.delete"), Kind::Danger, 141.0)
                    .clicked()
                {
                    confirm = true;
                }
            });
        });

        if confirm {
            self.canvas.clear();
        }
        if confirm || cancel || dismissed_clear {
            if self.modal == Modal::ClearConfirm {
                self.modal = Modal::None;
            }
        }

        // Export popup: name and format. The file location is asked afterwards.
        let mut export_go = false;
        let mut export_cancel = false;
        let export_open = self.modal == Modal::Export;
        let dismissed_export = dialog::show(ctx, "export_form", export_open, &c, 380.0, c.surface, |ui| {
            let form = &mut self.export_form;
            ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

            widgets::title(ui, &c, t("dialog.export.title"));
            ui.add_space(4.0);
            ui.add_space(14.0);

            ui.label(RichText::new(t("export.name")).size(13.0).color(c.text_dim));
            ui.add_space(14.0);
            widgets::text_field(ui, "export_name", &c, &mut form.name, "painty");
            ui.add_space(14.0);

            ui.label(RichText::new(t("export.format")).size(13.0).color(c.text_dim));
            ui.add_space(14.0);
            widgets::segmented_sized(
                ui,
                "export_format",
                &c,
                &mut form.format,
                &[(Format::Png, "PNG"), (Format::Jpeg, "JPEG")],
                162.0,
                41.0,
                14.0,
            );
            ui.add_space(14.0 + 6.0);

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                if widgets::button(ui, "export_cancel", &c, t("button.cancel"), Kind::Soft, 161.0)
                    .clicked()
                {
                    export_cancel = true;
                }
                if widgets::button(ui, "export_go", &c, t("button.export"), Kind::Primary, 161.0)
                    .clicked()
                {
                    export_go = true;
                }
            });
        });

        if export_go {
            let form = self.export_form.clone();
            self.modal = Modal::None;
            if let Some(error) = self.save_export(ctx, form, c.canvas_bg) {
                self.error_text = error;
                self.modal = Modal::Error;
            }
        }
        if export_cancel || dismissed_export {
            if self.modal == Modal::Export {
                self.modal = Modal::None;
            }
        }

        // Export error dialog.
        let error = self.error_text.clone();
        let mut error_close = false;
        let error_open = self.modal == Modal::Error;
        let dismissed_error = dialog::show(ctx, "export_error", error_open, &c, 360.0, c.surface, |ui| {
            ui.label(RichText::new(t("dialog.export_error.title")).size(18.0).strong());
            ui.add_space(6.0);
            ui.label(RichText::new(error.as_str()).color(c.text_dim));
            ui.add_space(16.0);
            if widgets::button(ui, "export_error_close", &c, t("button.close"), Kind::Soft, 312.0)
                .clicked()
            {
                error_close = true;
            }
        });
        if error_close || dismissed_error {
            if self.modal == Modal::Error {
                self.modal = Modal::None;
            }
        }

        // Update prompt: shown when a new version is available, while it downloads,
        // or when the install failed. It waits until every other dialog has faded out.
        let has_drawing = !self.canvas.is_empty();
        let other_dialog_visible = self.modal != Modal::None
            || ["settings", "confirm_clear", "export_form", "export_error", "confirm_exit"]
                .iter()
                .any(|id| dialog::is_visible(ctx, id));
        let show_update = update_prompt && !other_dialog_visible;
        let mut install = false;
        let mut later = false;
        let dismissed_update = dialog::show(ctx, "update", show_update, &c, 380.0, c.surface, |ui| {
            match &status {
                Status::Available(release) => {
                    ui.label(RichText::new(t("update.title")).size(18.0).strong());
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new(tf("update.ready", &[("version", release.version.as_str())]))
                            .color(c.text_dim),
                    );
                    if has_drawing {
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(t("update.lost_warning"))
                                .color(c.danger)
                                .size(13.0),
                        );
                    }
                    ui.add_space(16.0);
                    ui.horizontal(|ui| {
                        if widgets::button(ui, "later", &c, t("button.later"), Kind::Soft, 146.0)
                            .clicked()
                        {
                            later = true;
                        }
                        let install_label = if has_drawing {
                            t("button.install_anyway")
                        } else {
                            t("button.install")
                        };
                        if widgets::button(ui, "install", &c, install_label, Kind::Soft, 146.0)
                            .clicked()
                        {
                            install = true;
                        }
                    });
                }
                Status::Downloading(progress) => {
                    ui.label(RichText::new(t("update.downloading")).size(18.0).strong());
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
                    ui.label(RichText::new(t("update.failed_title")).size(18.0).strong());
                    ui.add_space(6.0);
                    ui.label(RichText::new(error.as_str()).color(c.text_dim));
                    ui.add_space(16.0);
                    if widgets::button(ui, "update_close", &c, t("button.close"), Kind::Soft, 332.0)
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
        let quit_open = self.modal == Modal::Quit;
        let focus_keep = opening_frame(ctx, "confirm_exit", quit_open);
        let dismissed_quit = dialog::show(ctx, "confirm_exit", quit_open, &c, 340.0, c.surface, |ui| {
            ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);
            widgets::title(ui, &c, t("dialog.quit.title"));
            ui.add_space(10.0);
            ui.label(RichText::new(t("dialog.quit.body")).size(14.0).color(c.text_dim));
            ui.add_space(24.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                let stay = widgets::button(
                    ui,
                    "quit_cancel",
                    &c,
                    t("button.keep_drawing"),
                    Kind::Soft,
                    141.0,
                );
                if focus_keep {
                    stay.request_focus();
                }
                if stay.clicked() {
                    keep = true;
                }
                if widgets::button(ui, "quit_confirm", &c, t("button.quit"), Kind::Danger, 141.0)
                    .clicked()
                {
                    quit = true;
                }
            });
        });

        if quit {
            self.modal = Modal::None;
            self.close_confirmed = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        if keep || dismissed_quit {
            if self.modal == Modal::Quit {
                self.modal = Modal::None;
            }
        }
    }
}