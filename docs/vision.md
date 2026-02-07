# Vision (Implementer Notes)

## Goal

`nayu` is a wallpaper setter/engine that works on both Wayland and X11.

It supports:

- `nayu set <path>` (one-shot client UX)
- `nayu daemon` (long-running, required for Wayland persistence)

Wayland default scaling is `cover/fill`.

## Invariants

- Works on Wayland and X11.
- Wayland `set` must auto-start the daemon if it isn't running.
- IPC protocol is stable and documented in `docs/ipc.md`.
- Error UX: one-line by default; verbose chain only with `NAYU_DEBUG=1`.

## Non-goals (v0)

- No transitions/effects.
- No per-output wallpapers.
