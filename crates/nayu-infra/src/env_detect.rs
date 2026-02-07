//! Environment detection (Wayland/X11).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionKind {
    Wayland,
    X11,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesktopKind {
    Cosmic,
    Gnome,
    Kde,
    Other,
}

pub fn detect_session() -> anyhow::Result<SessionKind> {
    let has_wayland = std::env::var_os("WAYLAND_DISPLAY").is_some();
    let has_x11 = std::env::var_os("DISPLAY").is_some();

    if has_wayland {
        return Ok(SessionKind::Wayland);
    }
    if has_x11 {
        return Ok(SessionKind::X11);
    }

    Err(anyhow::anyhow!(
        "unable to detect session (WAYLAND_DISPLAY/DISPLAY missing)"
    ))
}

pub fn detect_desktop() -> DesktopKind {
    // XDG_CURRENT_DESKTOP can be like: "GNOME", "KDE", or "COSMIC:GNOME".
    // We treat it as an ordered list.
    let raw = std::env::var("XDG_CURRENT_DESKTOP")
        .or_else(|_| std::env::var("DESKTOP_SESSION"))
        .unwrap_or_default();

    let mut items: Vec<String> = raw
        .split(|c| c == ':' || c == ';' || c == ',')
        .map(|s| s.trim().to_ascii_lowercase())
        .filter(|s| !s.is_empty())
        .collect();

    if items.is_empty() {
        return DesktopKind::Other;
    }

    // Normalize a few known variants.
    for it in &mut items {
        if it == "ubuntu" {
            continue;
        }
        if it.contains("gnome") {
            return DesktopKind::Gnome;
        }
        if it.contains("kde") || it.contains("plasma") {
            return DesktopKind::Kde;
        }
        if it.contains("cosmic") {
            return DesktopKind::Cosmic;
        }
    }

    DesktopKind::Other
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn with_env<F: FnOnce()>(f: F) {
        let _g = ENV_LOCK.lock().unwrap();

        let old_wayland = std::env::var_os("WAYLAND_DISPLAY");
        let old_display = std::env::var_os("DISPLAY");
        let old_desktop = std::env::var_os("XDG_CURRENT_DESKTOP");

        unsafe {
            std::env::remove_var("WAYLAND_DISPLAY");
            std::env::remove_var("DISPLAY");
            std::env::remove_var("XDG_CURRENT_DESKTOP");
        }

        f();

        unsafe {
            match old_wayland {
                Some(v) => std::env::set_var("WAYLAND_DISPLAY", v),
                None => std::env::remove_var("WAYLAND_DISPLAY"),
            }
            match old_display {
                Some(v) => std::env::set_var("DISPLAY", v),
                None => std::env::remove_var("DISPLAY"),
            }

            match old_desktop {
                Some(v) => std::env::set_var("XDG_CURRENT_DESKTOP", v),
                None => std::env::remove_var("XDG_CURRENT_DESKTOP"),
            }
        }
    }

    #[test]
    fn detect_wayland_wins() {
        with_env(|| {
            unsafe {
                std::env::set_var("DISPLAY", ":0");
                std::env::set_var("WAYLAND_DISPLAY", "wayland-1");
            }
            assert_eq!(detect_session().unwrap(), SessionKind::Wayland);
        });
    }

    #[test]
    fn detect_x11_when_no_wayland() {
        with_env(|| {
            unsafe {
                std::env::set_var("DISPLAY", ":0");
            }
            assert_eq!(detect_session().unwrap(), SessionKind::X11);
        });
    }

    #[test]
    fn detect_errors_when_unknown() {
        with_env(|| {
            let err = detect_session().unwrap_err();
            assert!(format!("{err}").contains("unable to detect session"));
        });
    }

    #[test]
    fn detect_desktop_gnome() {
        with_env(|| {
            unsafe {
                std::env::set_var("XDG_CURRENT_DESKTOP", "GNOME");
            }
            assert_eq!(detect_desktop(), DesktopKind::Gnome);
        });
    }

    #[test]
    fn detect_desktop_cosmic() {
        with_env(|| {
            unsafe {
                std::env::set_var("XDG_CURRENT_DESKTOP", "COSMIC:GNOME");
            }
            assert_eq!(detect_desktop(), DesktopKind::Cosmic);
        });
    }
}
