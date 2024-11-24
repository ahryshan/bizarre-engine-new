use bizarre_log::{init_logging, shutdown_logging};
use bizarre_render::renderer::VulkanRenderer;

fn main() {
    init_logging(None, None);

    {
        let renderer = VulkanRenderer::new().unwrap();
    }

    shutdown_logging();
}
