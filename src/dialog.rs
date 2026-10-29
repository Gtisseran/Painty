use eframe::egui::{
    self, vec2, Align2, Color32, Context, Frame, Id, Key, Margin, Order, Rounding, ScrollArea,
    Sense, Stroke, Ui,
};

use crate::theme::{self, Colors};

/// Popup modale animée (voile + glissement). Renvoie `true` si l'utilisateur
/// demande la fermeture (clic sur le voile ou Échap).
pub fn show(
    ctx: &Context,
    id: &str,
    open: bool,
    c: &Colors,
    content: impl FnOnce(&mut Ui),
) -> bool {
    let t = ctx.animate_bool_with_time(Id::new(("dialog_anim", id)), open, 0.16);
    if t <= 0.0 {
        return false;
    }

    let mut dismissed = false;
    let screen = ctx.screen_rect();

    // Voile
    egui::Area::new(Id::new(("dialog_scrim", id)))
        .order(Order::Foreground)
        .fixed_pos(screen.min)
        .show(ctx, |ui| {
            let (rect, resp) = ui.allocate_exact_size(screen.size(), Sense::click_and_drag());
            ui.painter()
                .rect_filled(rect, 0.0, Color32::from_black_alpha((t * 110.0) as u8));
            if open && resp.clicked() {
                dismissed = true;
            }
        });

    // Taille de l'écran dans les identifiants : à chaque redimensionnement,
    // egui repart d'une mesure propre au lieu de garder l'ancienne taille.
    let size_key = (screen.width().round() as i32, screen.height().round() as i32);
    let max_h = (screen.height() - 100.0).max(120.0);

    egui::Area::new(Id::new(("dialog_box", id, size_key)))
        .order(Order::Tooltip)
        .anchor(Align2::CENTER_CENTER, vec2(0.0, (1.0 - t) * 14.0))
        .show(ctx, |ui| {
            theme::apply(ui, c);
            Frame::none()
                .fill(c.bar_bg)
                .stroke(Stroke::new(1.0_f32, c.bar_border))
                .rounding(Rounding::same(18.0))
                .inner_margin(Margin::same(20.0))
                .show(ui, |ui| {
                    ui.set_width(300.0);
                    ScrollArea::vertical()
                        .id_salt(("dialog_scroll", id, size_key))
                        .max_height(max_h)
                        .auto_shrink([false, true])
                        .show(ui, |ui| {
                            ui.set_width(300.0);
                            content(ui);
                        });
                });
        });

    if open && ctx.input(|i| i.key_pressed(Key::Escape)) {
        dismissed = true;
    }

    dismissed
}