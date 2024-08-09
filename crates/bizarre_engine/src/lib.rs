pub use bizarre_app as app;
pub use bizarre_core as core;
pub use bizarre_ecs as ecs;
pub use bizarre_event as event;
pub use bizarre_window as window;

pub mod ecs_modules;

pub mod prelude {
    pub use bizarre_ecs::prelude::*;
}
