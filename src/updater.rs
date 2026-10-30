//! Update checks and self-update through GitHub Releases.
//! Only the program itself is downloaded (painty.exe), never an installer.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::thread;

use eframe::egui::{Context, ViewportCommand};

/// Latest release of the repository. The release must contain an asset named `painty.exe`.
const API_URL: &str = "https://api.github.com/repos/Gtisseran/painty/releases/latest";
const ASSET_NAME: &str = "painty.exe";
/// Name given to the previous executable after replacement.
const OLD_NAME: &str = "painty.old.exe";

/// A newer version found on GitHub.
#[derive(Clone)]
pub struct Release {
    pub version: String,
    pub url: String,
}

/// What the updater is doing, read by the UI.
#[derive(Clone, Default)]
pub enum Status {
    #[default]
    Idle,
    /// Asking GitHub for the latest version.
    Checking,
    UpToDate,
    /// A newer version exists and has not been installed yet.
    Available(Release),
    /// Downloading and replacing the executable.
    Downloading,
    /// The last manual check failed (shown in settings only).
    CheckFailed(String),
    /// The download or replacement failed (shown in the prompt).
    InstallFailed(String),
}

/// Messages sent back by background threads.
enum Message {
    Checked {
        result: Result<Option<Release>, String>,
        /// Manual checks show errors; automatic checks stay silent.
        manual: bool,
    },
    /// Holds the path of the executable that was replaced.
    Installed(Result<PathBuf, String>),
}

#[derive(Default)]
pub struct Updater {
    rx: Option<Receiver<Message>>,
    pub status: Status,
}

impl Updater {
    /// Starts a version check in the background.
    pub fn check(&mut self, ctx: &Context, manual: bool) {
        if matches!(self.status, Status::Checking | Status::Downloading) {
            return;
        }
        self.status = Status::Checking;

        let (tx, rx) = channel();
        self.rx = Some(rx);
        let ctx = ctx.clone();
        thread::spawn(move || {
            let result = fetch_latest();
            let _ = tx.send(Message::Checked { result, manual });
            ctx.request_repaint();
        });
    }

    /// Downloads the new executable in the background, then replaces the current one.
    pub fn install(&mut self, ctx: &Context, release: Release) {
        if matches!(self.status, Status::Downloading) {
            return;
        }
        self.status = Status::Downloading;

        let (tx, rx) = channel();
        self.rx = Some(rx);
        let ctx = ctx.clone();
        thread::spawn(move || {
            let result = download_and_replace(&release.url);
            let _ = tx.send(Message::Installed(result));
            ctx.request_repaint();
        });
    }

    /// Closes the update prompt without installing.
    pub fn dismiss(&mut self) {
        if matches!(self.status, Status::Available(_) | Status::InstallFailed(_)) {
            self.status = Status::Idle;
        }
    }

    /// Handles messages from background threads. Call once per frame.
    pub fn poll(&mut self, ctx: &Context) {
        let Some(rx) = &self.rx else {
            return;
        };
        let message = match rx.try_recv() {
            Ok(message) => message,
            Err(TryRecvError::Empty) => return,
            Err(TryRecvError::Disconnected) => {
                self.rx = None;
                return;
            }
        };
        self.rx = None;

        match message {
            Message::Checked { result, manual } => {
                self.status = match result {
                    Ok(Some(release)) => Status::Available(release),
                    Ok(None) => Status::UpToDate,
                    Err(error) if manual => Status::CheckFailed(error),
                    Err(_) => Status::Idle,
                };
            }
            Message::Installed(Ok(path)) => {
                restart(&path);
                ctx.send_viewport_cmd(ViewportCommand::Close);
            }
            Message::Installed(Err(error)) => {
                self.status = Status::InstallFailed(error);
            }
        }
    }
}

/// Asks GitHub for the latest release. Returns `None` if this version is up to date.
fn fetch_latest() -> Result<Option<Release>, String> {
    // Read the body as text, then parse it with serde_json (ureq's `json` feature is not enabled).
    let body = ureq::get(API_URL)
        .set("User-Agent", "painty-updater")
        .set("Accept", "application/vnd.github+json")
        .call()
        .map_err(|e| e.to_string())?
        .into_string()
        .map_err(|e| e.to_string())?;
    let json: serde_json::Value = serde_json::from_str(&body).map_err(|e| e.to_string())?;

    let tag = json["tag_name"].as_str().ok_or("release has no tag")?;
    let version = tag.trim_start_matches('v').to_owned();
    if !is_newer(&version, env!("CARGO_PKG_VERSION")) {
        return Ok(None);
    }

    let url = json["assets"]
        .as_array()
        .and_then(|assets| {
            assets
                .iter()
                .find(|asset| asset["name"].as_str() == Some(ASSET_NAME))
        })
        .and_then(|asset| asset["browser_download_url"].as_str())
        .ok_or("painty.exe not found in the release")?
        .to_owned();

    Ok(Some(Release { version, url }))
}

/// Compares "major.minor.patch" versions.
fn is_newer(candidate: &str, current: &str) -> bool {
    fn parse(v: &str) -> [u32; 3] {
        let mut parts = [0; 3];
        for (i, s) in v.split('.').take(3).enumerate() {
            parts[i] = s.parse().unwrap_or(0);
        }
        parts
    }
    parse(candidate) > parse(current)
}

/// Downloads the new executable and swaps it with the running one.
/// Returns the path of the replaced executable, to restart it.
fn download_and_replace(url: &str) -> Result<PathBuf, String> {
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let dir = exe.parent().ok_or("invalid install folder")?.to_path_buf();
    let new_exe = dir.join("painty.new.exe");
    let old_exe = dir.join(OLD_NAME);

    // 1. Download the new executable next to the current one.
    let response = ureq::get(url).call().map_err(|e| e.to_string())?;
    let mut reader = response.into_reader();
    let mut file = std::fs::File::create(&new_exe).map_err(|e| e.to_string())?;
    std::io::copy(&mut reader, &mut file).map_err(|e| e.to_string())?;
    drop(file);

    // 2. Windows allows renaming a running exe, but not overwriting it.
    let _ = std::fs::remove_file(&old_exe);
    std::fs::rename(&exe, &old_exe).map_err(|e| e.to_string())?;
    if let Err(e) = std::fs::rename(&new_exe, &exe) {
        // Put the old executable back so the app keeps working.
        let _ = std::fs::rename(&old_exe, &exe);
        return Err(e.to_string());
    }

    Ok(exe)
}

/// Starts the replaced executable. The current process closes right after.
fn restart(path: &Path) {
    let _ = std::process::Command::new(path).spawn();
}

/// Removes the executable left over from the last update. Call at startup.
pub fn cleanup() {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let _ = std::fs::remove_file(dir.join(OLD_NAME));
        }
    }
}