//! Atomic controls shared by the toolbar and the popups.
//!
//! Every control reads its colours from [`Colors`] and keeps a minimum 32×32
//! click target. Buttons and segmented controls can be reached with Tab and
//! activated with Space or Enter; the segmented control and the colour palette
//! also move with the left and right arrow keys.

use eframe::egui::{
    self, pos2, vec2, Align, Align2, Color32, CursorIcon, FontId, Id, Image, Key, Layout, Response,
    Rounding, Sense, Stroke, Ui, UiBuilder, WidgetInfo, WidgetType,
};

use crate::i18n::Lang;
use crate::theme::{self, Colors};

/// Visual style of a text button.
#[derive(Clone, Copy)]
pub enum Kind {
    /// Transparent until hovered.
    Ghost,
    /// Filled with the row colour: the secondary action of a popup.
    Soft,
    /// Inverted foreground/background: the main action of a popup.
    Primary,
    /// Red, for destructive actions.
    Danger,
}

/// Icons loaded from `assets/svg` (one file per theme).
#[derive(Clone, Copy)]
pub enum Icon {
    Undo,
    Trash,
    Export,
    Settings,
}

/// Shows a hand cursor over interactive elements.
fn pointer_cursor(ui: &Ui, resp: &Response) {
    if resp.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }
}

/// Text button used in popups. Primary actions use the inverted colour pair.
pub fn button(ui: &mut Ui, id: &str, c: &Colors, label: &str, kind: Kind, min_w: f32) -> Response {
    let (base, hover, fg) = match kind {
        Kind::Ghost => (Color32::TRANSPARENT, c.hover, c.text),
        Kind::Soft => (c.row, c.hover, c.text),
        Kind::Primary => (c.primary_bg, c.primary_bg.lerp_to_gamma(c.text_dim, 0.18), c.primary_fg),
        Kind::Danger => (c.danger, c.danger_hover, Color32::WHITE),
    };

    let font = if matches!(kind, Kind::Primary) {
        theme::semibold(14.0)
    } else {
        FontId::proportional(14.0)
    };
    let text_w = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font.clone(), fg)
        .size()
        .x;
    let size = vec2((text_w + 24.0).max(min_w), 40.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    if resp.clicked() {
        resp.request_focus();
    }

    let t = ui
        .ctx()
        .animate_bool_with_time(Id::new(("button_hover", id)), resp.hovered(), 0.12);
    let mut fill = if base.a() == 0 {
        hover.gamma_multiply(t)
    } else {
        base.lerp_to_gamma(hover, t)
    };
    if resp.is_pointer_button_down_on() {
        fill = hover;
    }

    let painter = ui.painter();
    painter.rect_filled(rect, Rounding::same(10.0), fill);
    painter.text(rect.center(), Align2::CENTER_CENTER, label, font, fg);

    pointer_cursor(ui, &resp);
    let enabled = ui.is_enabled();
    resp.widget_info(|| WidgetInfo::labeled(WidgetType::Button, enabled, label));
    resp
}

/// Compact text button used in the toolbar (32 px high, 13 px label).
pub fn toolbar_button(ui: &mut Ui, id: &str, c: &Colors, label: &str) -> Response {
    let font = FontId::proportional(13.0);
    let text_w = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font.clone(), c.text)
        .size()
        .x;
    let (rect, resp) = ui.allocate_exact_size(vec2(text_w + 20.0, 32.0), Sense::click());
    if resp.clicked() {
        resp.request_focus();
    }
    let t = ui
        .ctx()
        .animate_bool_with_time(Id::new(("toolbar_text_hover", id)), resp.hovered(), 0.12);
    let painter = ui.painter();
    painter.rect_filled(rect, Rounding::same(8.0), c.hover.gamma_multiply(t));
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        label,
        font,
        c.text,
    );
    pointer_cursor(ui, &resp);
    resp.widget_info(|| WidgetInfo::labeled(WidgetType::Button, ui.is_enabled(), label));
    resp
}

