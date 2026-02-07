# Errors (Implementer Notes)

- Default: one-line error.
- Debug: full chain with `NAYU_DEBUG=1`.

## Context guidelines

In debug mode, include:

- engine selected (Wayland/X11)
- socket path (IPC)
- command executed (ffmpeg) with exit status
- stderr excerpt for ffmpeg failures
