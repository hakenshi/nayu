# Architecture (Implementer Notes)

We split `nayu` into a control plane and a platform engine.

- Control plane:
  - CLI parsing
  - IPC server/client
  - state (current wallpaper path, scaling mode)
  - decoding pixels via ffmpeg (recommended)
- Platform engine:
  - Wayland engine (layer-shell background surfaces)
  - X11 engine (root pixmap + standard properties)

Do not mix platform-specific code into shared control-plane modules.
