use eframe::egui::{
    Color32, Context, FontData, FontDefinitions, FontFamily, FontId, Rounding, Shadow, Stroke,
    TextStyle, Ui, Visuals,
};

pub const INK_LIGHT: Color32 = Color32::from_rgb(20, 20, 20);
pub const INK_DARK: Color32 = Color32::from_rgb(240, 240, 240);

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    #[default]
    Light,
    Dark,
}

#[derive(Clone, Copy)]
pub struct Colors {
    pub dark: bool,
    pub bar_bg: Color32,
    pub bar_border: Color32,
    pub segment_bg: Color32,
    pub segment_active: Color32,
    pub hover: Color32,
    pub text: Color32,
    pub text_dim: Color32,
    pub accent: Color32,
    pub rail: Color32,
    pub danger: Color32,
    pub danger_hover: Color32,
    pub canvas_bg: Color32,
    pub ink: Color32,
}

impl Mode {
    pub fn colors(self, canvas_follows_theme: bool) -> Colors {
        match self {
            Mode::Light => Colors {
                dark: false,
                bar_bg: Color32::WHITE,
                bar_border: Color32::from_rgb(226, 226, 226),
                segment_bg: Color32::from_rgb(240, 240, 240),
                segment_active: Color32::WHITE,
                hover: Color32::from_rgb(232, 232, 232),
                text: Color32::from_rgb(30, 30, 30),
                text_dim: Color32::from_rgb(130, 130, 130),
                accent: Color32::from_rgb(13, 153, 255),
                rail: Color32::from_rgb(222, 222, 222),
                danger: Color32::from_rgb(229, 62, 62),
                danger_hover: Color32::from_rgb(200, 45, 45),
                canvas_bg: Color32::WHITE,
                ink: INK_LIGHT,
            },
            Mode::Dark => Colors {
                dark: true,
                bar_bg: Color32::from_rgb(28, 28, 30),
                bar_border: Color32::from_rgb(60, 60, 64),
                segment_bg: Color32::from_rgb(44, 44, 48),
                segment_active: Color32::from_rgb(72, 72, 78),
                hover: Color32::from_rgb(56, 56, 60),
                text: Color32::from_rgb(236, 236, 240),
                text_dim: Color32::from_rgb(150, 150, 158),
                accent: Color32::from_rgb(13, 153, 255),
                rail: Color32::from_rgb(72, 72, 78),
                danger: Color32::from_rgb(229, 62, 62),
                danger_hover: Color32::from_rgb(200, 45, 45),
                canvas_bg: if canvas_follows_theme {
                    Color32::from_rgb(20, 20, 22)
                } else {
                    Color32::WHITE
                },
                ink: if canvas_follows_theme { INK_DARK } else { INK_LIGHT },
            },
        }
    }
}

/// Pastilles de couleur : la première est l'encre (noir ou blanc selon le fond).
pub fn palette(ink: Color32) -> [Color32; 6] {
    [
        ink,
        Color32::from_rgb(242, 72, 34),
        Color32::from_rgb(255, 163, 0),
        Color32::from_rgb(20, 174, 92),
        Color32::from_rgb(13, 153, 255),
        Color32::from_rgb(151, 71, 255),
    ]
}

/// Police Segoe UI (Windows) + tailles de texte plus lisibles.
pub fn setup(ctx: &Context) {
    let mut fonts = FontDefinitions::default();
    if let Ok(bytes) = std::fs::read(r"C:\Windows\Fonts\segoeui.ttf") {
        fonts
            .font_data
            .insert("segoe".to_owned(), FontData::from_owned(bytes));
        fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .insert(0, "segoe".to_owned());
    }
    ctx.set_fonts(fonts);

    ctx.style_mut(|s| {
        s.text_styles = [
            (TextStyle::Heading, FontId::new(21.0, FontFamily::Proportional)),
            (TextStyle::Body, FontId::new(15.0, FontFamily::Proportional)),
            (TextStyle::Button, FontId::new(15.0, FontFamily::Proportional)),
            (TextStyle::Small, FontId::new(13.0, FontFamily::Proportional)),
            (TextStyle::Monospace, FontId::new(14.0, FontFamily::Monospace)),
        ]
        .into();
    });
}

/// Applique le thème (clair ou sombre) au `Ui` courant et à ses enfants.
pub fn apply(ui: &mut Ui, c: &Colors) {
    let mut v = if c.dark { Visuals::dark() } else { Visuals::light() };

    v.window_fill = c.bar_bg;
    v.panel_fill = c.bar_bg;
    v.window_stroke = Stroke::new(1.0_f32, c.bar_border);
    v.window_shadow = Shadow::NONE;
    v.popup_shadow = Shadow::NONE;
    v.extreme_bg_color = c.segment_bg;
    v.selection.bg_fill = c.accent;
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, c.text);
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, c.bar_border);

    let w = &mut v.widgets;
    w.inactive.weak_bg_fill = Color32::TRANSPARENT;
    w.inactive.bg_fill = c.rail;
    w.hovered.weak_bg_fill = c.hover;
    w.hovered.bg_fill = c.rail.lerp_to_gamma(c.text, 0.15);
    w.active.weak_bg_fill = c.hover;
    w.active.bg_fill = c.accent;

    for s in [&mut w.inactive, &mut w.hovered, &mut w.active] {
        s.rounding = Rounding::same(8.0);
        s.bg_stroke = Stroke::NONE;
        s.fg_stroke = Stroke::new(1.0_f32, c.text);
    }

    ui.style_mut().visuals = v;
}