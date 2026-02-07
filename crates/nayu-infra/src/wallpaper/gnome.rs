use std::path::Path;
use std::process::Command;

use anyhow::{anyhow, Context};

fn file_uri(path: &Path) -> anyhow::Result<String> {
    // GNOME expects a file:// URI.
    let s = path
        .to_str()
        .ok_or_else(|| anyhow!("path is not valid UTF-8"))?;
    Ok(format!("file://{s}"))
}

pub fn set_wallpaper(image_abs: &Path) -> anyhow::Result<()> {
    let uri = file_uri(image_abs)?;

    // `gsettings` is the stable CLI for dconf-backed settings.
    let status = Command::new("gsettings")
        .arg("set")
        .arg("org.gnome.desktop.background")
        .arg("picture-uri")
        .arg(&uri)
        .status()
        .context("run gsettings (picture-uri)")?;
    if !status.success() {
        return Err(anyhow!("gsettings failed (picture-uri)"));
    }

    // Best-effort: also set the dark variant if available (GNOME 42+).
    let _ = Command::new("gsettings")
        .arg("set")
        .arg("org.gnome.desktop.background")
        .arg("picture-uri-dark")
        .arg(&uri)
        .status();

    // Best-effort: match our default scaling (cover/fill).
    let _ = Command::new("gsettings")
        .arg("set")
        .arg("org.gnome.desktop.background")
        .arg("picture-options")
        .arg("zoom")
        .status();

    Ok(())
}
