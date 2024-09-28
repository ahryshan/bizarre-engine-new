use core::sync::{self, atomic};
use std::{
    borrow::Borrow,
    os::fd::{AsFd, OwnedFd},
    ptr::{self, slice_from_raw_parts_mut},
    slice,
    sync::{atomic::AtomicUsize, LazyLock, RwLock},
};

use lazy_static::lazy_static;
use rustix::{
    fs::ftruncate,
    mm::{munmap, MapFlags, ProtFlags},
    shm::OFlags,
};
use wayland_client::{
    delegate_noop,
    globals::{registry_queue_init, GlobalListContents},
    protocol::{
        wl_buffer::WlBuffer,
        wl_compositor::{self, WlCompositor},
        wl_display::WlDisplay,
        wl_registry::{self, WlRegistry},
        wl_shm::{self, WlShm},
        wl_shm_pool::WlShmPool,
        wl_surface::WlSurface,
    },
    Connection, Dispatch, Proxy, QueueHandle,
};
use wayland_protocols::xdg::{
    activation::v1::client::xdg_activation_token_v1::XdgActivationTokenV1,
    decoration::zv1::client::{
        zxdg_decoration_manager_v1::{self, ZxdgDecorationManagerV1},
        zxdg_toplevel_decoration_v1::{self, ZxdgToplevelDecorationV1},
    },
    shell::client::{
        xdg_surface::XdgSurface,
        xdg_toplevel::XdgToplevel,
        xdg_wm_base::{self, XdgWmBase},
    },
};

use crate::window_error::WindowResult;

use super::wl_window::WlWindowState;

pub(crate) static WL_CONTEXT: LazyLock<RwLock<WaylandContext>> =
    LazyLock::new(|| RwLock::new(WaylandContext::new()));

pub struct WaylandContext {
    pub(crate) conn: Connection,
    pub(crate) state: WaylandState,
    pub(crate) event_queue: wayland_client::EventQueue<WaylandState>,
}

pub struct WaylandState {
    pub(crate) compositor: WlCompositor,
    pub(crate) shm: WlShm,
    pub(crate) xdg: XdgWmBase,
    pub(crate) xdg_decoration_manager: ZxdgDecorationManagerV1,
}

pub struct WlWindowResources {
    pub(crate) shm_fd: OwnedFd,
    pub(crate) pool: WlShmPool,
    pub(crate) buffer: WlBuffer,
}

impl WaylandContext {
    fn new() -> Self {
        let conn = match Connection::connect_to_env() {
            Ok(conn) => {
                println!("Successfully connected to Wayland server!");
                conn
            }
            Err(err) => panic!("Could not create a Wayland connection: {err}"),
        };

        let (globals, mut event_queue) = registry_queue_init(&conn)
            .unwrap_or_else(|err| panic!("Could not initialize Wayland globals: {err}"));

        let qh = event_queue.handle();
        let state = WaylandState {
            compositor: globals
                .bind(&qh, 0..=WlCompositor::interface().version, ())
                .unwrap_or_else(|err| panic!("Could not obtain wl_compositor: {err}")),
            shm: globals
                .bind(&qh, 0..=WlShm::interface().version, ())
                .unwrap_or_else(|err| panic!("Could not obtain wl_compositor: {err}")),
            xdg: globals
                .bind(&qh, 0..=XdgWmBase::interface().version, ())
                .unwrap_or_else(|err| panic!("Could not obtain xdg_wm_base: {err}")),
            xdg_decoration_manager: globals
                .bind(&qh, 0..=ZxdgDecorationManagerV1::interface().version, ())
                .unwrap_or_else(|err| panic!("Could not obtain xdg_wm_base: {err}")),
        };

        conn.roundtrip();

        Self {
            conn,
            event_queue,
            state,
        }
    }

