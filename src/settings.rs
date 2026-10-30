use eframe::egui::{Context, RichText};

use crate::dialog;
use crate::theme::{Colors, Mode};
use crate::updater::{Status, Updater};
use crate::widgets::{self, Kind};

/// Settings dialog: theme, canvas background, updates and About section.
pub fn show(
    ctx: &Context,
    open: &mut bool,
    mode: &mut Mode,
    canvas_follows_theme: &mut bool,
    updater: &mut Updater,
    c: &Colors,
) {
    let mut close = false;

    let dismissed = dialog::show(ctx, "settings", *open, c, 380.0, |ui| {
        ui.label(RichText::new("Settings").size(18.0).strong());
        ui.add_space(16.0);

        ui.label(RichText::new("Appearance").color(c.text_dim));
        ui.add_space(6.0);
        widgets::segmented(
            ui,
            "theme",
            c,
            mode,
            &[(Mode::Light, "Light"), (Mode::Dark, "Dark")],
            147.0,
        );
        ui.add_space(12.0);

        ui.label(RichText::new("Canvas").color(c.text_dim));
        ui.add_space(6.0);
        widgets::segmented(
            ui,
            "canvas_follow",
            c,
            canvas_follows_theme,
            &[(false, "White"), (true, "Follow theme")],
            147.0,
        );
        ui.add_space(16.0);

        ui.label(RichText::new("Updates").color(c.text_dim));
        ui.add_space(6.0);
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
            ui.add_space(6.0);
        }
        if !busy
            && widgets::button(ui, "check_updates", c, "Check for updates", Kind::Soft, 300.0)
                .clicked()
        {
            updater.check(ctx, true);
        }
        ui.add_space(16.0);

        ui.label(RichText::new("About").color(c.text_dim));
        ui.add_space(4.0);
        ui.label(RichText::new("Painty").strong());
        ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));
        ui.label("Minimal paint app written in Rust with egui.");
        ui.add_space(16.0);

        if widgets::button(ui, "settings_close", c, "Close", Kind::Soft, 300.0).clicked() {
            close = true;
        }
    });

    if close || dismissed {
        *open = false;
    }
}