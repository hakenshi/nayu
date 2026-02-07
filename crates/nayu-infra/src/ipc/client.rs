//! IPC client implementation.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Context};

use nayu_core::protocol::{Request, Response};

use crate::env_detect::{detect_session, SessionKind};
use crate::wallpaper;

fn connect() -> anyhow::Result<UnixStream> {
    let sock = super::socket_path()?;
    UnixStream::connect(&sock).with_context(|| format!("connect {sock:?}"))
}

fn send(req: Request) -> anyhow::Result<Response> {
    let mut stream = connect()?;
    let line = match req {
        Request::Ping => "PING\n".to_string(),
        Request::Status => "STATUS\n".to_string(),
        Request::Set { path } => format!("SET {}\n", path.display()),
    };

    stream.write_all(line.as_bytes()).context("write")?;
    stream.flush().context("flush")?;

    let mut reader = BufReader::new(stream);
    let mut resp_line = String::new();
    reader.read_line(&mut resp_line).context("read response")?;
    Response::parse_line(&resp_line)
}

pub fn status() -> anyhow::Result<()> {
    match send(Request::Status)? {
        Response::Ok => {
            println!("OK");
            Ok(())
        }
        Response::OkMsg(msg) => {
            println!("OK {msg}");
            Ok(())
        }
        Response::Err(msg) => Err(anyhow!("{msg}")),
    }
}

pub fn set(image: PathBuf) -> anyhow::Result<()> {
    // v0: accept relative, but store/send absolute.
    let abs = if image.is_absolute() {
        image
    } else {
        std::env::current_dir().context("cwd")?.join(image)
    };

    if !abs.exists() {
        return Err(anyhow!("image path does not exist")).with_context(|| format!("{abs:?}"));
    }

    // If we're on a supported desktop environment, prefer direct set (no daemon required).
    // This makes it possible to use nayu standalone on COSMIC/GNOME/KDE.
    if wallpaper::try_set_direct(&abs)? {
        return Ok(());
    }

    match send(Request::Set { path: abs.clone() }) {
        Ok(Response::Ok) => return Ok(()),
        Ok(Response::OkMsg(_)) => return Ok(()),
        Ok(Response::Err(msg)) => return Err(anyhow!("{msg}")),
        Err(err) => {
            // If IPC fails, autostart daemon on Wayland.
            if detect_session()? == SessionKind::Wayland {
                autostart_daemon()?;
                match send(Request::Set { path: abs })? {
                    Response::Ok | Response::OkMsg(_) => Ok(()),
                    Response::Err(msg) => Err(anyhow!("{msg}")),
                }
            } else {
                // X11: direct set is not implemented yet.
                Err(err).context("daemon not running; X11 direct set not implemented")
            }
        }
    }
}

fn autostart_daemon() -> anyhow::Result<()> {
    let exe = std::env::current_exe().context("current_exe")?;

    let mut cmd = Command::new(exe);
    cmd.arg("daemon")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    cmd.spawn().context("spawn daemon")?;

    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline {
        if connect().is_ok() {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(50));
    }

    Err(anyhow!("daemon did not become ready"))
}
