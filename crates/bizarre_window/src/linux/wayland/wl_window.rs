use std::{
    hash::{DefaultHasher, Hash, Hasher},
    io::ErrorKind,
};

use bizarre_log::{core_fatal, core_trace};
use nalgebra_glm::{IVec2, UVec2, Vec2};
use wayland_client::{
    backend::WaylandError,
    delegate_noop,
    protocol::{
        wl_buffer::WlBuffer,
        wl_keyboard::{self, WlKeyboard},
        wl_pointer::{self, WlPointer},
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
    state: WlWindowState,
    wl_event_queue: wayland_client::EventQueue<WlWindowState>,
}

pub struct WlWindowResources {
    pub(crate) shm: SharedMemory,
    pub(crate) pool: WlShmPool,
    pub(crate) buffer: WlBuffer,
    pub(crate) keyboard: WlKeyboard,
    pub(crate) mouse: WlPointer,
    pub(crate) surface: WlSurface,
    pub(crate) xdg_surface: XdgSurface,
    pub(crate) xdg_toplevel: XdgToplevel,
    pub(crate) decorations: ZxdgToplevelDecorationV1,
}

impl Drop for WlWindowResources {
    fn drop(&mut self) {
        self.keyboard.release();
        self.mouse.release();
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
            pointer_input_frame: Vec::new(),
            title: create_info.title.clone(),
            close_requested: false,
            mode: create_info.mode,
            resources,
            status: WindowStatus::empty(),
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
        UVec2::new(self.state.size.x as u32, self.state.size.y as u32)
    }

    fn position(&self) -> IVec2 {
        todo!()
    }

    fn mode(&self) -> WindowMode {
        self.state.mode
    }

    fn raw_handle(&self) -> u64 {
        self.handle().as_raw() as u64
    }

    fn raw_window_ptr(&self) -> *const () {
        self.state.resources.surface.id().as_ptr() as _
    }

    fn handle(&self) -> WindowHandle {
        self.state.handle
    }

    fn title(&self) -> &str {
        &self.state.title
    }

    fn status(&self) -> crate::WindowStatus {
        self.state.status
    }

    fn set_size(&mut self, size: UVec2) -> WindowResult<()> {
        let [width, height] = [size.x as i32, size.y as i32];

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
        self.state.resources.xdg_toplevel.set_minimized();
        self.state.status |= WindowStatus::MINIMIZED;
        Ok(())
    }

    fn maximize(&mut self) -> WindowResult<()> {
        self.state.resources.xdg_toplevel.set_maximized();
        self.state.status |= WindowStatus::MAXIMIZED;
        Ok(())
    }

    fn unmaximize(&mut self) -> WindowResult<()> {
        self.state.resources.xdg_toplevel.unset_maximized();
        self.state.status.remove(WindowStatus::MAXIMIZED);
        Ok(())
    }

    fn toggle_maximize(&mut self) -> WindowResult<()> {
        if self.state.status.intersects(WindowStatus::MAXIMIZED) {
            self.state.resources.xdg_toplevel.unset_maximized();
            self.state.status.remove(WindowStatus::MAXIMIZED);
            Ok(())
        } else {
            self.state.resources.xdg_toplevel.set_maximized();
            self.state.status |= WindowStatus::MAXIMIZED;
            Ok(())
        }
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
                state.internal_event_queue.push(WindowEvent::Resize {
                    handle: state.handle,
                    size: UVec2::new(width as u32, height as u32),
                })
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
            _ => (),
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
                if !window_state.status.intersects(WindowStatus::KEYBOARD_FOCUS) {
                    return;
                }

                let keycode = (key + 8) as usize;
                let handle = window_state.handle;

                let event = match state {
                    WEnum::Value(val) => match val {
                        wl_keyboard::KeyState::Pressed => WindowEvent::KeyPress {
                            handle,
                            keycode: keycode as u8,
                        },
                        wl_keyboard::KeyState::Released => WindowEvent::KeyRelease {
                            handle,
                            keycode: keycode as u8,
                        },

                        _ => return,
                    },
                    _ => return,
                };

                window_state.internal_event_queue.push(event);
            }
            wl_keyboard::Event::Enter { surface, .. } => {
                if surface == window_state.resources.surface {
                    window_state.status |= WindowStatus::KEYBOARD_FOCUS;
                    window_state
                        .internal_event_queue
                        .push(WindowEvent::GainedKeyboardFocus(window_state.handle))
                }
            }
            wl_keyboard::Event::Leave { surface, .. } => {
                if surface == window_state.resources.surface {
                    window_state.status.remove(WindowStatus::KEYBOARD_FOCUS);
                    window_state
                        .internal_event_queue
                        .push(WindowEvent::LostKeyboardFocus(window_state.handle))
                }
            }
            _ => {}
        }
    }
}

impl Dispatch<WlPointer, ()> for WlWindowState {
    fn event(
        state: &mut Self,
        proxy: &WlPointer,
        event: <WlPointer as Proxy>::Event,
        data: &(),
        conn: &Connection,
        qhandle: &QueueHandle<Self>,
    ) {
        match event {
            wl_pointer::Event::Enter { surface, .. } => {
                if surface == state.resources.surface {
                    state.status.insert(WindowStatus::MOUSE_FOCUS);
                }
            }
            wl_pointer::Event::Leave { surface, .. } => {
                if surface == state.resources.surface {
                    state.status.remove(WindowStatus::MOUSE_FOCUS);
                }
            }
            wl_pointer::Event::Motion {
                surface_x,
                surface_y,
                ..
            } => {
                if state.status.intersects(WindowStatus::MOUSE_FOCUS) {
                    state.internal_event_queue.push(WindowEvent::PointerMove {
                        handle: state.handle,
                        position: Vec2::new(surface_x as f32, surface_y as f32),
                    });
                }
            }
            wl_pointer::Event::Button {
                button,
                state: button_state,
                ..
            } => {
                if state.status.intersects(WindowStatus::MOUSE_FOCUS) {
                    if let WEnum::Value(val) = button_state {
                        let event = match val {
                            wl_pointer::ButtonState::Pressed => WindowEvent::ButtonPress {
                                handle: state.handle,
                                button: (button - 272) as u8,
                            },
                            wl_pointer::ButtonState::Released => WindowEvent::ButtonRelease {
                                handle: state.handle,
                                button: (button - 272) as u8,
                            },
                            _ => {
                                core_fatal!("Wayland returned unnown button state: {val:?}");
                                return;
                            }
                        };

                        state.internal_event_queue.push(event)
                    }
                }
            }
            wl_pointer::Event::Axis { time, axis, value } => {
                state.pointer_input_frame.push(event);
            }
            wl_pointer::Event::AxisSource { axis_source } => {
                state.pointer_input_frame.push(event);
            }
            wl_pointer::Event::Frame => {
                state.handle_pointer_input_frame();
            }
            _ => (),
        }
    }
}

delegate_noop!(WlWindowState: ignore WlShm);
delegate_noop!(WlWindowState: ignore WlBuffer);
delegate_noop!(WlWindowState: ignore WlShmPool);
delegate_noop!(WlWindowState: ignore WlSurface);
