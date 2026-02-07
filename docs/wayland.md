# Wayland (Implementer Notes)

Minimal v0 scope:

- Layer-shell background surfaces for each output.
- Draw decoded wallpaper using `cover/fill` scaling.
- Stay alive to keep wallpaper visible.

## Protocols

We require layer-shell for background placement.

## Multi-output

- One surface per output.
- Each surface gets the same wallpaper image in v0.

## Scaling

Scaling mode: cover/fill.
Definition:

- Preserve aspect ratio.
- Scale to fully cover the output.
- Center-crop overflow.
