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

## Engine selection

Selection is based on session type:

- Wayland if `WAYLAND_DISPLAY` is set.
- X11 if `DISPLAY` is set.
- If neither is set: error.

## CLI surface (v0)

- `nayu daemon`
  - starts IPC server
  - selects engine
  - waits for IPC commands

- `nayu set <path>`
  - validates path exists
  - on Wayland: send IPC SET, autostart daemon if needed
  - on X11: either send IPC SET (if daemon exists) or set directly

## Decoding

We use ffmpeg to decode image formats to pixels.
Avoid pulling in heavy image libraries.

## Scaling

Default scaling mode is `cover/fill`.
We will keep the first implementation minimal and correct.
