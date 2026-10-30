use eframe::egui::{self, vec2, Align2, Context, Frame, Id, Margin, Rounding, Stroke};

use crate::model::{Brush, Tool};
use crate::theme::{self, Colors};
use crate::widgets::{self, Icon, Kind};

/// Actions the toolbar can request from the app.
pub enum Action {
    Undo,
    Clear,
    Settings,
    Export,
}

/// Smooth start and end for animations (0.0 -> 1.0).
fn ease(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// Draws the floating toolbar at the bottom of the window.
/// Returns the action clicked this frame, if any.
///
/// The toolbar stays on one row. Elements fold in and out smoothly as the
/// window width crosses a threshold:
/// - >= 1000 px: color swatches and the size in pixels
/// - >= 640 px:  separators and text buttons (below: icon buttons)
pub fn show(ctx: &Context, brush: &mut Brush, c: &Colors) -> Option<Action> {
    let w = ctx.screen_rect().width();
    let mut action = None;

    // Each value animates from 0.0 to 1.0 when its condition becomes true.
    let anim = |name: &str, on: bool| {
        ease(ctx.animate_bool_with_time(Id::new(("tb_vis", name)), on, 0.22))
    };
    let wide = anim("wide", w >= 1000.0);
    let full = anim("full", w >= 640.0);
    let compact = anim("compact", w < 640.0);

    // The slider follows the window width continuously.
    let slider_w = ctx.animate_value_with_time(
        Id::new("tb_slider_w"),
        (w * 0.18).clamp(70.0, 200.0),
        0.12,
    );

    egui::Area::new(Id::new("toolbar"))
        .anchor(Align2::CENTER_BOTTOM, vec2(0.0, -20.0))
        .show(ctx, |ui| {
            theme::apply(ui, c);

            Frame::none()
                .fill(c.bar_bg)
                .stroke(Stroke::new(1.0_f32, c.bar_border))
                .rounding(Rounding::same(18.0))
                .inner_margin(Margin::symmetric(12.0, 8.0))
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing = vec2(8.0, 6.0);
                    ui.spacing_mut().interact_size = vec2(28.0, 28.0);
                    ui.spacing_mut().slider_width = slider_w;

                    ui.horizontal(|ui| {
                        // Colors
                        for (i, color) in theme::palette(c.ink).iter().enumerate() {
                            let selected = brush.tool == Tool::Brush && brush.color == *color;
                            if widgets::swatch(ui, i, *color, selected, c, wide).clicked() {
                                brush.color = *color;
                                brush.tool = Tool::Brush;
                            }
                        }
                        // Opaque only: PNG export paints strokes over the
                        // background and does not composite transparency yet.
                        egui::widgets::color_picker::color_edit_button_srgba(
                            ui,
                            &mut brush.color,
                            egui::widgets::color_picker::Alpha::Opaque,
                        );

                        widgets::divider(ui, c, full);

                        // Size
                        ui.add(
                            egui::Slider::new(&mut brush.width, 1.0..=80.0)
                                .show_value(false)
                                .trailing_fill(true),
                        );
                        widgets::px_label(ui, c, &format!("{:.0} px", brush.width), wide);

                        widgets::divider(ui, c, full);

                        // Tool
                        widgets::segmented(
                            ui,
                            "tool",
                            c,
                            &mut brush.tool,
                            &[(Tool::Brush, "Brush"), (Tool::Eraser, "Eraser")],
                            68.0,
                        );

                        widgets::divider(ui, c, full);

                        // Actions: text buttons on wide windows, icons on narrow ones.
                        if widgets::button_anim(ui, "undo", c, "Undo", Kind::Ghost, 0.0, full)
                            .clicked()
                        {
                            action = Some(Action::Undo);
                        }
                        if widgets::button_anim(ui, "clear", c, "Clear", Kind::Ghost, 0.0, full)
                            .clicked()
                        {
                            action = Some(Action::Clear);
                        }
                        if widgets::icon_button(ui, "undo", c, Icon::Undo, "Undo", compact)
                            .clicked()
                        {
                            action = Some(Action::Undo);
                        }
                        if widgets::icon_button(ui, "clear", c, Icon::Trash, "Clear", compact)
                            .clicked()
                        {
                            action = Some(Action::Clear);
                        }

                        if widgets::icon_button(ui, "export", c, Icon::Export, "Export PNG (Ctrl+S)", 1.0).clicked() {
                            action = Some(Action::Export);
                        }

                        if widgets::settings_button(ui, c).clicked() {
                            action = Some(Action::Settings);
                        }
                    });
                });
        });

    action
}