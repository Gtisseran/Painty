use eframe::egui::{
    self, vec2, Align2, Context, Frame, Id, Key, Margin, Rounding, Stroke, Ui,
};

use crate::i18n::t;
use crate::model::{Brush, Tool};
use crate::theme::{self, Colors};
use crate::widgets::{self, Icon};

/// Actions the toolbar can request from the app.
pub enum Action {
    Undo,
    Clear,
    Settings,
    Export,
}

/// Toolbar layouts from the mockups. The compact layout is also used while the
/// eraser is active; the eraser then adds the pixel size and dims the ink swatch.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Variant {
    Compact,
    Labels,
    Palette,
}

/// Draws the floating toolbar at the bottom of the window and returns the
/// action clicked this frame, if any.
///
/// The variant follows the window width: compact below 700 px, labels from
/// 700 px, and the two-row palette from 860 px. Activating the eraser switches
/// to the compact layout so the size stays visible next to the dimmed ink.
pub fn show(ctx: &Context, brush: &mut Brush, c: &Colors) -> Option<Action> {
    let width = ctx.screen_rect().width();
    let eraser = brush.tool == Tool::Eraser;
    let variant = if eraser || width < 700.0 {
        Variant::Compact
    } else if width < 860.0 {
        Variant::Labels
    } else {
        Variant::Palette
    };
    let gap = if width < 560.0 {
        4.0
    } else if width < 700.0 {
        6.0
    } else {
        10.0
    };
    let slider_w = if variant == Variant::Palette {
        120.0
    } else {
        (width * 0.17).clamp(if eraser { 44.0 } else { 52.0 }, 120.0)
    };

    let mut action = None;

    egui::Area::new(Id::new("toolbar"))
        .anchor(Align2::CENTER_BOTTOM, vec2(0.0, -24.0))
        .show(ctx, |ui| {
            theme::apply(ui, c);

            Frame::none()
                .fill(c.surface)
                .stroke(Stroke::new(1.0_f32, c.border))
                .rounding(Rounding::same(14.0))
                .inner_margin(Margin::symmetric(16.0, 10.0))
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing = vec2(gap, 10.0);
                    ui.spacing_mut().interact_size = vec2(32.0, 32.0);

                    match variant {
                        Variant::Compact => {
                            ui.horizontal_centered(|ui| {
                                current_color(ui, brush, c, !eraser);
                                size_group(ui, brush, c, slider_w, eraser);
                                tool_group(ui, brush, c);
                                actions(ui, c, &mut action, false);
                            });
                        }
                        Variant::Labels => {
                            ui.horizontal_centered(|ui| {
                                current_color(ui, brush, c, true);
                                size_group(ui, brush, c, slider_w, false);
                                tool_group(ui, brush, c);
                                actions(ui, c, &mut action, true);
                            });
                        }
                        Variant::Palette => {
                            ui.vertical(|ui| {
                                ui.spacing_mut().item_spacing = vec2(gap, 10.0);
                                palette_row(ui, brush, c, slider_w);
                                actions_row(ui, brush, c, &mut action, true);
                            });
                        }
                    }
                });
        });

    action
}

/// Current ink swatch. It opens the colour picker and switches back to the brush.
fn current_color(ui: &mut Ui, brush: &mut Brush, c: &Colors, enabled: bool) {
    let before = brush.color;
    widgets::color_button(ui, "toolbar_color", c, &mut brush.color, enabled);
    if enabled && brush.color != before {
        brush.tool = Tool::Brush;
    }
}

/// Custom slider, followed by the size in pixels where the mockups show it.
fn size_group(ui: &mut Ui, brush: &mut Brush, c: &Colors, width: f32, show_px: bool) {
    widgets::size_slider(
        ui,
        c,
        &mut brush.width,
        1.0..=80.0,
        width,
        t("toolbar.size"),
    );
    if show_px {
        widgets::px_label(ui, c, &format!("{:.0} px", brush.width));
    }
}

fn tool_group(ui: &mut Ui, brush: &mut Brush, c: &Colors) {
    widgets::segmented_sized(
        ui,
        "toolbar_tool",
        c,
        &mut brush.tool,
        &[(Tool::Brush, t("tool.brush")), (Tool::Eraser, t("tool.eraser"))],
        64.0,
        32.0,
        13.0,
    );
}

/// Palette variant, first row: six ink colours, custom colour, size and "N px".
fn palette_row(ui: &mut Ui, brush: &mut Brush, c: &Colors, slider_w: f32) {
    ui.horizontal(|ui| {
        let colors = theme::palette(c.ink);
        let mut next = None;
        for (i, color) in colors.iter().enumerate() {
            let selected = brush.tool == Tool::Brush && brush.color == *color;
            let resp = widgets::palette_swatch(ui, i, *color, selected, c);
            if resp.clicked() {
                brush.color = *color;
                brush.tool = Tool::Brush;
            }
            if resp.has_focus() {
                let (left, right) = ui.input(|input| {
                    (
                        input.key_pressed(Key::ArrowLeft),
                        input.key_pressed(Key::ArrowRight),
                    )
                });
                if left || right {
                    let step = if left { colors.len() - 1 } else { 1 };
                    next = Some((i + step) % colors.len());
                }
            }
        }
        if let Some(index) = next {
            brush.color = colors[index];
            brush.tool = Tool::Brush;
            ui.memory_mut(|m| m.request_focus(Id::new(("palette_swatch", index))));
        }

        widgets::divider(ui, c);

        let before = brush.color;
        widgets::color_button(ui, "palette_color", c, &mut brush.color, true);
        if brush.color != before {
            brush.tool = Tool::Brush;
        }

        size_group(ui, brush, c, slider_w, true);
    });
}

/// Tool group then action buttons. `labels` uses text for Undo/Clear and icons
/// for Export/Settings, as in the "Pinceau avec texte" and palette mockups.
fn actions(ui: &mut Ui, c: &Colors, action: &mut Option<Action>, labels: bool) {
    widgets::divider(ui, c);

    if labels {
        if widgets::toolbar_button(ui, "toolbar_undo", c, t("action.undo")).clicked() {
            *action = Some(Action::Undo);
        }
        if widgets::toolbar_button(ui, "toolbar_clear", c, t("action.clear")).clicked() {
            *action = Some(Action::Clear);
        }
        widgets::divider(ui, c);
        if widgets::icon_button(ui, "toolbar_export", c, Icon::Export, t("action.export")).clicked() {
            *action = Some(Action::Export);
        }
    } else {
        if widgets::icon_button(ui, "toolbar_undo", c, Icon::Undo, t("action.undo")).clicked() {
            *action = Some(Action::Undo);
        }
        if widgets::icon_button(ui, "toolbar_clear", c, Icon::Trash, t("action.clear")).clicked() {
            *action = Some(Action::Clear);
        }
        if widgets::icon_button(ui, "toolbar_export", c, Icon::Export, t("action.export")).clicked() {
            *action = Some(Action::Export);
        }
    }

    if widgets::icon_button(ui, "toolbar_settings", c, Icon::Settings, t("settings.title")).clicked()
    {
        *action = Some(Action::Settings);
    }
}

/// Palette and labels variants, second row: tool segment then actions.
fn actions_row(
    ui: &mut Ui,
    brush: &mut Brush,
    c: &Colors,
    action: &mut Option<Action>,
    labels: bool,
) {
    ui.horizontal(|ui| {
        tool_group(ui, brush, c);
        actions(ui, c, action, labels);
    });
}
