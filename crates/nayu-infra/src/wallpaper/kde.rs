use std::path::Path;
use std::process::Command;

use anyhow::{anyhow, Context};

fn file_uri(path: &Path) -> anyhow::Result<String> {
    let s = path
        .to_str()
        .ok_or_else(|| anyhow!("path is not valid UTF-8"))?;
    Ok(format!("file://{s}"))
}

fn find_qdbus() -> Option<&'static str> {
    // Plasma 6 often ships qdbus6, Plasma 5 ships qdbus.
    for exe in ["qdbus6", "qdbus"] {
        if Command::new(exe).arg("--version").output().is_ok() {
            return Some(exe);
        }
    }
    None
}

pub fn set_wallpaper(image_abs: &Path) -> anyhow::Result<()> {
    let qdbus = find_qdbus().ok_or_else(|| anyhow!("qdbus not found (qdbus6/qdbus)"))?;
    let uri = file_uri(image_abs)?;

    // Based on the standard PlasmaShell JS API.
    let script = format!(
        "var allDesktops = desktops();\n\
         for (var i = 0; i < allDesktops.length; i++) {{\n\
           var d = allDesktops[i];\n\
           d.wallpaperPlugin = 'org.kde.image';\n\
           d.currentConfigGroup = ['Wallpaper', 'org.kde.image', 'General'];\n\
           d.writeConfig('Image', '{uri}');\n\
         }}\n"
    );

    let status = Command::new(qdbus)
        .arg("org.kde.plasmashell")
        .arg("/PlasmaShell")
        .arg("org.kde.PlasmaShell.evaluateScript")
        .arg(script)
        .status()
        .with_context(|| format!("run {qdbus} PlasmaShell.evaluateScript"))?;

    if !status.success() {
        return Err(anyhow!("qdbus wallpaper script failed"));
    }

    Ok(())
}
