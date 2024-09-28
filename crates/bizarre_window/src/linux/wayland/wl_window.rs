use std::io::ErrorKind;

use nalgebra_glm::{IVec2, U8Vec2, UVec2};
use wayland_client::{
    backend::WaylandError,
    delegate_noop,
    protocol::{wl_buffer::WlBuffer, wl_shm::WlShm, wl_shm_pool::WlShmPool, wl_surface::WlSurface},
    Dispatch,
};
use wayland_protocols::xdg::{
    decoration::zv1::client::zxdg_toplevel_decoration_v1::ZxdgToplevelDecorationV1,
    shell::client::{xdg_surface::XdgSurface, xdg_toplevel::XdgToplevel},
};

use crate::{
    window_error::{WindowError, WindowResult},
    PlatformWindow, WindowHandle,
};

use super::wl_context::{WlWindowResources, WL_CONTEXT};

pub struct WlWindow {
    event_queue: wayland_client::EventQueue<WlWindowState>,
    state: WlWindowState,
}

pub(crate) struct WlWindowState {
    pub(crate) surface: WlSurface,
    pub(crate) xdg_surface: XdgSurface,
    pub(crate) xdg_toplevel: XdgToplevel,
    pub(crate) decorations: ZxdgToplevelDecorationV1,
    pub(crate) resources: WlWindowResources,
}

impl PlatformWindow for WlWindow {
    fn new(create_info: &crate::WindowCreateInfo) -> WindowResult<Self>
    where
        Self: Sized,
    {
        let wayland_context = WL_CONTEXT
            .read()
            .map_err(|err| WindowError::ContextUnreachable {
                reason: format!("Could not aquire read lock on Wayland context: {err}"),
            })?;

        let [width, height] = create_info.size.into();
        let (width, height) = (width as usize, height as usize);

        let (event_queue, state) = wayland_context.create_window_state(width, height);

        let window = WlWindow { state, event_queue };

        Ok(window)
    }

    fn size(&self) -> UVec2 {
        todo!()
    }

    fn position(&self) -> IVec2 {
        todo!()
    }

    fn update_size_and_position(&mut self) -> WindowResult<(UVec2, IVec2)> {
        todo!()
    }

    fn mode(&self) -> crate::WindowMode {
        todo!()
    }

    fn raw_handle(&self) -> u32 {
        todo!()
    }

    fn handle(&self) -> WindowHandle {
        WindowHandle::from_raw(1u32)
    }

    fn title(&self) -> &str {
        todo!()
    }

    fn status(&self) -> crate::WindowStatus {
        todo!()
    }

    fn set_size(&mut self, size: UVec2) -> WindowResult<()> {
        todo!()
    }

    fn set_position(&mut self, position: IVec2) -> WindowResult<()> {
        todo!()
    }

    fn set_mode(&mut self, mode: crate::WindowMode) -> WindowResult<()> {
        todo!()
    }

    fn set_title(&mut self, title: String) -> WindowResult<()> {
        todo!()
    }

    fn set_decorations(&mut self, decorations: bool) -> WindowResult<()> {
        todo!()
    }

    fn map(&mut self) -> WindowResult<()> {
        Ok(())
    }

    fn unmap(&mut self) -> WindowResult<()> {
        todo!()
    }

    fn minimize(&mut self) -> WindowResult<()> {
        todo!()
    }

    fn restore(&mut self) -> WindowResult<()> {
        todo!()
    }

    fn maximize(&mut self) -> WindowResult<()> {
        todo!()
    }

    fn unmaximize(&mut self) -> WindowResult<()> {
        todo!()
    }

    fn handle_events(&mut self, event_queue: &mut bizarre_event::EventQueue) -> WindowResult<()> {
        self.event_queue.flush().unwrap();

        self.event_queue.dispatch_pending(&mut self.state).unwrap();

        match self.event_queue.prepare_read() {
            None => {
                self.event_queue.dispatch_pending(&mut self.state).unwrap();
            }
            Some(guard) => match guard.read() {
                Ok(count) if count > 0 => println!("window: dispatched: {count} events"),
                Ok(_) => {}
                Err(WaylandError::Io(err)) => {
                    if let ErrorKind::WouldBlock = err.kind() {
                    } else {
                        panic!("{err:?}")
                    }
                }
                Err(err) => {
                    panic!("{err:?}")
                }
            },
        }

        Ok(())
    }

    fn close_requested(&self) -> bool {
        false
    }
}

impl Dispatch<XdgSurface, (), WlWindowState> for WlWindowState {
    fn event(
        state: &mut WlWindowState,
        proxy: &XdgSurface,
        event: <XdgSurface as wayland_client::Proxy>::Event,
        data: &(),
        conn: &wayland_client::Connection,
        qhandle: &wayland_client::QueueHandle<WlWindowState>,
    ) {
        match event {
            wayland_protocols::xdg::shell::client::xdg_surface::Event::Configure { serial } => {
                println!("XdgSurface: acknowledging configure: {serial}");
                state.xdg_surface.ack_configure(serial)
            }
            _ => (),
        }
    }
}

delegate_noop!(WlWindowState: ignore WlShm);
delegate_noop!(WlWindowState: ignore WlBuffer);
delegate_noop!(WlWindowState: ignore WlShmPool);
delegate_noop!(WlWindowState: ignore WlSurface);
delegate_noop!(WlWindowState: ignore XdgToplevel);
delegate_noop!(WlWindowState: ignore ZxdgToplevelDecorationV1);
