use eframe::egui::{
    self, vec2, Align2, Color32, Context, Frame, Id, Key, Margin, Order, Rounding, ScrollArea,
    Sense, Stroke, Ui,
};

use crate::theme::{self, Colors};

/// Animated modal popup with a dimmed background and a slide-in effect.
/// `width` is the popup width in pixels.
/// Returns `true` when the user asks to close it (click on the background or Escape).
pub fn show(
    ctx: &Context,
    id: &str,
    open: bool,
    c: &Colors,
    width: f32,
    content: impl FnOnce(&mut Ui),
) -> bool {
    // Fade and slide progress, from 0.0 (closed) to 1.0 (open).
    let t = ctx.animate_bool_with_time(Id::new(("dialog_anim", id)), open, 0.16);
    if t <= 0.0 {
        return false;
    }

    let mut dismissed = false;
    let screen = ctx.screen_rect();

    // Dimmed background. A click on it closes the dialog.
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

    // The screen size is part of the IDs: when the window is resized, egui starts
    // measuring the dialog and its scroll area again instead of reusing the old size.
    let size_key = (screen.width().round() as i32, screen.height().round() as i32);
    // Leave a small margin around the popup, so it uses most of the window height.
    let max_h = (screen.height() - 40.0).max(120.0);

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
                    ui.set_width(width);
                    // Scrolls only when the window is too short for the content.
                    ScrollArea::vertical()
                        .id_salt(("dialog_scroll", id, size_key))
                        .max_height(max_h)
                        .auto_shrink([false, true])
                        .show(ui, |ui| {
                            ui.set_width(width);
                            content(ui);
                        });
                });
        });

    if open && ctx.input(|i| i.key_pressed(Key::Escape)) {
        dismissed = true;
    }

    dismissed
}