# Architecture

This document describes how Painty is organised, file by file.

## Overview

```text
main.rs ──► app.rs ──┬─► canvas.rs ──► model.rs
                    ├─► toolbar.rs ──► widgets.rs
                    ├─► settings.rs ─┐
                    ├─► dialogs ─────┴─► dialog.rs
                    ├─► export.rs
                    ├─► updater.rs
                    ├─► i18n.rs ◄── assets/lang/*.json
                    └─► config.rs

theme.rs is used by every UI file.
```

## Folders

| Path | Content |
|------|---------|
| `src/` | Rust source code |
| `assets/` | Images, icons, flags and translations, embedded into the binary at compile time |
| `assets/svg/` | Icons (dark and light versions) |
| `assets/flags/` | Flags used by the language dropdown |
| `assets/fonts/` | Inter fonts (regular and semibold) |
| `assets/lang/` | Translations (`en.json`, `fr.json`) |
| `dist/` | Installer output (ignored by Git) |
| `target/` | Build output (ignored by Git) |

## Assets

| File | Use |
|------|-----|
| `icon.png` | Logo, window icon and README header (256 × 256 PNG) |
| `painty.ico` | Executable and installer icon (multi-size ICO) |
| `svg/settings_dark.svg` / `svg/settings_light.svg` | Settings icons for dark and light themes |
| `svg/undo_dark.svg` / `svg/undo_light.svg` | Undo icons for dark and light themes |
| `svg/trash_dark.svg` / `svg/trash_light.svg` | Clear icons for dark and light themes |
| `svg/export_dark.svg` / `svg/export_light.svg` | Export icons for dark and light themes |
| `flags/fr.svg` / `flags/en.svg` | Flags used by the language dropdown |
| `fonts/Inter-Regular.ttf` / `fonts/Inter-SemiBold.ttf` | Interface fonts |
| `lang/en.json` / `lang/fr.json` | Interface translations. Both files must contain the same keys. |

Icons and flags are loaded using `egui::include_image!`, so they are embedded in the executable and do not need to be distributed separately.

## Source Files

| File | Role |
|------|------|
| `main.rs` | Entry point. Configures window options, the window icon, image loaders and fonts. Hides the console in release builds. |
| `app.rs` | Main application state (`Painty`). Handles keyboard shortcuts, the currently open dialog (`Modal`) and frame rendering. |
| `canvas.rs` | Drawing area. Smooths pointer movement, renders strokes, erases by splitting them and maintains undo/redo history. |
| `model.rs` | Core data types: `Tool`, `Line` (a single stroke) and `Brush` (current settings). |
| `toolbar.rs` | Floating toolbar at the bottom of the window. One row whose elements fold in and out with animations to fit the window width, plus a palette row that opens above it when there is room. |
| `widgets.rs` | Custom UI controls: text buttons, icon buttons, color swatches, size slider, segmented controls and the language dropdown. Focus rings are drawn only for keyboard navigation. |
| `dialog.rs` | Modal popup system: dimmed background, slide-in animation and scrolling when the window is too short. Popups are measured again once a window resize has settled. |
| `settings.rs` | Settings dialog: theme, canvas background, language, update check and version. |
| `export.rs` | Exports drawings to PNG or JPEG. Renders at 3× resolution and averages pixels for smoother edges. |
| `updater.rs` | Checks for updates and handles self-updates through GitHub Releases, including download progress. Runs in background threads. |
| `i18n.rs` | Translation system. Loads JSON files and provides `t("key")` for plain text and `tf("key", &[...])` for formatted text. |
| `config.rs` | Loads and saves settings (theme, canvas background and language) to `%APPDATA%\Painty\settings.txt`. |
| `theme.rs` | Defines light and dark theme colors (text contrast of at least 4.5:1), the Inter fonts and the egui style. |

## Toolbar

The toolbar is a single row that keeps the same order at every width. Each element sits in a cell whose width and opacity are animated, so it folds in or out instead of jumping:

1. The slider takes the space left over, between 32 and 120 px.
2. When it can reach 80 px with the text labels, Undo and Clear switch from icons to text.
3. When the slider can reach its full length, a palette row of six colors opens above the row.

Widths are measured every frame, so translations of any length fit. With the eraser active, the color button and the palette are dimmed and the size stays visible.

## Drawing

- **Brush:** Pointer positions are smoothed using `SMOOTHING` in `canvas.rs`. The smoothing is time-based, so it behaves consistently at any frame rate. Points are then added to a `Line`. `Line::push` skips points that are too close together, keeping long strokes lightweight. Each stroke is rendered with rounded joins.
- **Eraser:** Each eraser step cuts every stroke within the eraser radius. Strokes are split into runs of points that were not erased, so no gaps are left behind. A complete eraser gesture counts as one undo step.
- **Undo / Redo:** Each finished stroke or erase operation creates a checkpoint. Strokes are shared using `Arc`, so checkpoints only need to copy pointers. History is limited to `HISTORY_LIMIT` steps.

## Dialogs

`app.rs` maintains a single `Modal` value, which can represent the settings dialog, clear confirmation, export form, export error or quit confirmation.

The update prompt is driven by the updater status and waits until no other dialog is visible. Each dialog is rendered through `dialog.rs`, which preserves its content while it fades out.

## Export

The export dialog asks for a file name and format. Once confirmed, a file dialog lets the user choose where to save the drawing.

`export.rs` renders the entire drawing (not just the visible area) at three times the original resolution, averages each block of pixels and saves the result. The exported background uses the current canvas color.

## Update Flow

1. `fetch_latest()` retrieves the latest GitHub release and compares its tag with the version defined in `Cargo.toml`.
2. If a newer version is available, it looks for the `painty.exe` release asset.
3. When the user installs the update, the executable is downloaded next to the current one in chunks. `Progress` tracks the bytes received, and the UI reads this value every frame.
4. Incomplete downloads are rejected. Otherwise, the current executable is renamed and replaced with the downloaded version.
5. The new executable is launched, and the current application window closes.
6. On the next launch, `cleanup()` removes any leftover update files.

## Data Flow (One Frame)

1. `app.rs` checks the updater status and reads the current theme from `theme.rs`.
2. Keyboard shortcuts are processed unless a dialog is open.
3. `canvas.rs` handles pointer input and renders the strokes.
4. `toolbar.rs` handles brush settings and returns any triggered actions.
5. Dialogs are rendered one at a time through `dialog.rs`.
6. When a setting changes, `config.rs` saves the updated configuration to disk.

## Build Files

| File | Role |
|------|------|
| `Cargo.toml` | Dependencies, application version and license |
| `build.rs` | Embeds `assets/painty.ico` into `painty.exe` on Windows |
| `painty.iss` | Inno Setup configuration for building the installer |
| `run.ps1` | Sets the terminal tab title and runs `cargo run` |
| `build.ps1` | Builds the release executable using `cargo build --release` and creates the installer using Inno Setup |