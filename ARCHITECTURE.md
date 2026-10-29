# Architecture

## Folders

| Path | Content |
|------|---------|
| `src/` | Rust source code |
| `assets/` | Images embedded into the binary at compile time |
| `target/` | Build output (ignored by Git) |

## Assets

| File | Use |
|------|-----|
| `assets/icon.png` | Window icon (256×256 PNG) |
| `assets/painty.ico` | Executable icon (multi-size ICO, used by `build.rs`) |
| `assets/settings_dark.svg` | Settings icon for the light theme (dark stroke) |
| `assets/settings_light.svg` | Settings icon for the dark theme (white stroke) |

## Source files

| File | Role |
|------|------|
| `main.rs` | Entry point. Window options, icon, image loaders, fonts. |
| `app.rs` | Main application state (`Painty`). Ties all parts together each frame. |
| `canvas.rs` | Drawing area. Handles mouse input and paints strokes. |
| `model.rs` | Data types: `Tool`, `Line` (one stroke), `Brush` (current settings). |
| `toolbar.rs` | Bottom floating toolbar. Elements fold in and out with animations depending on window width. |
| `widgets.rs` | Custom controls: buttons, color swatches, segmented control, settings icon. |
| `dialog.rs` | Modal popup with dimmed background, slide-in animation and scrolling. |
| `settings.rs` | Settings dialog (theme, canvas background, About). |
| `config.rs` | Loads and saves settings to `%APPDATA%\Painty\settings.txt`. |
| `theme.rs` | Colors for light and dark mode, font setup, egui style. |

## Data flow

1. `main.rs` creates the window and `Painty`.
2. Each frame, `app.rs` reads the theme (`theme.rs`) and draws:
   - the canvas (`canvas.rs`), which updates the list of strokes (`model.rs`);
   - the toolbar (`toolbar.rs`), which edits the current brush;
   - the dialogs (`settings.rs`, the clear confirmation), built with `dialog.rs`.
3. When a setting changes, `config.rs` writes it to disk.

## Build script

`build.rs` (not in the list above) embeds `assets/painty.ico` into `painty.exe` on Windows.