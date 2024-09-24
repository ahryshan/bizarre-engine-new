use std::sync::{LazyLock, RwLock};

use lazy_static::lazy_static;
use wayland_client::{
    protocol::{
        wl_display::WlDisplay,
        wl_registry::{self, WlRegistry},
    },
    Connection, Dispatch, Proxy, QueueHandle,
};

pub(crate) static WL_CONTEXT: LazyLock<RwLock<WaylandContext>> =
    LazyLock::new(|| RwLock::new(WaylandContext::new()));

pub struct WaylandContext {
    pub(crate) conn: Connection,
    pub(crate) display: WlDisplay,
    pub(crate) state: WaylandState,
    pub(crate) event_queue: wayland_client::EventQueue<WaylandState>,
}

pub struct WaylandState {}

impl WaylandContext {
    fn new() -> Self {
        let conn = match Connection::connect_to_env() {
            Ok(conn) => {
                println!("Successfully connected to Wayland server!");
                conn
            }
            Err(err) => panic!("Could not create a Wayland connection: {err}"),
        };

        let display = conn.display();
        let mut event_queue = conn.new_event_queue::<WaylandState>();
        let qh = event_queue.handle();

        let registry = display.get_registry(&qh, ());

        let mut state = WaylandState {};

        event_queue.roundtrip(&mut state);

        Self {
            conn,
            display,
            event_queue,
            state,
        }
    }
}

impl Dispatch<WlRegistry, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _proxy: &WlRegistry,
        event: <WlRegistry as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            println!("[{name}] {interface} (v{version})");
        }
    }
}
