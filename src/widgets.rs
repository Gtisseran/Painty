use eframe::egui::{
    self, pos2, vec2, Align2, Color32, CursorIcon, FontId, Id, Image, Rect, Response, Rounding,
    Sense, Stroke, Ui, Vec2,
};

use crate::theme::Colors;
use crate::i18n::Lang;

/// Visual style of a text button.
#[derive(Clone, Copy)]
pub enum Kind {
    /// Transparent until hovered.
    Ghost,
    /// Filled with the segment background color.
    Soft,
    /// Red, for destructive actions.
    Danger,
}

/// Icons loaded from assets/svg (light and dark versions).
#[derive(Clone, Copy)]
pub enum Icon {
    Undo,
    Trash,
    Export,
}

/// Shows a hand cursor over interactive elements.
fn pointer_cursor(ui: &Ui, resp: &Response) {
    if resp.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }
}

/// Allocates space for an element that can be folded away.
/// `vis` goes from 0.0 (hidden) to 1.0 (fully shown). The gap after the element
/// is scaled too, so a hidden element takes no room at all.
fn alloc(ui: &mut Ui, size: Vec2, sense: Sense, vis: f32) -> (Rect, Response) {
    let gap = ui.spacing().item_spacing.x;
    ui.spacing_mut().item_spacing.x = gap * vis;
    let out = ui.allocate_exact_size(size, sense);
    ui.spacing_mut().item_spacing.x = gap;
    out
}

/// Text button with a fade on hover. Always fully visible.
pub fn button(ui: &mut Ui, id: &str, c: &Colors, label: &str, kind: Kind, min_w: f32) -> Response {
    button_anim(ui, id, c, label, kind, min_w, 1.0)
}

/// Text button whose width and opacity follow `vis` (0.0 hidden, 1.0 shown).
pub fn button_anim(
    ui: &mut Ui,
    id: &str,
    c: &Colors,
    label: &str,
    kind: Kind,
    min_w: f32,
    vis: f32,
) -> Response {
    let (base, hover, fg) = match kind {
        Kind::Ghost => (Color32::TRANSPARENT, c.hover, c.text),
        Kind::Soft => (c.track, c.hover, c.text),
        Kind::Danger => (c.danger, c.danger_hover, Color32::WHITE),
    };

    let font = FontId::proportional(15.0);
    let text_w = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font.clone(), fg)
        .size()
        .x;
    let full = vec2((text_w + 24.0).max(min_w), 32.0);

    // Not clickable while it is appearing or disappearing.
    let sense = if vis > 0.95 { Sense::click() } else { Sense::hover() };
    let (rect, resp) = alloc(ui, vec2(full.x * vis, full.y), sense, vis);
    if vis <= 0.01 {
        return resp;
    }

    let t = ui
        .ctx()
        .animate_bool_with_time(Id::new(("btn", id)), resp.hovered(), 0.12);
    let mut fill = if base.a() == 0 {
        hover.gamma_multiply(t)
    } else {
        base.lerp_to_gamma(hover, t)
    };
    // Pressed state.
    if resp.is_pointer_button_down_on() {
        fill = hover;
    }

    let painter = ui.painter().with_clip_rect(rect);
    painter.rect_filled(rect, Rounding::same(9.0), fill.gamma_multiply(vis));
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        label,
        font,
        fg.gamma_multiply(vis),
    );

    if vis > 0.95 {
        pointer_cursor(ui, &resp);
    }
    resp
}

