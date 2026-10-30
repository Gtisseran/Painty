# Architecture

## Folders

| Path | Content |
|------|---------|
| `src/` | Rust source code |
| `assets/` | Images embedded into the binary at compile time |
| `dist/` | Installer output (ignored by Git) |
| `target/` | Build output (ignored by Git) |

## Assets

| File | Use |
|------|-----|
| `assets/icon.png` | Window icon (256×256 PNG) |
| `assets/painty.ico` | Executable and installer icon (multi-size ICO) |
| `assets/settings_dark.svg` | Settings icon for the light theme (dark stroke) |
| `assets/settings_light.svg` | Settings icon for the dark theme (white stroke) |
| `assets/undo_dark.svg` | Undo icon for the light theme |
| `assets/undo_light.svg` | Undo icon for the dark theme |
| `assets/trash_dark.svg` | Clear icon for the light theme |
| `assets/trash_light.svg` | Clear icon for the dark theme |

## Source files

| File | Role |
|------|------|
| `main.rs` | Entry point. Window options, icon, image loaders, fonts. Hides the console in release builds. |
| `app.rs` | Main application state (`Painty`). Ties all parts together each frame. |
| `canvas.rs` | Drawing area. Handles mouse input and paints strokes. |
| `model.rs` | Data types: `Tool`, `Line` (one stroke), `Brush` (current settings). |
| `toolbar.rs` | Bottom floating toolbar. Elements fold in and out with animations depending on window width. |
| `widgets.rs` | Custom controls: buttons, icon buttons, color swatches, segmented control, settings icon. |
| `dialog.rs` | Modal popup with dimmed background, slide-in animation and scrolling. |
| `settings.rs` | Settings dialog: theme, canvas background, updates, About. |
| `updater.rs` | Update check and self-update through GitHub Releases. Runs in background threads. |
| `config.rs` | Loads and saves settings to `%APPDATA%\Painty\settings.txt`. |
| `theme.rs` | Colors for light and dark mode, font setup, egui style. |

## Data flow

1. `main.rs` creates the window and `Painty`.
2. `app.rs` starts a silent update check on the first frame, then reads the theme (`theme.rs`) every frame and draws:
   - the canvas (`canvas.rs`), which updates the list of strokes (`model.rs`);
   - the toolbar (`toolbar.rs`), which edits the current brush;
   - the dialogs (`settings.rs`, the clear confirmation, the update prompt), built with `dialog.rs`.
3. When a setting changes, `config.rs` writes it to disk.
4. `updater.rs` sends its results back through a channel, read by `app.rs` with `poll()`.

## Update flow

1. `fetch_latest()` reads the latest GitHub release and compares its tag with the version in `Cargo.toml`.
2. If a newer version exists, it finds the `painty.exe` asset.
3. On install, the new file is downloaded next to the current one, then the two files are swapped.
4. The new executable is started and the current window closes.
5. At the next launch, `cleanup()` removes the old executable.

## Build files

| File | Role |
|------|------|
| `Cargo.toml` | Dependencies and version number |
| `build.rs` | Embeds `assets/painty.ico` into `painty.exe` on Windows |
| `painty.iss` | Inno Setup script for the installer |
| `run.ps1` | Sets the terminal tab title and runs `cargo run` |