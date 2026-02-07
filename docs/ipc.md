# IPC (Implementer Contract)

## Socket

- Path: `$XDG_RUNTIME_DIR/nayu.sock`

## Protocol

Line-based UTF-8 commands.

### Commands

- `SET <absolute_path>\n`
- `PING\n`
- `STATUS\n`

### Responses

- `OK\n`
- `ERR <message>\n`

## `nayu set <path>` behavior

1. Try connect to the socket.
2. If connect succeeds: send `SET`.
3. If connect fails:
   - On Wayland: auto-start daemon, wait for socket responsiveness, then send `SET`.
   - On X11: may set wallpaper directly and exit (daemon optional).
