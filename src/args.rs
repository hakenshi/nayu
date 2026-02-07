//! CLI argument definitions.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "nayu")]
#[command(about = "Wallpaper engine for Wayland and X11", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Set wallpaper to the given image path.
    Set {
        /// Absolute or relative path to an image file.
        image: PathBuf,
    },

    /// Run background daemon (required for Wayland persistence).
    Daemon,

    /// Check if daemon is running.
    Status,
}
