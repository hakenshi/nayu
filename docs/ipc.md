# IPC (Implementer Contract)

## Socket

- Path: `$XDG_RUNTIME_DIR/nayu.sock`

Server must create the parent directory if missing.
Socket permissions should be user-only.

## Protocol

Line-based UTF-8 commands.

Paths in commands are absolute filesystem paths. We do not accept relative paths.
Whitespace rules:

- Command verb is first token.
- For `SET`, everything after the first space is treated as the path (no quoting in v0).

This keeps the protocol easy to debug with `socat`.

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

## Timeouts

- Client should not hang forever.
- When auto-starting daemon, retry connect for a short bounded window (seconds, not minutes).

## Forward-compatibility

- Unknown commands return `ERR unknown_command`.
- Unknown responses are treated as errors.
