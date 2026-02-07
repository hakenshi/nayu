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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn debug_enabled_tracks_env() {
        let _g = ENV_LOCK.lock().unwrap();
        let old = std::env::var_os("NAYU_DEBUG");

        unsafe {
            std::env::remove_var("NAYU_DEBUG");
        }
        assert!(!debug_enabled());

        unsafe {
            std::env::set_var("NAYU_DEBUG", "1");
        }
        assert!(debug_enabled());

        unsafe {
            std::env::set_var("NAYU_DEBUG", "");
        }
        assert!(!debug_enabled());

        unsafe {
            match old {
                Some(v) => std::env::set_var("NAYU_DEBUG", v),
                None => std::env::remove_var("NAYU_DEBUG"),
            }
        }
    }
}
