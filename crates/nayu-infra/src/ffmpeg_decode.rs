//! Decode images to pixels via ffmpeg.

use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{anyhow, Context};

/// Decode a single frame into BGRA bytes using a "cover" scaling strategy.
///
/// This matches WL_SHM_FORMAT_ARGB8888 memory layout on little-endian systems.
pub fn decode_bgra_cover(path: &Path, width: u32, height: u32) -> anyhow::Result<Vec<u8>> {
    let w = width;
    let h = height;

    let vf = format!("scale={w}:{h}:force_original_aspect_ratio=increase,crop={w}:{h}");

    let ffmpeg = std::env::var_os("NAYU_FFMPEG_BIN").unwrap_or_else(|| "ffmpeg".into());

    let child = Command::new(ffmpeg)
        .arg("-v")
        .arg("error")
        .arg("-i")
        .arg(path)
        .arg("-vf")
        .arg(vf)
        .arg("-frames:v")
        .arg("1")
        .arg("-f")
        .arg("rawvideo")
        .arg("-pix_fmt")
        .arg("bgra")
        .arg("-")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("spawn ffmpeg")?;

    let output = child.wait_with_output().context("wait ffmpeg")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("ffmpeg decode failed")).with_context(|| stderr.to_string());
    }

    let expected = (w as usize) * (h as usize) * 4;
    if output.stdout.len() != expected {
        return Err(anyhow!("ffmpeg returned unexpected size"))
            .with_context(|| format!("expected={expected} got={}", output.stdout.len()));
    }

    Ok(output.stdout)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use std::sync::Mutex;

    const PNG_1X1: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90,
        0x77, 0x53, 0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x08, 0xD7, 0x63, 0xF8,
        0xCF, 0xC0, 0x00, 0x00, 0x03, 0x01, 0x01, 0x00, 0x18, 0xDD, 0x8D, 0xB5, 0x00, 0x00, 0x00,
        0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn write_exe(path: &std::path::Path, body: &str) {
        std::fs::write(path, body).unwrap();
        let mut perm = std::fs::metadata(path).unwrap().permissions();
        perm.set_mode(0o755);
        std::fs::set_permissions(path, perm).unwrap();
    }

    #[test]
    fn decode_errors_on_nonzero_exit() {
        let _g = ENV_LOCK.lock().unwrap();

        let dir = tempfile::tempdir().unwrap();
        let fake = dir.path().join("ffmpeg");
        write_exe(
            &fake,
            "#!/bin/sh\n\n# fake ffmpeg\necho 'boom' 1>&2\nexit 1\n",
        );

        let old = std::env::var_os("NAYU_FFMPEG_BIN");
        unsafe { std::env::set_var("NAYU_FFMPEG_BIN", &fake) };

        let img = dir.path().join("img.png");
        std::fs::write(&img, PNG_1X1).unwrap();

        let err = decode_bgra_cover(&img, 16, 16).unwrap_err();
        let s = format!("{err:#}");
        assert!(s.contains("ffmpeg decode failed"));
        assert!(s.contains("boom"));

        unsafe {
            match old {
                Some(v) => std::env::set_var("NAYU_FFMPEG_BIN", v),
                None => std::env::remove_var("NAYU_FFMPEG_BIN"),
            }
        }
    }

    #[test]
    fn decode_1x1_png() {
        let _g = ENV_LOCK.lock().unwrap();

        let dir = tempfile::tempdir().unwrap();
        let fake = dir.path().join("ffmpeg");
        // Write exactly 4 bytes and succeed.
        write_exe(
            &fake,
            "#!/bin/sh\n\n# fake ffmpeg\nhead -c 4 /dev/zero\nexit 0\n",
        );

        let old = std::env::var_os("NAYU_FFMPEG_BIN");
        unsafe { std::env::set_var("NAYU_FFMPEG_BIN", &fake) };

        let p = dir.path().join("tiny.png");
        std::fs::write(&p, PNG_1X1).unwrap();

        let bytes = decode_bgra_cover(&p, 1, 1).unwrap();
        assert_eq!(bytes.len(), 4);

        unsafe {
            match old {
                Some(v) => std::env::set_var("NAYU_FFMPEG_BIN", v),
                None => std::env::remove_var("NAYU_FFMPEG_BIN"),
            }
        }
    }

    #[test]
    fn decode_errors_on_unexpected_size() {
        let _g = ENV_LOCK.lock().unwrap();

        let dir = tempfile::tempdir().unwrap();
        let fake = dir.path().join("ffmpeg");
        // Write 4 bytes but ask for 2x2 (= 16 bytes expected).
        write_exe(
            &fake,
            "#!/bin/sh\n\n# fake ffmpeg\nhead -c 4 /dev/zero\nexit 0\n",
        );

        let old = std::env::var_os("NAYU_FFMPEG_BIN");
        unsafe { std::env::set_var("NAYU_FFMPEG_BIN", &fake) };

        let p = dir.path().join("tiny.png");
        std::fs::write(&p, PNG_1X1).unwrap();

        let err = decode_bgra_cover(&p, 2, 2).unwrap_err();
        assert!(format!("{err:#}").contains("unexpected size"));

        unsafe {
            match old {
                Some(v) => std::env::set_var("NAYU_FFMPEG_BIN", v),
                None => std::env::remove_var("NAYU_FFMPEG_BIN"),
            }
        }
    }
}
