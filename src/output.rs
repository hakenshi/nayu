//! CLI output formatting.

pub fn debug_enabled() -> bool {
    std::env::var_os("NAYU_DEBUG").is_some_and(|v| !v.is_empty())
}

pub fn print_error(err: &anyhow::Error) {
    if debug_enabled() {
        eprintln!("{err:#}");
    } else {
        // Best-effort single line.
        eprintln!("{err}");
    }
}
