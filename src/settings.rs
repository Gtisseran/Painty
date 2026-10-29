use eframe::egui::{Context, RichText};

use crate::dialog;
use crate::theme::{Colors, Mode};
use crate::widgets::{self, Kind};

pub fn show(
    ctx: &Context,
    open: &mut bool,
    mode: &mut Mode,
    canvas_follows_theme: &mut bool,
    c: &Colors,
) {
    let mut close = false;

    let dismissed = dialog::show(ctx, "settings", *open, c, |ui| {
        ui.label(RichText::new("Paramètres").size(18.0).strong());
        ui.add_space(16.0);

        ui.label(RichText::new("Apparence").color(c.text_dim));
        ui.add_space(6.0);
        widgets::segmented(
            ui,
            "theme",
            c,
            mode,
            &[(Mode::Light, "Clair"), (Mode::Dark, "Sombre")],
            147.0,
        );
        ui.add_space(12.0);

        ui.label(RichText::new("Zone de dessin").color(c.text_dim));
        ui.add_space(6.0);
        widgets::segmented(
            ui,
            "canvas_follow",
            c,
            canvas_follows_theme,
            &[(false, "Blanche"), (true, "Suit le thème")],
            147.0,
        );
        ui.add_space(16.0);

        ui.label(RichText::new("À propos").color(c.text_dim));
        ui.add_space(4.0);
        ui.label(RichText::new("Painty").strong());
        ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));
        ui.label("Paint minimaliste en Rust, fait avec egui.");
        ui.add_space(16.0);

        if widgets::button(ui, "settings_close", c, "Fermer", Kind::Soft, 300.0).clicked() {
            close = true;
        }
    });

    if close || dismissed {
        *open = false;
    }
}