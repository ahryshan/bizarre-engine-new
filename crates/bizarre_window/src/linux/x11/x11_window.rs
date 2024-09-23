use std::fmt::Debug;

use bizarre_event::{EventQueue, EventReader};
use nalgebra_glm::{IVec2, UVec2};
use xcb::{x, Xid};

use crate::{
    platform_window::PlatformWindow,
    window::{WindowHandle, WindowMode},
    window_error::{WindowError, WindowResult},
    window_events::WindowEvent,
    WindowCreateInfo,
};

use super::{
    connection::{get_x11_context, get_x11_context_mut, X11Context},
    motif_hints::MotifHints,
    x11_event::X11WindowEvent,
};

xcb::atoms_struct! {
    #[derive(Clone, Copy, Debug)]
    pub(crate) struct Atoms {
        pub wm_protocols => b"WM_PROTOCOLS",
        pub delete_window => b"WM_DELETE_WINDOW",
        pub wm_state => b"_NET_WM_STATE",
        pub wm_state_maxv => b"_NET_WM_STATE_MAXIMIZED_VERT",
        pub wm_state_maxh => b"_NET_WM_STATE_MAXIMIZED_HORZ",
        pub motif_wm_hints => b"_MOTIF_WM_HINTS",
        pub wm_state_fullscreen => b"_NET_WM_STATE_FULLSCREEN",
        pub wm_state_hidden => b"_NET_WM_STATE_HIDDEN",
        pub wm_state_add => b"_NET_WM_STATE_ADD",
        pub wm_state_remove => b"_NET_WM_STATE_REMOVE",
        pub wm_allowed_actions => b"_NET_WM_ALLOWED_ACTIONS",
    }
}

pub struct X11Window {
    pub(crate) size: UVec2,
    pub(crate) position: IVec2,
    pub(crate) mode: WindowMode,
    pub(crate) decorations: bool,
    pub(crate) id: x::Window,
    pub(crate) title: String,
    pub(crate) atoms: Atoms,
    pub(crate) minimized: bool,
    pub(crate) maximized: bool,
    pub(crate) mapped: bool,
    pub(crate) close_requested: bool,
    pub(crate) event_reader: Option<EventReader>,
    pub(crate) __no_wm_state_add_remove: bool,
}

impl X11Window {
    fn get_full_property<P>(&self, prop: x::Atom, prop_type: x::Atom) -> WindowResult<Vec<P>>
    where
        P: x::PropEl + Debug + Copy + Clone,
    {
        const INITIAL_READ_LEN: u32 = 32;

        let mut props = vec![];
        let conn = &get_x11_context().conn;
        let reply = conn.wait_for_reply(conn.send_request(&x::GetProperty {
            delete: false,
            long_offset: 0,
            long_length: INITIAL_READ_LEN,
            r#type: prop_type,
            property: prop,
            window: self.id,
        }))?;

        props.extend_from_slice(reply.value());
        if reply.bytes_after() != 0 {
            let offset = reply.length();
            let length = reply.bytes_after() / 4;

            let reply = conn.wait_for_reply(conn.send_request(&x::GetProperty {
                delete: false,
                long_offset: offset,
                long_length: length,
                r#type: prop_type,
                property: prop,
                window: self.id,
            }))?;

            props.extend_from_slice(reply.value());
        }

        Ok(props)
    }

    fn filter_props<P>(props: &[P], blocklist: &[P]) -> Vec<P>
    where
        P: x::PropEl + Copy + Clone + PartialEq,
    {
        props
            .iter()
            .cloned()
            .filter(|p| !blocklist.contains(p))
            .collect()
    }

