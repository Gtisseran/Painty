# Architecture

This document describes how Painty is organised, file by file.

## Overview

```text
main.rs
  └── app.rs
       ├── canvas.rs
       │    └── model.rs
       ├── toolbar.rs
       │    └── widgets.rs
       ├── settings.rs
       │    └── dialog.rs
       ├── dialogs
       │    └── dialog.rs
       ├── export.rs
       ├── updater.rs
       └── config.rs

theme.rs is shared by all UI files.
```

## Folders

| Path | Content |
|------|---------|
| `src/` | Rust source code |
| `assets/` | Logo and icons, embedded into the binary at compile time |
| `dist/` | Installer output (ignored by Git) |
| `target/` | Build output (ignored by Git) |

## Assets

| File | Use |
|------|-----|
| `icon.png` | Logo, window icon and README header (256×256 PNG) |
| `painty.ico` | Executable and installer icon (multi-size ICO) |
| `settings_dark.svg` | Settings icon on the light theme (dark stroke) |
| `settings_light.svg` | Settings icon on the dark theme (white stroke) |
| `undo_dark.svg` / `undo_light.svg` | Undo icons for light and dark themes |
| `trash_dark.svg` / `trash_light.svg` | Clear icons for light and dark themes |
| `export_dark.svg` / `export_light.svg` | Export icons for light and dark themes |

Icons are loaded with `egui::include_image!`, so they are embedded in the executable and do not need to be distributed separately.

## Source Files

| File | Role |
|------|------|
| `main.rs` | Application entry point. Configures window options, window icon, image loaders and fonts. Hides the console in release builds. |
| `app.rs` | Main application state (`Painty`). Handles shortcuts, the quit confirmation and draws each frame, including the canvas, toolbar and dialogs. |
| `canvas.rs` | Drawing area. Smooths pointer input, renders strokes, splits strokes when erasing and manages undo/redo history. |
| `model.rs` | Core data types: `Tool`, `Line` (a single stroke) and `Brush` (current brush settings). |
| `toolbar.rs` | Floating toolbar at the bottom of the window. Elements fold in and out with animations depending on the available width. |
| `widgets.rs` | Custom UI controls: text buttons, icon buttons, color swatches, segmented controls and settings icons. |
| `dialog.rs` | Modal dialog system with a dimmed background, slide-in animation and scrolling support for short windows. |
| `settings.rs` | Settings dialog, including theme, canvas background, updates, version and license information. |
| `export.rs` | PNG export. Rasterises the canvas strokes into an image and saves the result. |
| `updater.rs` | Checks for and installs updates through GitHub Releases, with download progress handled in background threads. |
| `config.rs` | Loads and saves settings to `%APPDATA%\Painty\settings.txt`. |
| `theme.rs` | Defines light and dark colors, the Segoe UI font and the egui visual style. |

## Drawing

- **Brush:** Pointer positions are smoothed using `SMOOTHING` in `canvas.rs`, then added to a `Line`. `Line::push` skips points that are too close together, keeping long strokes lightweight. Strokes are rendered as polylines with rounded caps and joins.
- **Eraser:** Each erase operation checks strokes within the eraser radius. Affected strokes are split into runs of points that remain outside the erased area, preventing unwanted gaps.
- **Undo / Redo:** Each completed stroke or erase operation creates a history checkpoint. Strokes are shared using `Arc`, so checkpoints only need to copy pointers. History is limited by `HISTORY_LIMIT`.

## Export

`export.rs` creates an RGBA image buffer filled with the canvas background, renders each stroke as rounded segments and saves the result using the `image` crate. The image covers the bounding box of every stroke, with a small margin, so resizing the window does not cut the drawing.

The exported image uses the screen's pixel density (`pixels_per_point`) to maintain visual sharpness.

## Update Flow

1. `fetch_latest()` retrieves the latest GitHub release and compares its tag with the version defined in `Cargo.toml`.
2. If a newer version is available, the updater looks for the `painty.exe` release asset.
3. When installation starts, the executable is downloaded next to the current one in chunks. `Progress` tracks the downloaded bytes, and the UI reads this information each frame.
4. The current executable is renamed, and the downloaded executable takes its place.
5. The new executable is launched, and the current application window closes.
6. On the next launch, `cleanup()` removes leftover update files.

## Data Flow (One Frame)

1. `app.rs` checks the updater status and retrieves the current theme from `theme.rs`.
2. Keyboard shortcuts are processed, unless a dialog or the update prompt is open.
3. `canvas.rs` handles pointer input and renders the strokes.
4. `toolbar.rs` handles brush controls and returns any triggered actions.
5. `settings.rs` and other dialog-related components render settings, clear confirmations, export errors and update prompts through `dialog.rs`.
6. When a setting changes, `config.rs` saves the updated configuration to disk.

## Build Files

| File | Role |
|------|------|
| `Cargo.toml` | Project dependencies, version and license information |
| `build.rs` | Embeds `assets/painty.ico` into `painty.exe` on Windows |
| `painty.iss` | Inno Setup installer script |
| `run.ps1` | Sets the terminal tab title and runs `cargo run` |