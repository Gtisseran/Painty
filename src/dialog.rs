use eframe::egui::{
    self, vec2, Align2, Color32, Context, Frame, Id, Key, Margin, Order, Rounding, ScrollArea,
    Sense, Stroke, Ui,
};

use crate::theme::{self, Colors};

/// Space kept between the popup and the window edges.
const EDGE_MARGIN: f32 = 16.0;
/// Inner padding of the popup, on each side.
const INNER_PADDING: f32 = 20.0;

/// True while the dialog is on screen, including its fade-out.
/// Call it after `show` has run for the current frame.
pub fn is_visible(ctx: &Context, id: &str) -> bool {
    ctx.data(|d| d.get_temp::<f32>(Id::new(("dialog_visibility", id))))
        .is_some_and(|t| t > 0.0)
}

/// Animated modal popup with a dimmed background and a slide-in effect.
/// `width` is the preferred popup width in pixels. It is reduced if the window is
/// too narrow, and the popup is always kept inside the window.
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
    // Remembered so callers can check whether another dialog is still on screen.
    ctx.data_mut(|d| d.insert_temp(Id::new(("dialog_visibility", id)), t));
    if t <= 0.0 {
        return false;
    }

    let screen = ctx.screen_rect();
    let mut dismissed = false;

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

    // Never wider than the window allows, padding included.
    let max_w = (screen.width() - 2.0 * EDGE_MARGIN - 2.0 * INNER_PADDING).max(120.0);
    let width = width.min(max_w);
    // The popup must fit between the two margins, padding included.
    let max_h = (screen.height() - 2.0 * EDGE_MARGIN).max(120.0);
    let max_content_h = (max_h - 2.0 * INNER_PADDING).max(40.0);

    // The window size is part of the ids: when the window is resized, egui starts
    // measuring the popup and its scroll area again instead of reusing the old size.
    let size_key = (screen.width().round() as i32, screen.height().round() as i32);

    egui::Area::new(Id::new(("dialog_box", id, size_key)))
        .order(Order::Tooltip)
        .anchor(Align2::CENTER_CENTER, vec2(0.0, (1.0 - t) * 14.0))
        // Measure the content against the full height available, not a default size:
        // the popup then shrinks to its content or scrolls when it is too tall.
        .default_size(vec2(width + 2.0 * INNER_PADDING, max_h))
        .show(ctx, |ui| {
            theme::apply(ui, c);
            Frame::none()
                .fill(c.bar_bg)
                .stroke(Stroke::new(1.0_f32, c.bar_border))
                .rounding(Rounding::same(18.0))
                .inner_margin(Margin::same(INNER_PADDING))
                .show(ui, |ui| {
                    // The popup is exactly as wide as its content, which never exceeds the window.
                    ui.set_width(width);
                    // Scrolls only when the window is too short for the content.
                    ScrollArea::vertical()
                        .id_salt(("dialog_scroll", id, size_key))
                        .max_height(max_content_h)
                        .auto_shrink([false, true])
                        .show(ui, |ui| {
                            content(ui);
                        });
                });
        });

    if open && ctx.input(|i| i.key_pressed(Key::Escape)) {
        dismissed = true;
    }

    dismissed
}
