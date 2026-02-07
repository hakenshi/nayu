//! Wayland layer-shell background engine.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{anyhow, Context};

use smithay_client_toolkit::compositor::{CompositorHandler, CompositorState};
use smithay_client_toolkit::delegate_compositor;
use smithay_client_toolkit::delegate_layer;
use smithay_client_toolkit::delegate_output;
use smithay_client_toolkit::delegate_registry;
use smithay_client_toolkit::delegate_shm;
use smithay_client_toolkit::output::{OutputHandler, OutputState};
use smithay_client_toolkit::reexports::calloop::EventLoop;
use smithay_client_toolkit::reexports::calloop_wayland_source::WaylandSource;
use smithay_client_toolkit::reexports::client as wayland_client;
use smithay_client_toolkit::registry::{ProvidesRegistryState, RegistryState};
use smithay_client_toolkit::registry_handlers;
use smithay_client_toolkit::shell::wlr_layer::{
    Anchor, KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface,
    LayerSurfaceConfigure,
};
use smithay_client_toolkit::shell::WaylandSurface;
use smithay_client_toolkit::shm::{
    slot::{Buffer, SlotPool},
    Shm, ShmHandler,
};
use wayland_client::globals::registry_queue_init;
use wayland_client::protocol::{wl_output, wl_shm, wl_surface};
use wayland_client::{Connection, QueueHandle};

use calloop::channel::{Channel, Event};

use nayu_core::state::State;

use crate::ffmpeg_decode;

pub fn run_wayland(shared_state: Arc<Mutex<State>>, rx: Channel<PathBuf>) -> anyhow::Result<()> {
    let conn = Connection::connect_to_env().context("connect to Wayland")?;
    let (globals, event_queue) =
        registry_queue_init::<NayuWayland>(&conn).context("registry init")?;
    let qh = event_queue.handle();

    let mut event_loop: EventLoop<NayuWayland> = EventLoop::try_new().context("event loop")?;
    let loop_handle = event_loop.handle();
    WaylandSource::new(conn.clone(), event_queue)
        .insert(loop_handle.clone())
        .context("insert Wayland source")?;

    let compositor = CompositorState::bind(&globals, &qh).context("wl_compositor")?;
    let layer_shell = LayerShell::bind(&globals, &qh).context("layer shell")?;
    let shm = Shm::bind(&globals, &qh).context("wl_shm")?;

    let pool = SlotPool::new(256 * 256 * 4, &shm).context("create shm pool")?;

    let mut app = NayuWayland {
        registry_state: RegistryState::new(&globals),
        output_state: OutputState::new(&globals, &qh),
        compositor,
        layer_shell,
        shm,
        pool,
        shared_state,
        surfaces: HashMap::new(),
        redraw: false,
    };

    // IPC channel: mark redraw and update shared state path.
    {
        let st = Arc::clone(&app.shared_state);
        event_loop
            .handle()
            .insert_source(rx, move |event, _, app| {
                if let Event::Msg(path) = event {
                    if let Ok(mut s) = st.lock() {
                        s.current_path = Some(path);
                    }
                    app.redraw = true;
                }
            })
            .map_err(|e| anyhow!("insert ipc channel: {e}"))?;
    }

    // Create layer surfaces for already-known outputs.
    for output in app.output_state.outputs() {
        app.create_layer(&qh, output);
    }

    loop {
        event_loop.dispatch(Duration::from_millis(16), &mut app)?;

        // Perform redraws on the main thread.
        if app.redraw {
            app.redraw = false;
            app.draw_all(&qh);
        }
    }
}

struct NayuWayland {
    registry_state: RegistryState,
    output_state: OutputState,
    compositor: CompositorState,
    layer_shell: LayerShell,
    shm: Shm,
    pool: SlotPool,

    shared_state: Arc<Mutex<State>>,

    surfaces: HashMap<wl_surface::WlSurface, SurfaceEntry>,
    redraw: bool,
}

struct SurfaceEntry {
    output: wl_output::WlOutput,
    layer: LayerSurface,
    size: (u32, u32),
    last_buffer: Option<Buffer>,
}

