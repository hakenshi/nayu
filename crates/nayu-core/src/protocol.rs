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