/// Square 32×32 icon button with a tooltip.
pub fn icon_button(ui: &mut Ui, id: &str, c: &Colors, icon: Icon, tip: &str) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(32.0, 32.0), Sense::click());
    if resp.clicked() {
        resp.request_focus();
    }
    let t = ui
        .ctx()
        .animate_bool_with_time(Id::new(("icon_hover", id)), resp.hovered(), 0.12);
    ui.painter()
        .rect_filled(rect, Rounding::same(8.0), c.hover.gamma_multiply(t));

    let icon_rect = eframe::egui::Rect::from_center_size(rect.center(), vec2(16.0, 16.0));
    let source = match (icon, c.dark) {
        (Icon::Undo, false) => egui::include_image!("../assets/svg/undo_dark.svg"),
        (Icon::Undo, true) => egui::include_image!("../assets/svg/undo_light.svg"),
        (Icon::Trash, false) => egui::include_image!("../assets/svg/trash_dark.svg"),
        (Icon::Trash, true) => egui::include_image!("../assets/svg/trash_light.svg"),
        (Icon::Export, false) => egui::include_image!("../assets/svg/export_dark.svg"),
        (Icon::Export, true) => egui::include_image!("../assets/svg/export_light.svg"),
        (Icon::Settings, false) => egui::include_image!("../assets/svg/settings_dark.svg"),
        (Icon::Settings, true) => egui::include_image!("../assets/svg/settings_light.svg"),
    };
    Image::new(source).paint_at(ui, icon_rect);

    pointer_cursor(ui, &resp);
    resp.widget_info(|| WidgetInfo::labeled(WidgetType::Button, ui.is_enabled(), tip));
    resp.on_hover_text(tip)
}

/// 28×28 colour button. Opens the egui colour picker above the toolbar.
/// When `enabled` is false the swatch is drawn at 40 % opacity and ignores clicks.
pub fn color_button(
    ui: &mut Ui,
    id: &str,
    c: &Colors,
    color: &mut Color32,
    enabled: bool,
) -> Response {
    let sense = if enabled { Sense::click() } else { Sense::hover() };
    let (rect, resp) = ui.allocate_exact_size(vec2(28.0, 28.0), sense);
    let opacity = if enabled { 1.0 } else { 0.4 };
    let painter = ui.painter();
    painter.rect_filled(
        rect,
        Rounding::same(6.0),
        color.gamma_multiply(opacity),
    );
    painter.rect_stroke(
        rect,
        Rounding::same(6.0),
        Stroke::new(1.0_f32, c.knob_border.gamma_multiply(opacity)),
    );

    if enabled {
        if resp.clicked() {
            resp.request_focus();
            ui.memory_mut(|m| m.toggle_popup(Id::new(("color_popup", id))));
        }
        let popup_id = Id::new(("color_popup", id));
        if ui.memory(|m| m.is_popup_open(popup_id)) {
            egui::popup_above_or_below_widget(
                ui,
                popup_id,
                &resp,
                egui::AboveOrBelow::Above,
                egui::PopupCloseBehavior::CloseOnClickOutside,
                |ui| {
                    ui.spacing_mut().slider_width = 235.0;
                    egui::color_picker::color_picker_color32(
                        ui,
                        color,
                        egui::color_picker::Alpha::Opaque,
                    );
                },
            );
        }
        pointer_cursor(ui, &resp);
    }

    resp.widget_info(|| WidgetInfo::labeled(WidgetType::ColorButton, enabled, "Couleur"));
    resp
}

/// 20 px round colour swatch for the palette variant. Selected swatches get a
/// double ring (surface gap + text colour) like the mockup.
pub fn palette_swatch(
    ui: &mut Ui,
    index: usize,
    color: Color32,
    selected: bool,
    c: &Colors,
) -> Response {
    let (rect, _) = ui.allocate_exact_size(vec2(20.0, 20.0), Sense::hover());
    let resp = ui.interact(
        rect,
        Id::new(("palette_swatch", index)),
        Sense::click(),
    );
    if resp.clicked() {
        resp.request_focus();
    }

    let hov = ui.ctx().animate_bool_with_time(
        Id::new(("palette_hover", index)),
        resp.hovered(),
        0.12,
    );
    let painter = ui.painter();
    let center = rect.center();
    let r = 10.0 + hov * 0.8;
    painter.circle_filled(center, r, color);
    painter.circle_stroke(center, r, Stroke::new(1.0_f32, c.border));
    if selected {
        painter.circle_stroke(center, 12.0, Stroke::new(2.0_f32, c.surface));
        painter.circle_stroke(center, 13.5, Stroke::new(1.5_f32, c.text));
    }

    pointer_cursor(ui, &resp);
    resp.widget_info(|| {
        WidgetInfo::selected(WidgetType::Button, ui.is_enabled(), selected, "Couleur")
    });
    resp
}

