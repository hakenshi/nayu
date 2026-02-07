//! nayu-infra: OS adapters (Wayland/X11 engines, IPC server, ffmpeg).

pub mod env_detect;
pub mod ffmpeg_decode;
pub mod process_runner;
pub mod wallpaper;

pub mod ipc;
pub mod wayland;
pub mod x11;

// Re-export common entrypoints used by CLI.
pub use ipc::{client, server};
