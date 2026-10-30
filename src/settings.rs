use eframe::egui::{Context, RichText};

use crate::dialog;
use crate::theme::{Colors, Mode};
use crate::updater::{Status, Updater};
use crate::widgets::{self, Kind};

/// Preferred width of the settings popup. It shrinks when the window is narrower.
const WIDTH: f32 = 360.0;
/// Padding around the segments, as drawn by `widgets::segmented`.
const SEG_PADDING: f32 = 3.0;

/// Settings dialog: theme, canvas background, updates and a short About line.
pub fn show(
    ctx: &Context,
    open: &mut bool,
    mode: &mut Mode,
    canvas_follows_theme: &mut bool,
    updater: &mut Updater,
    c: &Colors,
) {
    let mut close = false;

    let dismissed = dialog::show(ctx, "settings", *open, c, WIDTH, |ui| {
        // Width the dialog can actually give us: it shrinks with the window.
        let width = ui.available_width();
        // Both segments share that width, minus the padding around them.
        let seg_w = ((width - 2.0 * SEG_PADDING) / 2.0).max(1.0);

        ui.label(RichText::new("Settings").size(18.0).strong());
        ui.add_space(12.0);

        ui.label(RichText::new("Appearance").color(c.text_dim));
        ui.add_space(4.0);
        widgets::segmented(
            ui,
            "theme",
            c,
            mode,
            &[(Mode::Light, "Light"), (Mode::Dark, "Dark")],
            seg_w,
        );
        ui.add_space(10.0);

        ui.label(RichText::new("Canvas").color(c.text_dim));
        ui.add_space(4.0);
        widgets::segmented(
            ui,
            "canvas_follow",
            c,
            canvas_follows_theme,
            &[(false, "White"), (true, "Follow theme")],
            seg_w,
        );
        ui.add_space(10.0);

        ui.label(RichText::new("Updates").color(c.text_dim));
        ui.add_space(4.0);
        let busy = matches!(updater.status, Status::Checking | Status::Downloading);
        let text = match &updater.status {
            Status::Idle => String::new(),
            Status::Checking => "Checking for updates…".to_owned(),
            Status::UpToDate => "You have the latest version.".to_owned(),
            Status::Available(release) => format!("Version {} is available.", release.version),
            Status::Downloading => "Downloading update…".to_owned(),
            Status::CheckFailed(error) => format!("Could not check for updates: {error}"),
            Status::InstallFailed(error) => format!("Update failed: {error}"),
        };
        if !text.is_empty() {
            ui.label(RichText::new(text).color(c.text_dim).size(13.0));
            ui.add_space(4.0);
        }
        if !busy
            && widgets::button(ui, "check_updates", c, "Check for updates", Kind::Soft, width)
                .clicked()
        {
            updater.check(ctx, true);
        }
        ui.add_space(14.0);

        ui.label(
            RichText::new(format!("Painty {} · MIT License", env!("CARGO_PKG_VERSION")))
                .color(c.text_dim)
                .size(13.0),
        );
        ui.add_space(10.0);

        if widgets::button(ui, "settings_close", c, "Close", Kind::Soft, width).clicked() {
            close = true;
        }
    });

    if close || dismissed {
        *open = false;
    }
}