/// 4 px track with an 18 px thumb. The whole track is clickable and draggable;
/// arrow keys adjust the value when the slider has focus.
pub fn size_slider(
    ui: &mut Ui,
    c: &Colors,
    value: &mut f32,
    range: std::ops::RangeInclusive<f32>,
    width: f32,
    tip: &str,
) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(width, 32.0), Sense::click_and_drag());
    if resp.clicked() || resp.drag_started() {
        resp.request_focus();
    }

    let min = *range.start();
    let max = *range.end();
    let margin = 9.0;
    let track = eframe::egui::Rect::from_center_size(
        rect.center(),
        vec2((width - margin * 2.0).max(1.0), 4.0),
    );

    if let Some(pointer) = resp.interact_pointer_pos() {
        let t = ((pointer.x - track.left()) / track.width()).clamp(0.0, 1.0);
        *value = min + t * (max - min);
    } else if resp.has_focus() {
        let (left, right, shift) = ui.input(|i| {
            (
                i.key_pressed(Key::ArrowLeft),
                i.key_pressed(Key::ArrowRight),
                i.modifiers.shift,
            )
        });
        let step = if shift { 5.0 } else { 1.0 };
        if left {
            *value -= step;
        }
        if right {
            *value += step;
        }
    }
    *value = value.clamp(min, max);

    let t = (*value - min) / (max - min);
    let knob_x = track.left() + t * track.width();
    let painter = ui.painter();
    painter.rect_filled(track, Rounding::same(4.0), c.separator);
    if knob_x > track.left() {
        painter.rect_filled(
            eframe::egui::Rect::from_min_max(track.min, pos2(knob_x, track.max.y)),
            Rounding::same(4.0),
            c.accent,
        );
    }
    let knob = eframe::egui::Rect::from_center_size(pos2(knob_x, track.center().y), vec2(18.0, 18.0));
    painter.circle_filled(knob.center(), 9.0, c.knob);
    painter.circle_stroke(knob.center(), 9.0, Stroke::new(1.0_f32, c.knob_border));
    if resp.has_focus() {
        painter.circle_stroke(knob.center(), 11.0, Stroke::new(1.0_f32, c.accent));
    }

    if resp.hovered() || resp.dragged() {
        ui.ctx().set_cursor_icon(CursorIcon::ResizeHorizontal);
    }
    resp.widget_info(|| WidgetInfo::labeled(WidgetType::Slider, ui.is_enabled(), tip));
    resp.on_hover_text(tip)
}

/// Brush size in pixels, 13 px, with the mockup's minimum width.
pub fn px_label(ui: &mut Ui, c: &Colors, text: &str) {
    let (rect, _) = ui.allocate_exact_size(vec2(38.0, 32.0), Sense::hover());
    ui.painter().text(
        pos2(rect.min.x, rect.center().y),
        Align2::LEFT_CENTER,
        text,
        FontId::proportional(13.0),
        c.text_dim,
    );
}

/// 1×22 px separator between toolbar groups.
pub fn divider(ui: &mut Ui, c: &Colors) {
    let (rect, _) = ui.allocate_exact_size(vec2(1.0, 22.0), Sense::hover());
    ui.painter().rect_filled(rect, 0.0, c.separator);
}

/// Segmented control with a pill that slides to the selected option.
/// `font` also selects the control height: 13 px labels use 32 px when
/// `height` is 32, otherwise the caller's height is used.
pub fn segmented_sized<T: Copy + PartialEq>(
    ui: &mut Ui,
    id: &str,
    c: &Colors,
    value: &mut T,
    options: &[(T, &str)],
    seg_w: f32,
    height: f32,
    font_size: f32,
) -> bool {
    let pad = if height >= 38.0 { 4.0 } else { 3.0 };
    let size = vec2(seg_w * options.len() as f32 + pad * 2.0, height);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());

    let current = options.iter().position(|(v, _)| *v == *value).unwrap_or(0);
    let pos = ui
        .ctx()
        .animate_value_with_time(Id::new(("seg_pos", id)), current as f32, 0.16);

    let painter = ui.painter();
    painter.rect_filled(rect, Rounding::same(10.0), c.track);

    let pill = eframe::egui::Rect::from_min_size(
        pos2(rect.min.x + pad + seg_w * pos, rect.min.y + pad),
        vec2(seg_w, height - pad * 2.0),
    );
    painter.rect_filled(pill, Rounding::same(7.0), c.active);
    painter.rect_stroke(
        pill,
        Rounding::same(7.0),
        Stroke::new(1.0_f32, c.segment_border()),
    );

    let mut clicked = None;
    let mut keyboard_target = None;
    for (i, (v, label)) in options.iter().enumerate() {
        let cell = eframe::egui::Rect::from_min_size(
            pos2(rect.min.x + pad + seg_w * i as f32, rect.min.y + pad),
            vec2(seg_w, height - pad * 2.0),
        );
        let resp = ui.interact(cell, Id::new(("seg_cell", id, i)), Sense::click());
        if resp.clicked() {
            resp.request_focus();
        }
        if resp.has_focus() {
            let (left, right) = ui.input(|i| {
                (
                    i.key_pressed(Key::ArrowLeft),
                    i.key_pressed(Key::ArrowRight),
                )
            });
            if left || right {
                let next = if left {
                    (i + options.len() - 1) % options.len()
                } else {
                    (i + 1) % options.len()
                };
                keyboard_target = Some((next, options[next].0));
            }
        }

        let selected = *v == *value;
        let sel = (1.0 - (pos - i as f32).abs()).clamp(0.0, 1.0);
        let hover = if resp.hovered() { 0.6 } else { 0.0 };
        let color = c.text_dim.lerp_to_gamma(c.text, sel.max(hover));
        painter.text(
            cell.center(),
            Align2::CENTER_CENTER,
            *label,
            FontId::proportional(font_size),
            color,
        );

        pointer_cursor(ui, &resp);
        resp.widget_info(|| {
            WidgetInfo::selected(WidgetType::Button, ui.is_enabled(), selected, *label)
        });
        if resp.clicked() {
            clicked = Some(*v);
        }
    }

    if let Some((index, v)) = keyboard_target {
        clicked = Some(v);
        ui.memory_mut(|m| m.request_focus(Id::new(("seg_cell", id, index))));
    }
    if let Some(v) = clicked {
        let changed = *value != v;
        *value = v;
        return changed;
    }
    false
}

