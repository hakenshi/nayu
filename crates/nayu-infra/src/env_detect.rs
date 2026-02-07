//! Environment detection (Wayland/X11).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionKind {
    Wayland,
    X11,
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