    fn remove_wm_state(&self, to_remove: &[x::Atom]) -> WindowResult<()> {
        if self.__no_wm_state_add_remove {
            let props = self.get_full_property::<x::Atom>(self.atoms.wm_state, x::ATOM_ATOM)?;
            let filtered = Self::filter_props(&props, to_remove);
            let conn = &get_x11_context().conn;
            conn.check_request(conn.send_request_checked(&x::ChangeProperty {
                mode: x::PropMode::Replace,
                window: self.id,
                data: &filtered,
                property: self.atoms.wm_state,
                r#type: x::ATOM_ATOM,
            }))?;
            Ok(())
        } else {
            let X11Context {
                conn, screen_num, ..
            } = get_x11_context();
            to_remove
                .chunks(2)
                .map(|atoms| {
                    let (atom1, atom2) = if atoms.len() == 2 {
                        (atoms[0], atoms[1])
                    } else {
                        (atoms[0], x::ATOM_NONE)
                    };

                    let data = x::ClientMessageData::Data32([
                        self.atoms.wm_state_remove.resource_id(),
                        atom1.resource_id(),
                        atom2.resource_id(),
                        0,
                        0,
                    ]);
                    let event = x::ClientMessageEvent::new(self.id, self.atoms.wm_state, data);
                    let screen = conn.get_setup().roots().nth(*screen_num as usize).unwrap();
                    conn.send_request_checked(&x::SendEvent {
                        propagate: false,
                        destination: x::SendEventDest::Window(screen.root()),
                        event: &event,
                        event_mask: x::EventMask::SUBSTRUCTURE_NOTIFY,
                    })
                })
                .map(|c| conn.check_request(c))
                .collect::<Result<_, _>>()?;
            Ok(())
        }
    }

    fn add_wm_state(&self, to_add: &[x::Atom]) -> WindowResult<()> {
        if self.__no_wm_state_add_remove {
            let conn = &get_x11_context().conn;
            conn.check_request(conn.send_request_checked(&x::ChangeProperty {
                mode: x::PropMode::Append,
                property: self.atoms.wm_state,
                r#type: x::ATOM_ATOM,
                window: self.id,
                data: to_add,
            }))?;
            Ok(())
        } else {
            let X11Context {
                conn, screen_num, ..
            } = get_x11_context();
            let screen_num = *screen_num;
            to_add
                .chunks(2)
                .map(|atoms| {
                    let (atom1, atom2) = if atoms.len() == 2 {
                        (atoms[0], atoms[1])
                    } else {
                        (atoms[0], x::ATOM_NONE)
                    };

                    let data = x::ClientMessageData::Data32([
                        self.atoms.wm_state_add.resource_id(),
                        atom1.resource_id(),
                        atom2.resource_id(),
                        1,
                        0,
                    ]);
                    let event = x::ClientMessageEvent::new(self.id, self.atoms.wm_state, data);
                    let screen = conn.get_setup().roots().nth(screen_num as usize).unwrap();
                    conn.send_request_checked(&x::SendEvent {
                        propagate: false,
                        destination: x::SendEventDest::Window(screen.root()),
                        event: &event,
                        event_mask: x::EventMask::SUBSTRUCTURE_NOTIFY,
                    })
                })
                .map(|c| conn.check_request(c))
                .collect::<Result<_, _>>()?;
            Ok(())
        }
    }
}

