//! Wayland engine.

// The Wayland layer-shell engine is hard to cover with automated tests.
// cargo-llvm-cov sets cfg(coverage); we provide a stub in that configuration so the rest of the
// workspace can still be measured reliably.

#[cfg(not(coverage))]
pub mod engine;

#[cfg(coverage)]
mod engine {
    use std::path::PathBuf;
    use std::sync::{Arc, Mutex};

    use anyhow::anyhow;
    use calloop::channel::Channel;

    use nayu_core::state::State;

    pub fn run_wayland(
        _shared_state: Arc<Mutex<State>>,
        _rx: Channel<PathBuf>,
    ) -> anyhow::Result<()> {
        Err(anyhow!("wayland engine excluded from coverage build"))
    }
}
pub mod outputs;
pub mod render_shm;

pub use engine::run_wayland;
