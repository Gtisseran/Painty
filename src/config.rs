use std::{env, fs, path::PathBuf};

use crate::theme::Mode;

pub struct Config {
    pub mode: Mode,
    pub canvas_follows_theme: bool,
}

fn path() -> Option<PathBuf> {
    env::var_os("APPDATA").map(|p| PathBuf::from(p).join("Painty").join("settings.txt"))
}

pub fn load() -> Config {
    let mut cfg = Config {
        mode: Mode::Light,
        canvas_follows_theme: false,
    };

    let Some(text) = path().and_then(|p| fs::read_to_string(p).ok()) else {
        return cfg;
    };

    for line in text.lines() {
        if let Some((key, value)) = line.split_once('=') {
            match (key.trim(), value.trim()) {
                ("theme", "dark") => cfg.mode = Mode::Dark,
                ("theme", "light") => cfg.mode = Mode::Light,
                ("canvas_follows_theme", v) => cfg.canvas_follows_theme = v == "true",
                _ => {}
            }
        }
    }

    cfg
}

pub fn save(mode: Mode, canvas_follows_theme: bool) {
    let Some(path) = path() else {
        return;
    };
    if let Some(dir) = path.parent() {
        let _ = fs::create_dir_all(dir);
    }
    let text = format!(
        "theme={}\ncanvas_follows_theme={}\n",
        if mode == Mode::Dark { "dark" } else { "light" },
        canvas_follows_theme
    );
    let _ = fs::write(path, text);
}