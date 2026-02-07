set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

default:
  @just --list

build:
  cargo build

daemon:
  cargo run -- daemon

set IMG:
  cargo run -- set "{{IMG}}"

test:
  cargo test --workspace

cov:
  cargo llvm-cov --workspace --fail-under-lines 80

fmt:
  cargo fmt --all

clippy:
  cargo clippy --workspace --all-targets -- -D warnings
