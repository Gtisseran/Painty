# Painty

A minimal paint application written in Rust with [egui](https://github.com/emilk/egui) and eframe.

## Features

- Freehand brush with adjustable size and color
- Eraser
- Undo (toolbar button or Ctrl+Z)
- Clear all, with confirmation
- Light and dark theme
- Canvas background that either stays white or follows the theme
- Settings saved between sessions
- Responsive toolbar with three layouts

## Requirements

- Windows 10 or 11
- Rust (stable), installed with [rustup](https://rustup.rs)
- Visual Studio Build Tools with the C++ workload (MSVC linker)

## Run

```powershell
.\run.ps1
```

Or directly:

```powershell
cargo run
```

Release build:

```powershell
cargo build --release
```

The executable is written to `target\release\painty.exe`.

## Project layout

See [ARCHITECTURE.md](ARCHITECTURE.md) for a description of each file.

## Settings

Settings are stored in `%APPDATA%\Painty\settings.txt`.

## License

Private project. All rights reserved.