impl PlatformWindow for X11Window {
    fn new(create_info: &WindowCreateInfo) -> WindowResult<Self>
    where
        Self: Sized,
    {
        let X11Context {
            conn, screen_num, ..
        } = get_x11_context();
        let setup = conn.get_setup();
        let screen = setup.roots().nth(*screen_num as usize).unwrap();

        let window: x::Window = conn.generate_id();

        let create_cookie = conn.send_request_checked(&x::CreateWindow {
            depth: x::COPY_FROM_PARENT as u8,
            parent: screen.root(),
            wid: window,
            x: create_info.position.x as i16,
            y: create_info.position.y as i16,
            width: create_info.size.x as u16,
            height: create_info.size.y as u16,
            border_width: 0,
            class: x::WindowClass::InputOutput,
            visual: screen.root_visual(),
            value_list: &[
                x::Cw::BackPixel(screen.white_pixel()),
                x::Cw::EventMask(
                    x::EventMask::EXPOSURE
                        | x::EventMask::KEY_PRESS
                        | x::EventMask::KEY_RELEASE
                        | x::EventMask::BUTTON_PRESS
                        | x::EventMask::BUTTON_RELEASE
                        | x::EventMask::POINTER_MOTION
                        | x::EventMask::STRUCTURE_NOTIFY
                        | x::EventMask::SUBSTRUCTURE_NOTIFY,
                ),
            ],
        });

        let rename_cookie = conn.send_request_checked(&x::ChangeProperty {
            mode: x::PropMode::Replace,
            window,
            property: x::ATOM_WM_NAME,
            r#type: x::ATOM_STRING,
            data: create_info.title.as_bytes(),
        });

        let atoms = Atoms::intern_all(&conn)?;

        let __no_wm_state_add_remove =
            atoms.wm_state_add == x::ATOM_NONE || atoms.wm_state_remove == x::ATOM_NONE;

        let window_delete_cookie = conn.send_request_checked(&x::ChangeProperty {
            mode: x::PropMode::Replace,
            window,
            property: atoms.wm_protocols,
            r#type: x::ATOM_ATOM,
            data: &[atoms.delete_window],
        });

        let window_geometry_cookie = conn.send_request(&x::GetGeometry {
            drawable: x::Drawable::Window(window),
        });

        let decorations_cookie = {
            let hints = if create_info.decorations {
                MotifHints::default_decorations()
            } else {
                MotifHints::no_decorations()
            };

            conn.send_request_checked(&x::ChangeProperty {
                mode: x::PropMode::Replace,
                window,
                r#type: atoms.motif_wm_hints,
                property: atoms.motif_wm_hints,
                data: &hints.to_prop_data(),
            })
        };

        let wm_state_cookie = {
            let mut props = vec![];
            if let WindowMode::Fullscreen = create_info.mode {
                props.push(atoms.wm_state_fullscreen);
            }

            if create_info.maximized {
                props.extend_from_slice(&[atoms.wm_state_maxv, atoms.wm_state_maxh]);
            }
            if create_info.minimized {
                props.push(atoms.wm_state_hidden);
            }

            conn.send_request_checked(&x::ChangeProperty {
                mode: x::PropMode::Replace,
                window,
                r#type: x::ATOM_ATOM,
                property: atoms.wm_state,
                data: &props,
            })
        };

        let wm_allowed_actions_cookie = {
            let values = create_info
                .allowed_actions
                .iter()
                .cloned()
                .map(x::Atom::from)
                .collect::<Vec<_>>();

            conn.send_request_checked(&x::ChangeProperty {
                mode: x::PropMode::Replace,
                window,
                property: atoms.wm_allowed_actions,
                r#type: x::ATOM_ATOM,
                data: &values,
            })
        };

        conn.check_request(create_cookie)
            .map_err(WindowError::from)?;
        conn.check_request(rename_cookie)
            .map_err(WindowError::from)?;
        conn.check_request(window_delete_cookie)
            .map_err(WindowError::from)?;
        conn.check_request(decorations_cookie)
            .map_err(WindowError::from)?;
        conn.check_request(wm_state_cookie)
            .map_err(WindowError::from)?;
        conn.check_request(wm_allowed_actions_cookie)
            .map_err(WindowError::from)?;

        let geometry = conn.wait_for_reply(window_geometry_cookie)?;

        let window = Self {
            size: UVec2::from([geometry.width() as u32, geometry.height() as u32]),
            position: IVec2::from([geometry.x() as i32, geometry.y() as i32]),
            mode: create_info.mode,
            decorations: create_info.decorations,
            mapped: false,
            id: window,
            title: create_info.title.clone(),
            maximized: create_info.maximized,
            minimized: create_info.minimized,
            close_requested: false,
            atoms,
            event_reader: None,
            __no_wm_state_add_remove,
        };

        Ok(window)
    }

    fn size(&self) -> UVec2 {
        self.size
    }

    fn update_size_and_position(&mut self) -> WindowResult<(UVec2, IVec2)> {
        let conn = &get_x11_context().conn;
        let geometry = conn.wait_for_reply(conn.send_request(&x::GetGeometry {
            drawable: x::Drawable::Window(self.id),
        }))?;

        let size = UVec2::from([geometry.width() as u32, geometry.height() as u32]);
        let position = IVec2::from([geometry.x() as i32, geometry.y() as i32]);

        let result = (size, position);
        (self.size, self.position) = result;

        Ok(result)
    }

    fn position(&self) -> IVec2 {
        self.position
    }

    fn mode(&self) -> crate::window::WindowMode {
        self.mode
    }