/// Square button with an icon from assets/svg and a tooltip.
/// Width and opacity follow `vis`.
pub fn icon_button(ui: &mut Ui, id: &str, c: &Colors, icon: Icon, tip: &str, vis: f32) -> Response {
    let sense = if vis > 0.95 { Sense::click() } else { Sense::hover() };
    let (rect, resp) = alloc(ui, vec2(32.0 * vis, 32.0), sense, vis);
    if vis <= 0.01 {
        return resp;
    }

    let t = ui
        .ctx()
        .animate_bool_with_time(Id::new(("icon_btn", id)), resp.hovered(), 0.12);
    let old_clip = ui.clip_rect();
    ui.set_clip_rect(rect);

    ui.painter()
        .rect_filled(rect, Rounding::same(9.0), c.hover.gamma_multiply(t * vis));

    // Icon is 18 px, centered. Clipped to the button while it folds.
    let icon_rect = Rect::from_center_size(rect.center(), vec2(18.0, 18.0));
    let tint = Color32::from_white_alpha((vis * 255.0) as u8);
    let image = match (icon, c.dark) {
        (Icon::Undo, false) => Image::new(egui::include_image!("../assets/svg/undo_dark.svg")),
        (Icon::Undo, true) => Image::new(egui::include_image!("../assets/svg/undo_light.svg")),
        (Icon::Trash, false) => Image::new(egui::include_image!("../assets/svg/trash_dark.svg")),
        (Icon::Trash, true) => Image::new(egui::include_image!("../assets/svg/trash_light.svg")),
        (Icon::Export, false) => Image::new(egui::include_image!("../assets/svg/export_dark.svg")),
        (Icon::Export, true) => Image::new(egui::include_image!("../assets/svg/export_light.svg")),
    };
    image.tint(tint).paint_at(ui, icon_rect);

    ui.set_clip_rect(old_clip);

    if vis > 0.95 {
        pointer_cursor(ui, &resp);
        return resp.on_hover_text(tip);
    }
    resp
}

/// Color swatch with an animated selection ring. Size and opacity follow `vis`.
pub fn swatch(
    ui: &mut Ui,
    idx: usize,
    color: Color32,
    selected: bool,
    c: &Colors,
    vis: f32,
) -> Response {
    let sense = if vis > 0.95 { Sense::click() } else { Sense::hover() };
    let (rect, resp) = alloc(ui, vec2(28.0 * vis, 28.0), sense, vis);
    if vis <= 0.01 {
        return resp;
    }

    let sel = ui
        .ctx()
        .animate_bool_with_time(Id::new(("swatch_sel", idx)), selected, 0.15);
    let hov = ui.ctx().animate_bool_with_time(
        Id::new(("swatch_hov", idx)),
        resp.hovered() && vis > 0.95,
        0.12,
    );

    let painter = ui.painter().with_clip_rect(rect.expand(2.0));
    let center = rect.center();
    // The swatch grows slightly on hover.
    let r = (10.0 + hov * 1.5) * vis;
    painter.circle_filled(center, r, color.gamma_multiply(vis));
    painter.circle_stroke(
        center,
        r,
        Stroke::new(1.0_f32, c.border.gamma_multiply(vis)),
    );
    if sel > 0.0 {
        painter.circle_stroke(
            center,
            (11.0 + sel * 2.0) * vis,
            Stroke::new(2.0_f32, c.accent.gamma_multiply(sel * vis)),
        );
    }

    if vis > 0.95 {
        pointer_cursor(ui, &resp);
    }
    resp
}

/// Brush size in pixels. Width and opacity follow `vis`.
pub fn px_label(ui: &mut Ui, c: &Colors, text: &str, vis: f32) {
    let (rect, _) = alloc(ui, vec2(44.0 * vis, 20.0), Sense::hover(), vis);
    if vis > 0.01 {
        ui.painter().with_clip_rect(rect).text(
            pos2(rect.min.x + 22.0, rect.center().y),
            Align2::CENTER_CENTER,
            text,
            FontId::proportional(15.0),
            c.text_dim.gamma_multiply(vis),
        );
    }
}

/// Thin vertical separator between toolbar groups. Fades in and out with `vis`.
pub fn divider(ui: &mut Ui, c: &Colors, vis: f32) {
    let (rect, _) = alloc(ui, vec2(1.0 * vis, 20.0), Sense::hover(), vis);
    ui.painter()
        .rect_filled(rect, 0.0, c.border.gamma_multiply(vis));
}

