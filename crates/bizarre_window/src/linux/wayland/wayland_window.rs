use std::{env::temp_dir, os::fd::AsFd, thread::sleep, time::Duration};

use wayland_client::{
    delegate_noop,
    protocol::{wl_buffer::WlBuffer, wl_registry, wl_shm, wl_shm_pool::WlShmPool},
    Dispatch, QueueHandle,
};
use wayland_protocols::xdg::shell::client::{
    xdg_surface::{self, XdgSurface},
    xdg_toplevel::XdgToplevel,
    xdg_wm_base,
};

use crate::{WindowCreateInfo, WindowMode, WindowTrait};

use super::wayland_context::{wayland_connection, wayland_context, WaylandContext};

#[derive(Debug)]
pub struct WaylandWindowState {
    pub(crate) xdg_surface: XdgSurface,
    pub(crate) toplevel: XdgToplevel,
    pub(crate) buffer: WlBuffer,
    pub(crate) pool: WlShmPool,

    pub(crate) should_close: bool,
}

#[derive(Debug)]
pub struct WaylandWindow {
    pub(crate) event_queue: wayland_client::EventQueue<WaylandWindowState>,
    pub(crate) state: WaylandWindowState,
}

impl Dispatch<XdgSurface, ()> for WaylandWindowState {
    fn event(
        state: &mut Self,
        proxy: &XdgSurface,
        event: <XdgSurface as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &wayland_client::Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        if let xdg_surface::Event::Configure { serial, .. } = event {
            proxy.ack_configure(serial);
            let wl_surface = wayland_context().wl_surface.as_ref().unwrap();
            wl_surface.attach(Some(&state.buffer), 0, 0);
            wl_surface.commit();
        }
    }
}

delegate_noop!(WaylandWindowState: ignore XdgToplevel);
delegate_noop!(WaylandWindowState: ignore WlShmPool);
delegate_noop!(WaylandWindowState: ignore WlBuffer);

impl WaylandWindow {
    pub(crate) fn init(create_info: &WindowCreateInfo) -> anyhow::Result<Self> {
        let event_queue = Self::init_event_queue();
        let qh = event_queue.handle();

        let state = Self::init_context(&qh, create_info.size.x as i32, create_info.size.y as i32);

        let window = Self { event_queue, state };

        Ok(window)
    }

    fn init_event_queue() -> wayland_client::EventQueue<WaylandWindowState> {
        let conn = &wayland_connection().conn;
        let event_queue = conn.new_event_queue();
        event_queue
    }

    fn init_xdg_surface(qh: &QueueHandle<WaylandWindowState>) -> (XdgSurface, XdgToplevel) {
        let ctx = wayland_context();
        let wl_surface = ctx.wl_surface.as_ref().unwrap();
        let xdg_wm_base = ctx.xdg_wm_base.as_ref().unwrap();
        let xdg_surface = xdg_wm_base.get_xdg_surface(wl_surface, qh, ());
        let toplevel = xdg_surface.get_toplevel(qh, ());

        (xdg_surface, toplevel)
    }

    fn init_shm(
        qh: &QueueHandle<WaylandWindowState>,
        init_w: i32,
        init_h: i32,
    ) -> (WlShmPool, WlBuffer) {
        let ctx = wayland_context();

        let shm = ctx.wl_shm.as_ref().unwrap();
        let file = tempfile::tempfile().unwrap();
        let pool = shm.create_pool(file.as_fd(), init_w * init_h * 4, qh, ());
        let buffer = pool.create_buffer(
            0,
            init_w,
            init_h,
            init_w * 4,
            wl_shm::Format::Argb8888,
            qh,
            (),
        );

        (pool, buffer)
    }

    fn init_context(
        qh: &QueueHandle<WaylandWindowState>,
        init_w: i32,
        init_h: i32,
    ) -> WaylandWindowState {
        let ctx = wayland_context();
        let (xdg_surface, toplevel) = Self::init_xdg_surface(qh);
        let (pool, buffer) = Self::init_shm(qh, init_w, init_h);

        WaylandWindowState {
            buffer,
            pool,
            xdg_surface,
            toplevel,
        }
    }
}
