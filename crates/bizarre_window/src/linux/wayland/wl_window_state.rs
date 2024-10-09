use bizarre_log::core_trace;
use nalgebra_glm::{IVec2, UVec2};
use wayland_client::{protocol::wl_shm, QueueHandle};

use crate::{window_events::WindowEvent, WindowHandle, WindowMode, WindowStatus};

use super::wl_window::WlWindowResources;

pub(crate) struct WlWindowState {
    pub(crate) handle: WindowHandle,
    pub(crate) internal_event_queue: Vec<WindowEvent>,
    pub(crate) size: IVec2,
    pub(crate) position: IVec2,
    pub(crate) title: String,
    pub(crate) mode: WindowMode,
    pub(crate) close_requested: bool,
    pub(crate) resources: WlWindowResources,
    pub(crate) status: WindowStatus,
}

impl WlWindowState {
    pub fn resize(&mut self, qh: &QueueHandle<WlWindowState>, width: i32, height: i32) {
        if self.size.x == width && self.size.y == height {
            return;
        }

        self.size = [width, height].into();

        let stride = width * 4;
        let pool_size = (height * 2 * stride) as usize;

        self.resources.buffer.destroy();

        if pool_size > self.resources.shm.size() {
            self.resources.shm.resize(pool_size);
            self.resources.pool.resize(pool_size as i32);
        }

        self.resources.buffer = self.resources.pool.create_buffer(
            0,
            width as i32,
            height as i32,
            stride as i32,
            wl_shm::Format::Xrgb8888,
            qh,
            (),
        );

        self.resources
            .xdg_surface
            .set_window_geometry(0, 0, width as i32, height as i32);

        self.resources
            .surface
            .attach(Some(&self.resources.buffer), 0, 0);

        self.resources.surface.commit();
    }
}
