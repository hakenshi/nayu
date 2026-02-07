//! IPC server implementation.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

use anyhow::{anyhow, Context};

use nayu_core::protocol::{Request, Response};
use nayu_core::state::State;

use crate::env_detect::{detect_session, SessionKind};

pub fn run_daemon() -> anyhow::Result<()> {
    match detect_session()? {
        SessionKind::Wayland => run_daemon_wayland(),
        SessionKind::X11 => run_daemon_x11(),
    }
}

fn run_daemon_wayland() -> anyhow::Result<()> {
    let state = Arc::new(Mutex::new(State::default()));
    let (tx, rx) = calloop::channel::channel::<PathBuf>();

    // IPC thread: accept connections, update shared state, notify engine.
    {
        let state = Arc::clone(&state);
        thread::spawn(move || {
            if let Err(err) = run_ipc_server(&state, Some(tx)) {
                if std::env::var_os("NAYU_DEBUG").is_some() {
                    eprintln!("nayu ipc server error: {err:#}");
                }
            }
        });
    }

    // Wayland engine runs on main thread.
    crate::wayland::run_wayland(state, rx)
}

fn run_daemon_x11() -> anyhow::Result<()> {
    let state = Arc::new(Mutex::new(State::default()));
    run_ipc_server(&state, None)
}

fn run_ipc_server(
    state: &Arc<Mutex<State>>,
    notify: Option<calloop::channel::Sender<PathBuf>>,
) -> anyhow::Result<()> {
    let sock = super::socket_path()?;

    if sock.exists() {
        std::fs::remove_file(&sock).with_context(|| format!("remove existing socket {sock:?}"))?;
    }

    let listener = UnixListener::bind(&sock).with_context(|| format!("bind socket {sock:?}"))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&sock, std::fs::Permissions::from_mode(0o600));
    }

    loop {
        let (stream, _addr) = listener.accept().context("accept")?;
        handle_client(stream, state, notify.as_ref())?;
    }
}

fn handle_client(
    stream: UnixStream,
    state: &Arc<Mutex<State>>,
    notify: Option<&calloop::channel::Sender<PathBuf>>,
) -> anyhow::Result<()> {
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
                if let Some(n) = notify {
                    // Best-effort notify.
                    let _ = n.send(s.current_path.clone().expect("set"));
                }
                Response::Ok
            }
        };

        w.write_all(resp.to_line().as_bytes())?;
        w.flush()?;
    }

    Ok(())
}
