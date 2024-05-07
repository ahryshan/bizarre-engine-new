use crate::{WindowCreateInfo, WindowTrait};

use super::wayland_window::WaylandWindow;

impl WindowTrait for WaylandWindow {
    fn new(create_info: &WindowCreateInfo) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(WaylandWindow::init(create_info)?)
    }

    fn size(&self) -> nalgebra_glm::UVec2 {
        todo!()
    }

    fn position(&self) -> nalgebra_glm::IVec2 {
        todo!()
    }

    fn update_size_and_position(
        &mut self,
    ) -> anyhow::Result<(nalgebra_glm::UVec2, nalgebra_glm::IVec2)> {
        todo!()
    }

    fn mode(&self) -> crate::WindowMode {
        todo!()
    }

    fn raw_handle(&self) -> u32 {
        todo!()
    }

    fn title(&self) -> &str {
        todo!()
    }

    fn status(&self) -> crate::window::WindowStatus {
        todo!()
    }

    fn set_size(&mut self, size: nalgebra_glm::UVec2) -> anyhow::Result<()> {
        todo!()
    }

    fn set_position(&mut self, position: nalgebra_glm::IVec2) -> anyhow::Result<()> {
        todo!()
    }

    fn set_mode(&mut self, mode: crate::WindowMode) -> anyhow::Result<()> {
        todo!()
    }

    fn set_title(&mut self, title: String) -> anyhow::Result<()> {
        todo!()
    }

    fn set_decorations(&mut self, decorations: bool) -> anyhow::Result<()> {
        todo!()
    }

    fn map(&mut self) -> anyhow::Result<()> {
        todo!()
    }

    fn unmap(&mut self) -> anyhow::Result<()> {
        todo!()
    }

    fn minimize(&mut self) -> anyhow::Result<()> {
        todo!()
    }

    fn restore(&mut self) -> anyhow::Result<()> {
        todo!()
    }

    fn maximize(&mut self) -> anyhow::Result<()> {
        todo!()
    }

    fn unmaximize(&mut self) -> anyhow::Result<()> {
        todo!()
    }

    fn close_requested(&self) -> bool {
        self.state.should_close
    }
}
