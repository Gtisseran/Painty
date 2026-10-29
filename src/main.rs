mod app;
mod canvas;
mod config;
mod dialog;
mod model;
mod settings;
mod theme;
mod toolbar;
mod widgets;

use eframe::egui;

fn main() -> eframe::Result {
    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon.png"))
        .expect("icône invalide");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 600.0])
            .with_min_inner_size([480.0, 320.0])
            .with_title("Painty")
            .with_icon(icon),
        ..Default::default()
    };

    eframe::run_native(
        "Painty",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            theme::setup(&cc.egui_ctx);
            Ok(Box::new(app::Painty::new()))
        }),
    )
}