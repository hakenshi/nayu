//! IPC server/client.

use std::path::{Path, PathBuf};

use anyhow::anyhow;

pub mod client;
pub mod server;

pub(crate) fn socket_path() -> anyhow::Result<PathBuf> {
    if let Some(p) = std::env::var_os("NAYU_SOCKET_PATH") {
        return Ok(PathBuf::from(p));
    }

    let dir =
        std::env::var_os("XDG_RUNTIME_DIR").ok_or_else(|| anyhow!("XDG_RUNTIME_DIR is not set"))?;
    Ok(Path::new(&dir).join("nayu.sock"))
}
