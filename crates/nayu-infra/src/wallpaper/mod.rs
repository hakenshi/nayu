//! Desktop wallpaper backends.
//!
//! v0 goal: set wallpaper on common desktops without requiring `rze`.

use std::path::Path;

use anyhow::Context;

use crate::env_detect::DesktopKind;

mod cosmic;
mod gnome;
mod kde;

/// Try to set wallpaper directly (without IPC/daemon).
///
/// Returns `Ok(true)` if a direct backend handled the request.
/// Returns `Ok(false)` if no direct backend matched.
pub fn try_set_direct(image_abs: &Path) -> anyhow::Result<bool> {
    let desktop = crate::env_detect::detect_desktop();

    match desktop {
        DesktopKind::Cosmic => {
            cosmic::set_wallpaper(image_abs).context("COSMIC wallpaper")?;
            Ok(true)
        }
        DesktopKind::Gnome => {
            gnome::set_wallpaper(image_abs).context("GNOME wallpaper")?;
            Ok(true)
        }
        DesktopKind::Kde => {
            kde::set_wallpaper(image_abs).context("KDE wallpaper")?;
            Ok(true)
        }
        DesktopKind::Other => Ok(false),
    }
}
