use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context};

fn config_home() -> anyhow::Result<PathBuf> {
    if let Some(v) = std::env::var_os("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(v));
    }
    let home = std::env::var_os("HOME").ok_or_else(|| anyhow!("HOME not set"))?;
    Ok(PathBuf::from(home).join(".config"))
}

fn atomic_write(path: &Path, bytes: &[u8]) -> anyhow::Result<()> {
    let dir = path.parent().ok_or_else(|| anyhow!("invalid path"))?;
    fs::create_dir_all(dir).with_context(|| format!("create dir {}", dir.display()))?;

    let tmp = dir.join(format!(
        ".{}.tmp",
        path.file_name().and_then(|s| s.to_str()).unwrap_or("nayu")
    ));
    fs::write(&tmp, bytes).with_context(|| format!("write {}", tmp.display()))?;
    fs::rename(&tmp, path)
        .with_context(|| format!("rename {} -> {}", tmp.display(), path.display()))?;
    Ok(())
}

pub fn set_wallpaper(image_abs: &Path) -> anyhow::Result<()> {
    // Contract mirrored from rze/docs/integrations.md.
    // cosmic-bg watches cosmic-config files.
    let base = config_home()?.join("cosmic/com.system76.CosmicBackground/v1");
    let same_on_all = base.join("same-on-all");
    let all = base.join("all");

    // Force same wallpaper on all displays.
    atomic_write(&same_on_all, b"true\n").context("write same-on-all")?;

    // Update the `source: Path("...")` within the existing config blob.
    let mut text = fs::read_to_string(&all).with_context(|| {
        format!(
            "read {} (open COSMIC Wallpaper settings once)",
            all.display()
        )
    })?;

    let needle = "source: Path(\"";
    let Some(start) = text.find(needle) else {
        return Err(anyhow!("COSMIC config: missing `source: Path(\"...\")`"));
    };
    let after = &text[start + needle.len()..];
    let Some(end_rel) = after.find("\")") else {
        return Err(anyhow!("COSMIC config: unterminated source path"));
    };
    let path_start = start + needle.len();
    let path_end = path_start + end_rel;

    let new_path = image_abs
        .to_str()
        .ok_or_else(|| anyhow!("path is not valid UTF-8"))?;

    text.replace_range(path_start..path_end, new_path);

    atomic_write(&all, text.as_bytes()).context("write all")?;
    Ok(())
}