    pub fn create_window_state(
        &self,
        width: usize,
        height: usize,
    ) -> (wayland_client::EventQueue<WlWindowState>, WlWindowState) {
        let stride = width * 4;
        let pool_size = height * 2 * stride;

        let shm_fd = Self::open_shm(pool_size);

        let (ptr, pool_data) = unsafe {
            let ptr = rustix::mm::mmap(
                ptr::null_mut(),
                pool_size,
                ProtFlags::READ | ProtFlags::WRITE,
                MapFlags::SHARED,
                shm_fd.as_fd(),
                0,
            )
            .unwrap()
            .cast::<u32>();

            (ptr, slice::from_raw_parts_mut(ptr, pool_size))
        };

        let event_queue = self.conn.new_event_queue();
        let qh = event_queue.handle();

        let pool = self
            .state
            .shm
            .create_pool(shm_fd.as_fd(), pool_size as i32, &qh, ());

        let buffer = pool.create_buffer(
            0,
            width as i32,
            height as i32,
            stride as i32,
            wl_shm::Format::Xrgb8888,
            &qh,
            (),
        );

        for y in 0..height {
            for x in 0..width {
                if (x + y / 32 * 32) % 64 < 32 {
                    pool_data[y * width + x] = 0xFF666666;
                } else {
                    pool_data[y * width + x] = 0xFFEEEEEE;
                }
            }
        }

        unsafe {
            munmap(ptr.cast(), pool_size);
        }

        let surface = self.state.compositor.create_surface(&qh, ());
        let xdg_surface = self.state.xdg.get_xdg_surface(&surface, &qh, ());
        let xdg_toplevel = xdg_surface.get_toplevel(&qh, ());

        xdg_toplevel.set_title("Hello wayland".into());

        let decorations =
            self.state
                .xdg_decoration_manager
                .get_toplevel_decoration(&xdg_toplevel, &qh, ());

        decorations.set_mode(zxdg_toplevel_decoration_v1::Mode::ServerSide);

        surface.attach(Some(&buffer), 0, 0);
        surface.damage(0, 0, i32::MAX, i32::MAX);
        surface.commit();

        event_queue.flush();

        let resources = WlWindowResources {
            buffer,
            pool,
            shm_fd,
        };

        let state = WlWindowState {
            surface,
            xdg_surface,
            xdg_toplevel,
            decorations,
            resources,
        };

        (event_queue, state)
    }

    pub fn drain_system_events(&mut self, eq: &mut bizarre_event::EventQueue) -> WindowResult<()> {
        self.event_queue.flush();

        self.event_queue.dispatch_pending(&mut self.state).unwrap();

        Ok(())
    }

    fn open_shm(size: usize) -> OwnedFd {
        use rustix::shm::Mode;

        static NEXT_FILE_NUMBER: AtomicUsize = AtomicUsize::new(1);

        let fd = loop {
            let file_number = NEXT_FILE_NUMBER.fetch_add(1, atomic::Ordering::AcqRel);

            let filename = format!("/wl_shm-{file_number:0>4}");

            let result = rustix::shm::open(
                filename,
                OFlags::RDWR | OFlags::CREATE | OFlags::EXCL,
                Mode::from_bits_retain(600),
            );

            if let Ok(fd) = result {
                break fd;
            }
        };

        ftruncate(fd.as_fd(), size as u64).unwrap();

        fd
    }
}

impl Dispatch<WlRegistry, GlobalListContents> for WaylandState {
    fn event(
        _state: &mut Self,
        _proxy: &WlRegistry,
        _event: <WlRegistry as Proxy>::Event,
        _data: &GlobalListContents,
        _conn: &Connection,
        _qhandle: &QueueHandle<Self>,
    ) {
        match _event {
            wl_registry::Event::Global {
                name,
                interface,
                version,
            } => println!("[{name}] {interface} (v.{version})"),
            wl_registry::Event::GlobalRemove { name } => {}
            _ => {}
        }
    }
}

impl Dispatch<XdgWmBase, ()> for WaylandState {
    fn event(
        state: &mut Self,
        proxy: &XdgWmBase,
        event: <XdgWmBase as Proxy>::Event,
        data: &(),
        conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        println!("xdg_wm_base dispatch");
        match event {
            xdg_wm_base::Event::Ping { serial } => {
                println!("Got ping: pong");
                state.xdg.pong(serial);
            }
            _ => todo!(),
        }
    }
}

delegate_noop!(WaylandState: ignore WlCompositor);
delegate_noop!(WaylandState: ignore ZxdgDecorationManagerV1);
delegate_noop!(WaylandState: ignore WlShm);