impl NayuWayland {
    fn create_layer(&mut self, qh: &QueueHandle<Self>, output: wl_output::WlOutput) {
        // One layer surface per output.
        let surface = self.compositor.create_surface(qh);
        let key = surface.clone();
        let layer = self.layer_shell.create_layer_surface(
            qh,
            surface,
            Layer::Background,
            Some("nayu"),
            Some(&output),
        );

        layer.set_anchor(Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT);
        layer.set_keyboard_interactivity(KeyboardInteractivity::None);
        layer.set_size(0, 0);
        layer.set_exclusive_zone(-1);

        layer.commit();
        self.surfaces.insert(
            key,
            SurfaceEntry {
                output,
                layer,
                size: (0, 0),
                last_buffer: None,
            },
        );
    }

    fn draw_all(&mut self, qh: &QueueHandle<Self>) {
        let current = {
            let s = match self.shared_state.lock() {
                Ok(s) => s,
                Err(_) => return,
            };
            s.current_path.clone()
        };

        let Some(path) = current else { return };

        let surfaces: Vec<wl_surface::WlSurface> = self.surfaces.keys().cloned().collect();
        for surface in surfaces {
            let Some(entry) = self.surfaces.get_mut(&surface) else {
                continue;
            };
            let (w, h) = entry.size;
            if w == 0 || h == 0 {
                continue;
            }
            let pool = &mut self.pool;
            if let Err(err) = draw_one(pool, qh, entry, &path, w, h) {
                if std::env::var_os("NAYU_DEBUG").is_some() {
                    eprintln!("nayu draw error: {err:#}");
                }
            }
        }
    }
}

fn draw_one(
    pool: &mut SlotPool,
    _qh: &QueueHandle<NayuWayland>,
    entry: &mut SurfaceEntry,
    path: &PathBuf,
    width: u32,
    height: u32,
) -> anyhow::Result<()> {
    let bytes = ffmpeg_decode::decode_bgra_cover(path, width, height)
        .with_context(|| format!("decode {} to {width}x{height}", path.display()))?;

    let stride = (width * 4) as i32;
    let (buffer, canvas) = pool
        .create_buffer(
            width as i32,
            height as i32,
            stride,
            wl_shm::Format::Argb8888,
        )
        .context("create shm buffer")?;

    canvas.copy_from_slice(&bytes);

    let wl_surface = entry.layer.wl_surface();
    buffer
        .attach_to(wl_surface)
        .map_err(|_| anyhow!("activate shm buffer"))?;
    wl_surface.damage_buffer(0, 0, width as i32, height as i32);
    wl_surface.commit();

    // Keep newest buffer alive; dropping the previous one is safe (it will be destroyed on server
    // release if still active).
    entry.last_buffer = Some(buffer);
    Ok(())
}

impl ProvidesRegistryState for NayuWayland {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }

    registry_handlers!(OutputState);
}

impl CompositorHandler for NayuWayland {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_factor: i32,
    ) {
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wl_output::Transform,
    ) {
    }

    fn frame(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _time: u32,
    ) {
        // We do not animate. Redraw is triggered by configure/SET.
    }

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }
}

impl OutputHandler for NayuWayland {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        output: wl_output::WlOutput,
    ) {
        self.create_layer(qh, output);
        self.redraw = true;
    }

    fn update_output(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _output: wl_output::WlOutput,
    ) {
    }

    fn output_destroyed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        output: wl_output::WlOutput,
    ) {
        let key = self.surfaces.iter().find_map(|(k, v)| {
            if v.output == output {
                Some(k.clone())
            } else {
                None
            }
        });
        if let Some(k) = key {
            self.surfaces.remove(&k);
        }
        self.redraw = true;
    }
}

impl ShmHandler for NayuWayland {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl LayerShellHandler for NayuWayland {
    fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {
        // Ignore; if compositor closes background surface, nothing to do.
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        layer: &LayerSurface,
        configure: LayerSurfaceConfigure,
        _serial: u32,
    ) {
        let (w, h) = configure.new_size;

        let key = layer.wl_surface().clone();
        if let Some(entry) = self.surfaces.get_mut(&key) {
            entry.size = (w, h);
        }
        self.redraw = true;
    }
}

delegate_registry!(NayuWayland);
delegate_compositor!(NayuWayland);
delegate_output!(NayuWayland);
delegate_shm!(NayuWayland);
delegate_layer!(NayuWayland);