/// Segmented control used by the settings rows (13 px labels, 34 px high).
pub fn segmented<T: Copy + PartialEq>(
    ui: &mut Ui,
    id: &str,
    c: &Colors,
    value: &mut T,
    options: &[(T, &str)],
    seg_w: f32,
) -> bool {
    segmented_sized(ui, id, c, value, options, seg_w, 34.0, 13.0)
}

/// Language button: flag and language name on the current-language colour.
pub fn language_button(ui: &mut Ui, c: &Colors, lang: Lang) -> Response {
    let font = FontId::proportional(13.0);
    let text_w = ui
        .painter()
        .layout_no_wrap(lang.name().to_owned(), font.clone(), c.text)
        .size()
        .x;
    let size = vec2(text_w + 18.0 + 8.0 + 24.0, 34.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    if resp.clicked() {
        resp.request_focus();
    }

    let fill = if c.dark { c.active } else { c.track };
    let hover = if c.dark {
        c.active.lerp_to_gamma(c.text, 0.06)
    } else {
        c.track.lerp_to_gamma(c.text, 0.06)
    };
    let t = ui
        .ctx()
        .animate_bool_with_time(Id::new(("lang_hover", lang.code())), resp.hovered(), 0.12);
    let painter = ui.painter();
    painter.rect_filled(rect, Rounding::same(10.0), fill.lerp_to_gamma(hover, t));

    let flag_rect = eframe::egui::Rect::from_min_size(
        pos2(rect.min.x + 12.0, rect.center().y - 6.0),
        vec2(18.0, 12.0),
    );
    let flag = match lang {
        Lang::Fr => egui::include_image!("../assets/flags/fr.svg"),
        Lang::En => egui::include_image!("../assets/flags/en.svg"),
    };
    Image::new(flag).paint_at(ui, flag_rect);

    painter.text(
        pos2(flag_rect.max.x + 8.0, rect.center().y),
        Align2::LEFT_CENTER,
        lang.name(),
        font,
        c.text,
    );

    pointer_cursor(ui, &resp);
    resp.widget_info(|| WidgetInfo::labeled(WidgetType::Button, ui.is_enabled(), lang.name()));
    resp
}

/// A settings row: label on the left, control on the right, on a rounded row.
pub fn setting_row<R>(
    ui: &mut Ui,
    c: &Colors,
    label: &str,
    control: impl FnOnce(&mut Ui) -> R,
) -> R {
    let width = ui.available_width();
    let (rect, _) = ui.allocate_exact_size(vec2(width, 54.0), Sense::hover());
    ui.painter()
        .rect_filled(rect, Rounding::same(12.0), c.row);

    let inner = rect.shrink2(vec2(16.0, 10.0));
    let mut row = ui.new_child(
        UiBuilder::new()
            .max_rect(inner)
            .layout(Layout::left_to_right(Align::Center)),
    );
    row.spacing_mut().item_spacing.x = 0.0;
    row.add(egui::Label::new(
        egui::RichText::new(label).size(15.0).color(c.text),
    ).truncate());

    row.with_layout(Layout::right_to_left(Align::Center), |ui| control(ui))
        .inner
}