/// Segmented control with a pill that slides to the selected option.
pub fn segmented<T: Copy + PartialEq>(
    ui: &mut Ui,
    id: &str,
    c: &Colors,
    value: &mut T,
    options: &[(T, &str)],
    seg_w: f32,
) {
    let pad = 3.0;
    let h = 34.0;
    let size = vec2(seg_w * options.len() as f32 + pad * 2.0, h);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());

    // Animated horizontal position of the pill, in option units.
    let current = options.iter().position(|(v, _)| *v == *value).unwrap_or(0);
    let pos = ui
        .ctx()
        .animate_value_with_time(Id::new(("seg_pos", id)), current as f32, 0.16);

    let painter = ui.painter();
    painter.rect_filled(rect, Rounding::same(11.0), c.track);

    let pill = Rect::from_min_size(
        pos2(rect.min.x + pad + seg_w * pos, rect.min.y + pad),
        vec2(seg_w, h - pad * 2.0),
    );
    painter.rect_filled(pill, Rounding::same(8.0), c.active);
    painter.rect_stroke(pill, Rounding::same(8.0), Stroke::new(1.0_f32, c.border));

    let mut clicked = None;
    for (i, (v, label)) in options.iter().enumerate() {
        let cell = Rect::from_min_size(
            pos2(rect.min.x + pad + seg_w * i as f32, rect.min.y + pad),
            vec2(seg_w, h - pad * 2.0),
        );
        let resp = ui.interact(cell, Id::new(("seg_cell", id, i)), Sense::click());

        // Text darkens as the pill reaches it, and on hover.
        let sel = (1.0 - (pos - i as f32).abs()).clamp(0.0, 1.0);
        let hover = if resp.hovered() { 0.6 } else { 0.0 };
        let color = c.text_dim.lerp_to_gamma(c.text, sel.max(hover));
        painter.text(
            cell.center(),
            Align2::CENTER_CENTER,
            *label,
            FontId::proportional(15.0),
            color,
        );

        pointer_cursor(ui, &resp);
        if resp.clicked() {
            clicked = Some(*v);
        }
    }

    if let Some(v) = clicked {
        *value = v;
    }
}

/// Settings icon. Uses the light or dark SVG depending on the theme.
pub fn settings_button(ui: &mut Ui, c: &Colors) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(34.0, 32.0), Sense::click());
    let t = ui
        .ctx()
        .animate_bool_with_time(Id::new("settings_btn_hover"), resp.hovered(), 0.12);

    ui.painter()
        .rect_filled(rect, Rounding::same(9.0), c.hover.gamma_multiply(t));

    let icon_rect = Rect::from_center_size(rect.center(), vec2(18.0, 17.0));
    // Dark theme: white icon. Light theme: dark icon.
    let source = if c.dark {
        egui::include_image!("../assets/svg/settings_light.svg")
    } else {
        egui::include_image!("../assets/svg/settings_dark.svg")
    };
    Image::new(source).paint_at(ui, icon_rect);

    pointer_cursor(ui, &resp);
    resp
}

/// Language choice, with its flag. Highlighted when selected.
pub fn language_button(ui: &mut Ui, c: &Colors, lang: Lang, selected: bool, width: f32) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(width, 34.0), Sense::click());
    let hov = ui
        .ctx()
        .animate_bool_with_time(Id::new(("lang_hov", lang.code())), resp.hovered(), 0.12);
    let sel = ui
        .ctx()
        .animate_bool_with_time(Id::new(("lang_sel", lang.code())), selected, 0.16);

    let painter = ui.painter().clone();
    let fill = c.track.lerp_to_gamma(c.hover, hov).lerp_to_gamma(c.active, sel);
    painter.rect_filled(rect, Rounding::same(9.0), fill);
    if sel > 0.0 {
        painter.rect_stroke(rect, Rounding::same(9.0), Stroke::new(1.0_f32, c.border.gamma_multiply(sel)));
    }

    let flag_rect = Rect::from_min_size(
        pos2(rect.min.x + 12.0, rect.center().y - 8.0),
        vec2(22.0, 16.0),
    );
    let flag = match lang {
        Lang::Fr => egui::include_image!("../assets/flags/fr.svg"),
        Lang::En => egui::include_image!("../assets/flags/en.svg"),
    };
    Image::new(flag).paint_at(ui, flag_rect);

    let color = if selected { c.text } else { c.text_dim };
    painter.text(
        pos2(flag_rect.max.x + 8.0, rect.center().y),
        Align2::LEFT_CENTER,
        lang.name(),
        FontId::proportional(15.0),
        color,
    );

    pointer_cursor(ui, &resp);
    resp
}