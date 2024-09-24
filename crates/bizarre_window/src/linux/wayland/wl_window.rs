use nalgebra_glm::{IVec2, UVec2};

use crate::{window_error::WindowResult, PlatformWindow};

use super::wl_context::WL_CONTEXT;

pub struct WlWindow {}

impl PlatformWindow for WlWindow {
    fn new(create_info: &crate::WindowCreateInfo) -> WindowResult<Self>
    where
        Self: Sized,
    {
        WL_CONTEXT.read().unwrap();
        todo!();
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

    fn handle(&self) -> crate::WindowHandle {
        todo!()
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
        todo!()
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
        todo!()
    }

    fn close_requested(&self) -> bool {
        todo!()
    }
}
