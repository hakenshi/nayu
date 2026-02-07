# Diagrams (Implementer Notes)

## Set flow

set -> try IPC -> if missing and Wayland: start daemon -> SET

```text
nayu set <path>
  -> if WAYLAND_DISPLAY:
       try connect $XDG_RUNTIME_DIR/nayu.sock
       if ok: SET
       else: spawn nayu daemon, wait, then SET
  -> else if DISPLAY:
       set root pixmap (or IPC if daemon exists)
  -> else:
       error
```