    fn set_size(&mut self, size: UVec2) -> WindowResult<()> {
        if self.size == size {
            return Ok(());
        }

        let conn = &get_x11_context().conn;
        let cookie = conn.send_request_checked(&x::ConfigureWindow {
            window: self.id,
            value_list: &[
                x::ConfigWindow::Width(size.x),
                x::ConfigWindow::Height(size.y),
            ],
        });

        conn.check_request(cookie)?;

        self.size = size;

        Ok(())
    }

    fn set_position(&mut self, position: IVec2) -> WindowResult<()> {
        if self.position == position {
            return Ok(());
        }

        let conn = &get_x11_context().conn;
        let cookie = conn.send_request_checked(&x::ConfigureWindow {
            window: self.id,
            value_list: &[
                x::ConfigWindow::X(position.x),
                x::ConfigWindow::Y(position.y),
            ],
        });

        conn.check_request(cookie)?;

        self.position = position;

        Ok(())
    }

    fn set_mode(&mut self, mode: crate::window::WindowMode) -> WindowResult<()> {
        let conn = &get_x11_context().conn;

        let was_mapped = self.mapped;

        if was_mapped {
            self.unmap()?;
        }

        match mode {
            WindowMode::Windowed => {
                let wm_state =
                    self.get_full_property::<x::Atom>(self.atoms.wm_state, x::ATOM_ATOM)?;
                let wm_state = Self::filter_props(&wm_state, &[self.atoms.wm_state_fullscreen]);
                let cookie = conn.send_request_checked(&x::ChangeProperty {
                    mode: x::PropMode::Replace,
                    window: self.id,
                    property: self.atoms.wm_state,
                    r#type: x::ATOM_ATOM,
                    data: &wm_state,
                });

                conn.check_request(cookie)?
            }
            WindowMode::Fullscreen => {
                let cookie = conn.send_request_checked(&x::ChangeProperty {
                    mode: x::PropMode::Prepend,
                    window: self.id,
                    data: &[self.atoms.wm_state_fullscreen],
                    r#type: x::ATOM_ATOM,
                    property: self.atoms.wm_state,
                });
                conn.check_request(cookie)?
            }
        }

        if was_mapped {
            self.map()?;
        }

        Ok(())
    }

    fn map(&mut self) -> WindowResult<()> {
        if self.mapped {
            return Ok(());
        }

        let conn = &get_x11_context().conn;

        conn.check_request(conn.send_request_checked(&x::MapWindow { window: self.id }))?;

        self.mapped = true;
        Ok(())
    }

    fn unmap(&mut self) -> WindowResult<()> {
        if !self.mapped {
            return Ok(());
        }

        let conn = &get_x11_context().conn;

        conn.check_request(conn.send_request_checked(&x::UnmapWindow { window: self.id }))?;

        self.mapped = false;
        Ok(())
    }

    fn raw_handle(&self) -> u32 {
        self.id.resource_id()
    }

