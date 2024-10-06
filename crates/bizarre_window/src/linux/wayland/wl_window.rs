use std::{
    hash::{DefaultHasher, Hash, Hasher},
    io::ErrorKind,
};

use bizarre_event::EventQueue;
use nalgebra_glm::{IVec2, U8Vec2, UVec2};
use wayland_client::{
    backend::WaylandError,
    delegate_noop,
    protocol::{
        wl_buffer::WlBuffer,
        wl_keyboard::{self, WlKeyboard},
        wl_shm::WlShm,
        wl_shm_pool::WlShmPool,
        wl_surface::WlSurface,
    },
    Connection, Dispatch, Proxy, QueueHandle, WEnum,
};
use wayland_protocols::xdg::{
    decoration::zv1::client::zxdg_toplevel_decoration_v1::{self, ZxdgToplevelDecorationV1},
    shell::client::{
        xdg_surface::{self, XdgSurface},
        xdg_toplevel::{self, XdgToplevel},
    },
};

use crate::{
    window_error::{WindowError, WindowResult},
    window_events::WindowEvent,
    PlatformWindow, WindowHandle, WindowMode, WindowStatus,
};

use super::{shared_memory::SharedMemory, wl_context::WL_CONTEXT, wl_window_state::WlWindowState};

pub struct WlWindow {
    wl_event_queue: wayland_client::EventQueue<WlWindowState>,
    state: WlWindowState,
}

pub struct WlWindowResources {
    pub(crate) shm: SharedMemory,
    pub(crate) pool: WlShmPool,
    pub(crate) buffer: WlBuffer,
    pub(crate) keyboard: WlKeyboard,
    pub(crate) surface: WlSurface,
    pub(crate) xdg_surface: XdgSurface,
    pub(crate) xdg_toplevel: XdgToplevel,
    pub(crate) decorations: ZxdgToplevelDecorationV1,
}

impl Drop for WlWindowResources {
    fn drop(&mut self) {
        self.decorations.destroy();
        self.xdg_toplevel.destroy();
        self.xdg_surface.destroy();
        self.buffer.destroy();
        self.pool.destroy();
        self.surface.destroy();
    }
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

        let (event_queue, resources) = wayland_context.create_window_resources(width, height);

        let handle = {
            let mut hasher = DefaultHasher::new();
            resources.surface.hash(&mut hasher);
            let hash = hasher.finish();
            let handle = WindowHandle::from_raw(hash);
            println!("Created window handle: {handle:?}, {:?}", resources.surface);
            handle
        };

        let size = {
            let [x, y] = create_info.size.into();
            [x as i32, y as i32].into()
        };

        let state = WlWindowState {
            handle,
            internal_event_queue: Default::default(),
            size,
            position: create_info.position.clone(),
            title: create_info.title.clone(),
            close_requested: false,
            mode: create_info.mode,
            resources,
        };

        let mut window = WlWindow {
            state,
            wl_event_queue: event_queue,
        };

        window.set_position(create_info.position)?;
        window.set_size(create_info.size)?;
        window.set_mode(create_info.mode)?;
        window.set_title(create_info.title.clone())?;

        window.state.resources.surface.commit();

        window.wl_event_queue.flush();

