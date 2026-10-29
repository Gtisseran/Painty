use eframe::egui::{self, vec2, Align2, Context, Frame, Id, Label, Margin, RichText, Rounding, Stroke, Ui};

use crate::model::{Brush, Tool};
use crate::theme::{self, Colors};
use crate::widgets::{self, Kind};

pub enum Action {
    Undo,
    Clear,
    Settings,
}

enum Layout {
    Wide,
    Medium,
    Narrow,
}

impl Layout {
    fn for_width(w: f32) -> Self {
        if w >= 960.0 {
            Layout::Wide
        } else if w >= 640.0 {
            Layout::Medium
        } else {
            Layout::Narrow
        }
    }
}

pub fn show(ctx: &Context, brush: &mut Brush, c: &Colors) -> Option<Action> {
    let layout = Layout::for_width(ctx.screen_rect().width());
    let mut action = None;

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

                    match layout {
                        Layout::Wide => {
                            ui.horizontal(|ui| {
                                colors_group(ui, brush, c, true);
                                widgets::divider(ui, c);
                                size_group(ui, brush, c, 180.0, true);
                                widgets::divider(ui, c);
                                tool_group(ui, brush, c);
                                widgets::divider(ui, c);
                                action_group(ui, c, &mut action);
                            });
                        }
                        Layout::Medium => {
                            ui.horizontal(|ui| {
                                colors_group(ui, brush, c, false);
                                widgets::divider(ui, c);
                                size_group(ui, brush, c, 120.0, false);
                                widgets::divider(ui, c);
                                tool_group(ui, brush, c);
                                widgets::divider(ui, c);
                                action_group(ui, c, &mut action);
                            });
                        }
                        Layout::Narrow => {
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    colors_group(ui, brush, c, true);
                                    size_group(ui, brush, c, 150.0, false);
                                });
                                ui.horizontal(|ui| {
                                    tool_group(ui, brush, c);
                                    action_group(ui, c, &mut action);
                                });
                            });
                        }
                    }
                });
        });

    action
}

fn colors_group(ui: &mut Ui, brush: &mut Brush, c: &Colors, swatches: bool) {
    if swatches {
        for (i, color) in theme::palette(c.ink).iter().enumerate() {
            let selected = brush.tool == Tool::Brush && brush.color == *color;
            if widgets::swatch(ui, i, *color, selected, c).clicked() {
                brush.color = *color;
                brush.tool = Tool::Brush;
            }
        }
    }
    ui.color_edit_button_srgba(&mut brush.color);
}

fn size_group(ui: &mut Ui, brush: &mut Brush, c: &Colors, slider_w: f32, show_px: bool) {
    ui.spacing_mut().slider_width = slider_w;
    ui.add(
        egui::Slider::new(&mut brush.width, 1.0..=80.0)
            .show_value(false)
            .trailing_fill(true),
    );
    if show_px {
        ui.add_sized(
            [44.0, 20.0],
            Label::new(RichText::new(format!("{:.0} px", brush.width)).color(c.text_dim)),
        );
    }
}

fn tool_group(ui: &mut Ui, brush: &mut Brush, c: &Colors) {
    widgets::segmented(
        ui,
        "tool",
        c,
        &mut brush.tool,
        &[(Tool::Brush, "Pinceau"), (Tool::Eraser, "Gomme")],
        68.0,
    );
}

fn action_group(ui: &mut Ui, c: &Colors, action: &mut Option<Action>) {
    if widgets::button(ui, "undo", c, "Annuler", Kind::Ghost, 0.0).clicked() {
        *action = Some(Action::Undo);
    }
    if widgets::button(ui, "clear", c, "Effacer", Kind::Ghost, 0.0).clicked() {
        *action = Some(Action::Clear);
    }
    if widgets::settings_button(ui, c).clicked() {
        *action = Some(Action::Settings);
    }
}