    fn handle(&self) -> WindowHandle {
        WindowHandle::from_raw(self.id.resource_id())
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn set_title(&mut self, title: String) -> WindowResult<()> {
        if self.title == title {
            return Ok(());
        }

        let conn = &get_x11_context().conn;

        let cookie = conn.send_request_checked(&x::ChangeProperty {
            mode: x::PropMode::Replace,
            window: self.id,
            r#type: x::ATOM_STRING,
            data: title.as_bytes(),
            property: x::ATOM_WM_NAME,
        });

        conn.check_request(cookie).map_err(WindowError::from)?;

        Ok(())
    }

    fn minimize(&mut self) -> WindowResult<()> {
        let conn = &get_x11_context().conn;
        let cookie = conn.send_request_checked(&x::ChangeProperty {
            property: self.atoms.wm_state,
            r#type: x::ATOM_ATOM,
            data: &[self.atoms.wm_state_hidden],
            window: self.id,
            mode: x::PropMode::Prepend,
        });
        conn.check_request(cookie)?;
        Ok(())
    }

    fn status(&self) -> crate::window::WindowStatus {
        crate::window::WindowStatus {
            minimized: self.minimized,
            maximized: self.maximized,
            mapped: self.mapped,
        }
    }

    fn restore(&mut self) -> WindowResult<()> {
        let wm_state = self.get_full_property(self.atoms.wm_state, x::ATOM_ATOM)?;
        let wm_state = Self::filter_props(&wm_state, &[self.atoms.wm_state_hidden]);
        let conn = &get_x11_context().conn;
        let cookie = conn.send_request_checked(&x::ChangeProperty {
            mode: x::PropMode::Replace,
            window: self.id,
            data: &wm_state,
            r#type: x::ATOM_ATOM,
            property: self.atoms.wm_state,
        });
        conn.check_request(cookie)?;
        Ok(())
    }

    fn maximize(&mut self) -> WindowResult<()> {
        self.add_wm_state(&[self.atoms.wm_state_maxv, self.atoms.wm_state_maxh])?;
        let wm_state = self.get_full_property::<x::Atom>(self.atoms.wm_state, x::ATOM_ATOM)?;
        println!("wm_state: {wm_state:?}");

        Ok(())
    }

    fn unmaximize(&mut self) -> WindowResult<()> {
        self.remove_wm_state(&[self.atoms.wm_state_maxv, self.atoms.wm_state_maxh])?;
        let wm_state = self.get_full_property::<x::Atom>(self.atoms.wm_state, x::ATOM_ATOM)?;
        println!("wm_state: {wm_state:?}");

        Ok(())
    }

    fn set_decorations(&mut self, decorations: bool) -> WindowResult<()> {
        let hints = if decorations {
            MotifHints::default_decorations()
        } else {
            MotifHints::no_decorations()
        };

        let conn = &get_x11_context().conn;
        conn.check_request(conn.send_request_checked(&x::ChangeProperty {
            mode: x::PropMode::Replace,
            window: self.id,
            r#type: self.atoms.motif_wm_hints,
            property: self.atoms.motif_wm_hints,
            data: &hints.to_prop_data(),
        }))?;

        self.decorations = decorations;

        Ok(())
    }

    fn close_requested(&self) -> bool {
        self.close_requested
    }

    fn handle_events(&mut self, eq: &mut EventQueue) -> WindowResult<()> {
        let reader = self.event_reader.get_or_insert_with(|| {
            let reader = eq.create_reader();
            eq.register_reader::<X11WindowEvent>(reader);
            reader
        });

        if let Some(events) = eq.pull_events::<X11WindowEvent>(reader) {
            events
                .iter()
                .filter_map(|ev| {
                    if ev.window_handle() != self.handle() {
                        None
                    } else {
                        self.handle_window_event(ev)
                    }
                })
                .flatten()
                .for_each(|ev| eq.push_event(ev))
        }

        Ok(())
    }
}

impl X11Window {
    fn handle_window_event(&mut self, event: &X11WindowEvent) -> Option<Vec<WindowEvent>> {
        let mut output = vec![];

        match event {
            X11WindowEvent::ConfigureNotify {
                handle,
                position,
                size,
            } => {
                if position != &self.position {
                    let event = WindowEvent::Moved {
                        handle: *handle,
                        position: *position,
                    };
                    self.position = *position;
                    output.push(event);
                }

                if size != &self.size {
                    let event = WindowEvent::Resize {
                        handle: *handle,
                        size: *size,
                    };
                    self.size = *size;
                    output.push(event);
                }
            }

            X11WindowEvent::ClientMessage {
                handle,
                data: x::ClientMessageData::Data32([atom, ..]),
            } => {
                if atom == &self.atoms.delete_window.resource_id() {
                    self.close_requested = true;
                    let event = WindowEvent::Close(*handle);
                    output.push(event);
                }
            }

            X11WindowEvent::KeyPress { handle, keycode } => {
                output.push(WindowEvent::KeyPress {
                    handle: *handle,
                    keycode: *keycode as usize,
                });
            }

            X11WindowEvent::KeyRelease { handle, keycode } => {
                output.push(WindowEvent::KeyRelease {
                    handle: *handle,
                    keycode: *keycode as usize,
                });
            }

            X11WindowEvent::DestroyNotify { handle, .. } => {
                self.close_requested = true;
                output.push(WindowEvent::Close(*handle));
            }
            _ => {}
        }

        if output.is_empty() {
            None
        } else {
            Some(output)
        }
    }
}

impl Drop for X11Window {
    fn drop(&mut self) {
        let conn = &get_x11_context().conn;
        let result =
            conn.check_request(conn.send_request_checked(&x::DestroyWindow { window: self.id }));

        if let Err(err) = result {
            eprintln!("{err}")
        }
    }
}
