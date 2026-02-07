use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};

use nayu_core::protocol::Response;
static ENV_LOCK: Mutex<()> = Mutex::new(());

fn with_env<F: FnOnce()>(sock: &std::path::Path, f: F) {
    let old_socket = std::env::var_os("NAYU_SOCKET_PATH");
    let old_wayland = std::env::var_os("WAYLAND_DISPLAY");
    let old_display = std::env::var_os("DISPLAY");

    unsafe {
        std::env::set_var("NAYU_SOCKET_PATH", sock);
        std::env::remove_var("WAYLAND_DISPLAY");
        std::env::set_var("DISPLAY", ":0");
    }

    f();

    unsafe {
        match old_socket {
            Some(v) => std::env::set_var("NAYU_SOCKET_PATH", v),
            None => std::env::remove_var("NAYU_SOCKET_PATH"),
        }
        match old_wayland {
            Some(v) => std::env::set_var("WAYLAND_DISPLAY", v),
            None => std::env::remove_var("WAYLAND_DISPLAY"),
        }
        match old_display {
            Some(v) => std::env::set_var("DISPLAY", v),
            None => std::env::remove_var("DISPLAY"),
        }
    }
}

fn wait_for_socket(path: &std::path::Path) {
    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline {
        if path.exists() {
            return;
        }
        thread::sleep(Duration::from_millis(20));
    }
    panic!("socket did not appear: {path:?}");
}

fn send_line(sock: &std::path::Path, line: &str) -> String {
    let mut stream = UnixStream::connect(sock).unwrap();
    stream.write_all(line.as_bytes()).unwrap();
    stream.flush().unwrap();

    let mut reader = BufReader::new(stream);
    let mut resp = String::new();
    reader.read_line(&mut resp).unwrap();
    resp
}

#[test]
fn ipc_server_smoke_ping_status_set() {
    let _g = ENV_LOCK.lock().unwrap();

    let dir = tempfile::tempdir().unwrap();
    let sock = dir.path().join("nayu.sock");

    with_env(&sock, || {
        // Spawn daemon.
        thread::spawn(|| {
            nayu_infra::ipc::server::run_daemon().unwrap();
        });

        wait_for_socket(&sock);

        let line = send_line(&sock, "PING\n");
        assert_eq!(Response::parse_line(&line).unwrap(), Response::Ok);

        let line = send_line(&sock, "STATUS\n");
        let r = Response::parse_line(&line).unwrap();
        assert!(matches!(r, Response::OkMsg(_)));

        let img = dir.path().join("img.png");
        std::fs::write(&img, b"fake").unwrap();
        let line = send_line(&sock, &format!("SET {}\n", img.display()));
        assert_eq!(Response::parse_line(&line).unwrap(), Response::Ok);

        let line = send_line(&sock, "STATUS\n");
        let r = Response::parse_line(&line).unwrap();
        match r {
            Response::OkMsg(msg) => assert!(msg.contains(&img.display().to_string())),
            other => panic!("unexpected response: {other:?}"),
        }
    });
}

#[test]
fn ipc_client_set_and_status_work_when_server_running() {
    let _g = ENV_LOCK.lock().unwrap();

    let dir = tempfile::tempdir().unwrap();
    let sock = dir.path().join("nayu.sock");

    with_env(&sock, || {
        thread::spawn(|| {
            nayu_infra::ipc::server::run_daemon().unwrap();
        });

        wait_for_socket(&sock);

        let img = dir.path().join("img.png");
        std::fs::write(&img, b"fake").unwrap();

        // Use a relative path to cover the absolute-join logic.
        let old_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(dir.path()).unwrap();
        nayu_infra::ipc::client::set(PathBuf::from("img.png")).unwrap();
        std::env::set_current_dir(old_cwd).unwrap();

        // Smoke: just ensure it doesn't error.
        nayu_infra::ipc::client::status().unwrap();
    });
}
