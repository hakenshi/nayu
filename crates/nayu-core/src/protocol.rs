//! IPC protocol parsing/formatting.

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
    Set { path: PathBuf },
    Ping,
    Status,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Response {
    Ok,
    OkMsg(String),
    Err(String),
}

impl Request {
    pub fn parse_line(line: &str) -> anyhow::Result<Self> {
        let line = line.trim_end_matches(['\r', '\n']);
        if line == "PING" {
            return Ok(Self::Ping);
        }
        if line == "STATUS" {
            return Ok(Self::Status);
        }

        if let Some(rest) = line.strip_prefix("SET ") {
            let path = rest.trim();
            if path.is_empty() {
                return Err(anyhow!("SET requires a path"));
            }
            let pb = PathBuf::from(path);
            return Ok(Self::Set { path: pb });
        }

        Err(anyhow!("unknown_command"))
    }
}

impl Response {
    pub fn to_line(&self) -> String {
        match self {
            Self::Ok => "OK\n".to_string(),
            Self::OkMsg(msg) => format!("OK {msg}\n"),
            Self::Err(msg) => format!("ERR {msg}\n"),
        }
    }

    pub fn parse_line(line: &str) -> anyhow::Result<Self> {
        let line = line.trim_end_matches(['\r', '\n']);

        if line == "OK" {
            return Ok(Self::Ok);
        }
        if let Some(rest) = line.strip_prefix("OK ") {
            return Ok(Self::OkMsg(rest.to_string()));
        }
        if let Some(rest) = line.strip_prefix("ERR ") {
            return Ok(Self::Err(rest.to_string()));
        }

        Err(anyhow!("invalid_response")).with_context(|| format!("line: {line:?}"))
    }
}

pub fn validate_abs_path(path: &Path) -> anyhow::Result<()> {
    if !path.is_absolute() {
        return Err(anyhow!("path must be absolute")).with_context(|| format!("{path:?}"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_parse_ping_status() {
        assert_eq!(Request::parse_line("PING").unwrap(), Request::Ping);
        assert_eq!(Request::parse_line("PING\n").unwrap(), Request::Ping);
        assert_eq!(Request::parse_line("STATUS").unwrap(), Request::Status);
        assert_eq!(Request::parse_line("STATUS\r\n").unwrap(), Request::Status);
    }

    #[test]
    fn request_parse_set() {
        let r = Request::parse_line("SET /tmp/x.png").unwrap();
        assert_eq!(
            r,
            Request::Set {
                path: PathBuf::from("/tmp/x.png")
            }
        );

        let r = Request::parse_line("SET   /tmp/y.png  \n").unwrap();
        assert_eq!(
            r,
            Request::Set {
                path: PathBuf::from("/tmp/y.png")
            }
        );
    }

    #[test]
    fn request_parse_errors() {
        let err = Request::parse_line("SET  \n").unwrap_err();
        assert!(format!("{err}").contains("SET requires a path"));

        let err = Request::parse_line("NOPE").unwrap_err();
        assert!(format!("{err}").contains("unknown_command"));
    }

    #[test]
    fn response_roundtrip_ok() {
        assert_eq!(
            Response::parse_line(&Response::Ok.to_line()).unwrap(),
            Response::Ok
        );
        assert_eq!(
            Response::parse_line(&Response::OkMsg("hello".into()).to_line()).unwrap(),
            Response::OkMsg("hello".into())
        );
        assert_eq!(
            Response::parse_line(&Response::Err("bad".into()).to_line()).unwrap(),
            Response::Err("bad".into())
        );
    }

    #[test]
    fn response_parse_errors() {
        let err = Response::parse_line("WHAT\n").unwrap_err();
        assert!(format!("{err:#}").contains("invalid_response"));
    }

    #[test]
    fn validate_abs_path_enforces_absolute() {
        validate_abs_path(Path::new("/tmp/ok")).unwrap();
        assert!(validate_abs_path(Path::new("relative.png")).is_err());
    }
}
