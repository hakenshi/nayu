//! Daemon state (current wallpaper path, scaling mode).

use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct State {
    pub current_path: Option<PathBuf>,
    pub mode: ScalingMode,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum ScalingMode {
    /// Preserve aspect ratio, crop to fill output.
    #[default]
    Cover,
}
