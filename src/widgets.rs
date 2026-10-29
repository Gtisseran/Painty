use eframe::egui::{
    self, pos2, vec2, Align2, Color32, CursorIcon, FontId, Id, Image, Rect, Response, Rounding,
    Sense, Stroke, Ui,
};

use crate::theme::Colors;

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

/// Shows a hand cursor over interactive elements.
fn pointer_cursor(ui: &Ui, resp: &Response) {
    if resp.hovered() {
        ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
    }
}

/// Text button with a fade on hover.
pub fn button(ui: &mut Ui, id: &str, c: &Colors, label: &str, kind: Kind, min_w: f32) -> Response {
    let (base, hover, fg) = match kind {
        Kind::Ghost => (Color32::TRANSPARENT, c.hover, c.text),
        Kind::Soft => (c.segment_bg, c.hover, c.text),
        Kind::Danger => (c.danger, c.danger_hover, Color32::WHITE),
    };

    let font = FontId::proportional(15.0);
    let text_w = ui
        .painter()
        .layout_no_wrap(label.to_owned(), font.clone(), fg)
        .size()
        .x;
    let size = vec2((text_w + 24.0).max(min_w), 32.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());

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

    let painter = ui.painter();
    painter.rect_filled(rect, Rounding::same(9.0), fill);
    painter.text(rect.center(), Align2::CENTER_CENTER, label, font, fg);

    pointer_cursor(ui, &resp);
    resp
}

/// Color swatch with an animated selection ring.
pub fn swatch(ui: &mut Ui, idx: usize, color: Color32, selected: bool, c: &Colors) -> Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(28.0, 28.0), Sense::click());

    let sel = ui
        .ctx()
        .animate_bool_with_time(Id::new(("swatch_sel", idx)), selected, 0.15);
    let hov = ui
        .ctx()
        .animate_bool_with_time(Id::new(("swatch_hov", idx)), resp.hovered(), 0.12);

    let painter = ui.painter();
    let center = rect.center();
    // The swatch grows slightly on hover.
    let r = 10.0 + hov * 1.5;
    painter.circle_filled(center, r, color);
    painter.circle_stroke(center, r, Stroke::new(1.0_f32, c.bar_border));
    if sel > 0.0 {
        painter.circle_stroke(
            center,
            11.0 + sel * 2.0,
            Stroke::new(2.0_f32, c.accent.gamma_multiply(sel)),
        );
    }

    pointer_cursor(ui, &resp);
    resp
}

/// Thin vertical separator between toolbar groups.
pub fn divider(ui: &mut Ui, c: &Colors) {
    let (rect, _) = ui.allocate_exact_size(vec2(1.0, 20.0), Sense::hover());
    ui.painter().rect_filled(rect, 0.0, c.bar_border);
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
    painter.rect_filled(rect, Rounding::same(11.0), c.segment_bg);

    let pill = Rect::from_min_size(
        pos2(rect.min.x + pad + seg_w * pos, rect.min.y + pad),
        vec2(seg_w, h - pad * 2.0),
    );
    painter.rect_filled(pill, Rounding::same(8.0), c.segment_active);
    painter.rect_stroke(pill, Rounding::same(8.0), Stroke::new(1.0_f32, c.bar_border));

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
        egui::include_image!("../assets/settings_light.svg")
    } else {
        egui::include_image!("../assets/settings_dark.svg")
    };
    Image::new(source).paint_at(ui, icon_rect);

    pointer_cursor(ui, &resp);
    resp
}