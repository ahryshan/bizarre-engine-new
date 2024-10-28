use nalgebra_glm::{IVec2, Vec2};
use wayland_client::{
    protocol::{wl_pointer, wl_shm},
    QueueHandle, WEnum,
};

use crate::{window_events::WindowEvent, WindowHandle, WindowMode, WindowStatus};

use super::wl_window::WlWindowResources;

pub(crate) struct WlWindowState {
    pub(crate) handle: WindowHandle,
    pub(crate) internal_event_queue: Vec<WindowEvent>,
    pub(crate) size: IVec2,
    pub(crate) title: String,
    pub(crate) pointer_input_frame: Vec<wl_pointer::Event>,
    pub(crate) mode: WindowMode,
    pub(crate) close_requested: bool,
    pub(crate) resources: WlWindowResources,
    pub(crate) status: WindowStatus,
}

impl WlWindowState {
    pub(crate) fn resize(&mut self, qh: &QueueHandle<WlWindowState>, width: i32, height: i32) {
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

    pub(crate) fn handle_pointer_input_frame(&mut self) {
        struct InputFrameInfo {
            scroll_delta: Vec2,
            source: Option<wl_pointer::AxisSource>,
        }

        let input_info = self.pointer_input_frame.drain(..).fold(
            InputFrameInfo {
                scroll_delta: Vec2::zeros(),
                source: None,
            },
            |acc, curr| match curr {
                wl_pointer::Event::Axis { axis, value, .. } => match axis {
                    WEnum::Value(wl_pointer::Axis::VerticalScroll) => InputFrameInfo {
                        scroll_delta: acc.scroll_delta + Vec2::new(0.0, value as f32),
                        ..acc
                    },
                    WEnum::Value(wl_pointer::Axis::HorizontalScroll) => InputFrameInfo {
                        scroll_delta: acc.scroll_delta + Vec2::new(value as f32, 0.0),
                        ..acc
                    },
                    _ => acc,
                },
                wl_pointer::Event::AxisSource { axis_source } => match axis_source {
                    WEnum::Value(val) => InputFrameInfo {
                        source: Some(val),
                        ..acc
                    },
                    _ => acc,
                },
                _ => acc,
            },
        );

        match input_info.source {
            Some(wl_pointer::AxisSource::Wheel) | Some(wl_pointer::AxisSource::WheelTilt)
                if input_info.scroll_delta != Vec2::new(0.0, 0.0) =>
            {
                let button = match <[f32; 2]>::from(input_info.scroll_delta) {
                    [0.0, y] if y > 0.0 => 12,
                    [0.0, y] if y < 0.0 => 13,
                    [x, 0.0] if x > 0.0 => 14,
                    [x, 0.0] if x < 0.0 => 15,
                    _ => unreachable!(),
                };

                self.internal_event_queue.extend_from_slice(&[
                    WindowEvent::Scroll {
                        handle: self.handle,
                        delta: input_info.scroll_delta,
                    },
                    WindowEvent::ButtonPress {
                        handle: self.handle,
                        button,
                    },
                    WindowEvent::ButtonRelease {
                        handle: self.handle,
                        button,
                    },
                ]);
            }
            Some(_) if input_info.scroll_delta != Vec2::new(0.0, 0.0) => {
                self.internal_event_queue.push(WindowEvent::Scroll {
                    handle: self.handle,
                    delta: input_info.scroll_delta,
                })
            }
            _ => {}
        }
    }
}