        Ok(window)
    }

    fn size(&self) -> UVec2 {
        todo!()
    }

    fn position(&self) -> IVec2 {
        todo!()
    }

    fn mode(&self) -> WindowMode {
        todo!()
    }

    fn raw_handle(&self) -> u64 {
        todo!()
    }

    fn handle(&self) -> WindowHandle {
        self.state.handle
    }

    fn title(&self) -> &str {
        todo!()
    }

    fn status(&self) -> crate::WindowStatus {
        todo!()
    }

    fn set_size(&mut self, size: UVec2) -> WindowResult<()> {
        let [width, height] = [size.x as i32, size.y as i32];

        println!("setting window size: {:?}", [width, height]);

        self.state
            .resources
            .xdg_surface
            .set_window_geometry(0, 0, width, height);

        self.wl_event_queue.flush().unwrap();

        Ok(())
    }

    fn set_position(&mut self, _position: IVec2) -> WindowResult<()> {
        Ok(())
    }

    fn set_mode(&mut self, mode: WindowMode) -> WindowResult<()> {
        match mode {
            WindowMode::Fullscreen => self.state.resources.xdg_toplevel.set_fullscreen(None),
            WindowMode::Windowed => {
                self.state.resources.xdg_toplevel.unset_fullscreen();
                self.state
                    .resources
                    .decorations
                    .set_mode(zxdg_toplevel_decoration_v1::Mode::ServerSide);
            }
            WindowMode::WindowedBorderless => {
                self.state.resources.xdg_toplevel.unset_fullscreen();
                self.state
                    .resources
                    .decorations
                    .set_mode(zxdg_toplevel_decoration_v1::Mode::ClientSide);
            }
        }

        self.wl_event_queue.flush();

        Ok(())
    }

    fn set_title(&mut self, title: String) -> WindowResult<()> {
        self.state.resources.xdg_toplevel.set_title(title);

        self.wl_event_queue.flush();

        Ok(())
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

    fn process_events(&mut self, event_queue: &mut bizarre_event::EventQueue) -> WindowResult<()> {
        self.state
            .resources
            .surface
            .damage(0, 0, i32::MAX, i32::MAX);

        self.wl_event_queue.flush().unwrap();

        self.wl_event_queue
            .dispatch_pending(&mut self.state)
            .unwrap();

        match self.wl_event_queue.prepare_read() {
            None => {
                self.wl_event_queue
                    .dispatch_pending(&mut self.state)
                    .unwrap();
            }
            Some(guard) => match guard.read() {
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

        self.state
            .internal_event_queue
            .drain(..)
            .for_each(|ev| event_queue.push_event(ev));

        Ok(())
    }

    fn close_requested(&self) -> bool {
        self.state.close_requested
    }

    fn close(&mut self) -> WindowResult<()> {
        todo!()
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
        println!("xdg_surface: {event:#?}");
        match event {
            xdg_surface::Event::Configure { serial } => {
                proxy.ack_configure(serial);
                state.resources.surface.commit();
            }
            _ => (),
        }
    }
}

impl Dispatch<XdgToplevel, ()> for WlWindowState {
    fn event(
        state: &mut Self,
        proxy: &XdgToplevel,
        event: <XdgToplevel as Proxy>::Event,
        data: &(),
        conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        println!("xdg_toplevel: {event:#?}");
        match event {
            xdg_toplevel::Event::Close => {
                state
                    .internal_event_queue
                    .push(WindowEvent::Close(state.handle));
                state.close_requested = true;
            }
            xdg_toplevel::Event::Configure { width, height, .. } => {
                if width == 0 || height == 0 {
                    return;
                }

                state.resize(qhandle, width, height);
            }
            _ => {}
        }
    }
}

impl Dispatch<ZxdgToplevelDecorationV1, ()> for WlWindowState {
    fn event(
        _: &mut Self,
        _: &ZxdgToplevelDecorationV1,
        event: <ZxdgToplevelDecorationV1 as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            zxdg_toplevel_decoration_v1::Event::Configure { mode } => {
                println!("xdg_toplevel_decoration: set mode `{mode:?}`")
            }
            _ => todo!(),
        }
    }
}

impl Dispatch<WlKeyboard, ()> for WlWindowState {
    fn event(
        window_state: &mut Self,
        proxy: &WlKeyboard,
        event: <WlKeyboard as Proxy>::Event,
        data: &(),
        conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        match event {
            wl_keyboard::Event::Key {
                serial,
                time,
                key,
                state,
            } => {
                let keycode = (key + 8) as usize;
                let handle = window_state.handle;

                let event = match state {
                    WEnum::Value(val) => match val {
                        wl_keyboard::KeyState::Pressed => WindowEvent::KeyPress { handle, keycode },
                        wl_keyboard::KeyState::Released => {
                            WindowEvent::KeyRelease { handle, keycode }
                        }

                        _ => return,
                    },
                    _ => return,
                };

                window_state.internal_event_queue.push(event);
            }
            _ => {}
        }
    }
}

delegate_noop!(WlWindowState: ignore WlShm);
delegate_noop!(WlWindowState: ignore WlBuffer);
delegate_noop!(WlWindowState: ignore WlShmPool);
delegate_noop!(WlWindowState: ignore WlSurface);
