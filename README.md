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
- Responsive toolbar that adapts smoothly to the window width
- Automatic update check at startup, and a manual check in Settings

## Download

Get the latest installer from the [Releases](https://github.com/Gtisseran/painty/releases) page: `PaintySetup-x.y.z.exe`.

## Requirements (to build)

- Windows 10 or 11
- Rust (stable), installed with [rustup](https://rustup.rs)
- Visual Studio Build Tools with the C++ workload (MSVC linker)
- [Inno Setup 6](https://jrsoftware.org/isinfo.php), only to build the installer

## Run

```powershell
.\run.ps1
```

Or directly:

```powershell
cargo run
```

## Build the installer

```powershell
cargo build --release
& "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe" painty.iss
```

The installer is written to `dist\PaintySetup-<version>.exe`.

## Updates

On startup, Painty checks GitHub Releases in the background. If a newer version exists, it asks whether to install it. Only `painty.exe` is downloaded and replaces the running executable. No installer is run.

The check can also be started manually from Settings → Updates.

## Project layout

See [ARCHITECTURE.md](ARCHITECTURE.md) for a description of each file.

## Settings

Settings are stored in `%APPDATA%\Painty\settings.txt`.

## License

All rights reserved.