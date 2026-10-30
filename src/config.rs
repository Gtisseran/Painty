use std::{env, fs, path::PathBuf};

use crate::i18n::Lang;
use crate::theme::Mode;

/// User settings, saved between sessions.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Config {
    pub mode: Mode,
    pub canvas_follows_theme: bool,
    pub language: Lang,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mode: Mode::Light,
            canvas_follows_theme: false,
            language: Lang::default(),
        }
    }
}

/// `%APPDATA%\Painty\settings.txt`
fn path() -> Option<PathBuf> {
    env::var_os("APPDATA").map(|p| PathBuf::from(p).join("Painty").join("settings.txt"))
}

/// Loads the settings, or the defaults when the file is missing or unreadable.
pub fn load() -> Config {
    let mut cfg = Config::default();

    let Some(text) = path().and_then(|p| fs::read_to_string(p).ok()) else {
        return cfg;
    };

    for line in text.lines() {
        if let Some((key, value)) = line.split_once('=') {
            match (key.trim(), value.trim()) {
                ("theme", "dark") => cfg.mode = Mode::Dark,
                ("theme", "light") => cfg.mode = Mode::Light,
                ("canvas_follows_theme", v) => cfg.canvas_follows_theme = v == "true",
                ("language", v) => {
                    if let Some(lang) = Lang::from_code(v) {
                        cfg.language = lang;
                    }
                }
                _ => {}
            }
        }
    }

    cfg
}

/// Writes the settings to disk. Errors are ignored: settings are not critical.
pub fn save(cfg: &Config) {
    let Some(path) = path() else {
        return;
    };
    if let Some(dir) = path.parent() {
        let _ = fs::create_dir_all(dir);
    }
    let text = format!(
        "theme={}\ncanvas_follows_theme={}\nlanguage={}\n",
        if cfg.mode == Mode::Dark { "dark" } else { "light" },
        cfg.canvas_follows_theme,
        cfg.language.code()
    );
    let _ = fs::write(path, text);
}