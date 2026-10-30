<p align="center">
  <img src="assets/icon.png" width="128" alt="Painty logo">
</p>

<h1 align="center">Painty</h1>

<p align="center">
  A minimal paint app for Windows, written in Rust with egui.
</p>

<p align="center">
  <a href="https://github.com/Gtisseran/painty/releases/latest"><b>Download</b></a>
  ·
  <a href="ARCHITECTURE.md">Architecture</a>
  ·
  <a href="LICENSE">MIT License</a>
</p>

---

## Features

- **Brush** with adjustable size and color, and a color picker
- **Smooth strokes** that follow the pointer without jitter
- **Eraser** that cuts through existing strokes
- **Undo and redo** (Ctrl+Z / Ctrl+Y)
- **Export to PNG** (Ctrl+S)
- **Clear all**, with confirmation
- **Light and dark theme**, with a white or theme-coloured canvas
- **Responsive toolbar** that adapts smoothly to the window width
- **Automatic updates** from GitHub Releases, with a progress bar
- **Settings saved** between sessions

## Install

1. Go to the [latest release](https://github.com/Gtisseran/painty/releases/latest).
2. Download `PaintySetup-x.y.z.exe` and run it.
3. Choose where to install. No administrator rights are needed.

Painty checks for updates when it starts. You can also check from **Settings → Updates**.

## Keyboard shortcuts

| Shortcut | Action |
|----------|--------|
| Ctrl+Z | Undo |
| Ctrl+Y | Redo |
| Ctrl+S | Export as PNG |

## Build from source

Requirements:

- Windows 10 or 11
- [Rust](https://rustup.rs) (stable)
- Visual Studio Build Tools with the **C++ workload** (MSVC linker)
- [Inno Setup 6](https://jrsoftware.org/isinfo.php), only to build the installer

Run the app:

```powershell
.\run.ps1
```

Build the executable and the installer:

```powershell
cargo build --release
& "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe" painty.iss
```

The installer is written to `dist\PaintySetup-<version>.exe`.

## Updates

Painty downloads only `painty.exe` from the latest GitHub release and replaces itself. No installer is run, and the new version starts right after the download.

## Settings

Settings are stored in `%APPDATA%\Painty\settings.txt`.

## Project structure

See [ARCHITECTURE.md](ARCHITECTURE.md) for a full description of each file.