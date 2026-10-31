use eframe::egui::{self, Context, RichText};

use crate::dialog;
use crate::i18n::{self, t, tf, Lang};
use crate::theme::{Colors, Mode};
use crate::updater::{Status, Updater};
use crate::widgets::{self, Kind};

/// Preferred width of the settings popup. It shrinks when the window is narrower.
const WIDTH: f32 = 360.0;
/// Padding around the segments, as drawn by `widgets::segmented`.
const SEG_PADDING: f32 = 3.0;
/// Height of the update status line. Reserved even when empty, so the popup never changes height.
const STATUS_H: f32 = 18.0;

/// Settings dialog: theme, canvas background, language, updates and a short About line.
pub fn show(
    ctx: &Context,
    open: &mut bool,
    mode: &mut Mode,
    canvas_follows_theme: &mut bool,
    language: &mut Lang,
    updater: &mut Updater,
    c: &Colors,
) {
    let mut close = false;

    let dismissed = dialog::show(ctx, "settings", *open, c, WIDTH, |ui| {
        // Width the dialog can actually give us: it shrinks with the window.
        let width = ui.available_width();
        // Both segments share that width, minus the padding around them.
        let seg_w = ((width - 2.0 * SEG_PADDING) / 2.0).max(1.0);

        ui.label(RichText::new(t("settings.title")).size(18.0).strong());
        ui.add_space(12.0);

        ui.label(RichText::new(t("settings.appearance")).color(c.text_dim));
        ui.add_space(4.0);
        widgets::segmented(
            ui,
            "theme",
            c,
            mode,
            &[(Mode::Light, t("theme.light")), (Mode::Dark, t("theme.dark"))],
            seg_w,
        );
        ui.add_space(10.0);

        ui.label(RichText::new(t("settings.canvas")).color(c.text_dim));
        ui.add_space(4.0);
        widgets::segmented(
            ui,
            "canvas_follow",
            c,
            canvas_follows_theme,
            &[(false, t("canvas.white")), (true, t("canvas.follow"))],
            seg_w,
        );
        ui.add_space(10.0);

        ui.label(RichText::new(t("settings.language")).color(c.text_dim));
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 2.0 * SEG_PADDING;
            for lang in [Lang::Fr, Lang::En] {
                if widgets::language_button(ui, c, lang).clicked() {
                    *language = lang;
                    // Applies from the next frame, so every text switches at once.
                    i18n::set_lang(lang);
                }
            }
        });
        ui.add_space(10.0);

        ui.label(RichText::new(t("settings.updates")).color(c.text_dim));
        ui.add_space(4.0);
        let busy = matches!(updater.status, Status::Checking | Status::Downloading(_));
        let text = match &updater.status {
            Status::Idle => String::new(),
            Status::Checking => t("update.checking").to_owned(),
            Status::UpToDate => t("update.up_to_date").to_owned(),
            Status::Available(release) => {
                tf("update.available", &[("version", release.version.as_str())])
            }
            Status::Downloading(progress) => match progress.fraction() {
                Some(f) => tf(
                    "update.progress",
                    &[("percent", format!("{:.0}", f * 100.0).as_str())],
                ),
                None => t("update.downloading").to_owned(),
            },
            Status::CheckFailed(error) => tf("update.check_failed", &[("error", error.as_str())]),
            Status::InstallFailed(error) => {
                tf("update.install_failed", &[("error", error.as_str())])
            }
        };
        // Always reserves the same height, whatever the text.
        ui.add_sized(
            [width, STATUS_H],
            egui::Label::new(RichText::new(text).color(c.text_dim).size(13.0)),
        );
        ui.add_space(4.0);

        // The button is always drawn, only disabled while busy: its disappearance
        // used to shrink the popup and clip the rest of it.
        let check = ui
            .add_enabled_ui(!busy, |ui| {
                widgets::button(ui, "check_updates", c, t("button.check_updates"), Kind::Soft, width)
                    .clicked()
            })
            .inner;
        if check {
            updater.check(ctx, true);
        }
        ui.add_space(14.0);

        ui.label(
            RichText::new(tf(
                "settings.about",
                &[("version", env!("CARGO_PKG_VERSION"))],
            ))
            .color(c.text_dim)
            .size(13.0),
        );
        ui.add_space(10.0);

        if widgets::button(ui, "settings_close", c, t("button.close"), Kind::Soft, width).clicked() {
            close = true;
        }
    });

    if close || dismissed {
        *open = false;
    }
}