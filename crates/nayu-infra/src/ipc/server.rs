//! IPC server implementation.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Context};

use nayu_core::protocol::{Request, Response};
use nayu_core::state::State;

fn socket_path() -> anyhow::Result<PathBuf> {
    let dir =
        std::env::var_os("XDG_RUNTIME_DIR").ok_or_else(|| anyhow!("XDG_RUNTIME_DIR is not set"))?;
    Ok(Path::new(&dir).join("nayu.sock"))
}

pub fn run_daemon() -> anyhow::Result<()> {
    let sock = socket_path()?;

    // Ensure old socket is gone.
    if sock.exists() {
        std::fs::remove_file(&sock).with_context(|| format!("remove existing socket {sock:?}"))?;
    }

    let listener = UnixListener::bind(&sock).with_context(|| format!("bind socket {sock:?}"))?;

    // Best-effort perms: user-only.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&sock, std::fs::Permissions::from_mode(0o600));
    }

    let state = Arc::new(Mutex::new(State::default()));

    loop {
        let (stream, _addr) = listener.accept().context("accept")?;
        handle_client(stream, &state)?;
    }
}

fn handle_client(stream: UnixStream, state: &Arc<Mutex<State>>) -> anyhow::Result<()> {
    let mut w = stream.try_clone().context("clone stream")?;
    let r = BufReader::new(stream);

    for line in r.lines() {
        let line = line.context("read line")?;
        let req = match Request::parse_line(&line) {
            Ok(r) => r,
            Err(_) => {
                w.write_all(Response::Err("unknown_command".into()).to_line().as_bytes())?;
                w.flush()?;
                continue;
            }
        };

        let resp = match req {
            Request::Ping => Response::Ok,
            Request::Status => {
                let s = state.lock().map_err(|_| anyhow!("state lock poisoned"))?;
                let msg = match &s.current_path {
                    Some(p) => format!("mode=cover path={}", p.display()),
                    None => "mode=cover path=<unset>".to_string(),
                };
                Response::OkMsg(msg)
            }
            Request::Set { path } => {
                // Store only. Engine application comes later.
                let mut s = state.lock().map_err(|_| anyhow!("state lock poisoned"))?;
                s.current_path = Some(path);
                Response::Ok
            }
        };

        w.write_all(resp.to_line().as_bytes())?;
        w.flush()?;
    }

    Ok(())
}
