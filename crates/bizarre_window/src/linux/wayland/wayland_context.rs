use std::sync::{LazyLock, RwLock};

use wayland_client::{
    globals::{registry_queue_init, GlobalList},
    Connection, EventQueue,
};

use super::wayland_window::WaylandWindow;

pub(crate) struct WaylandContext {
    pub(crate) event_queue: EventQueue<WaylandWindow>,
    pub(crate) globals: GlobalList,
}

static WAYLAND_CONNECTION: LazyLock<Connection> =
    LazyLock::new(|| Connection::connect_to_env().unwrap());
