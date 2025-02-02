use std::{
    cell::{LazyCell, OnceCell},
    sync::{LazyLock, OnceLock},
    thread::{self, ThreadId},
};

use sdl::{EventSubsystem, Sdl, VideoSubsystem};

static INIT_THREAD_ID: OnceLock<ThreadId> = OnceLock::new();

thread_local! {
    static SDL_CONTEXT: OnceCell<sdl::Sdl> = OnceCell::new();

    static SDL_VIDEO: OnceCell<sdl::VideoSubsystem> = OnceCell::new();

    static SDL_EVENTS: OnceCell<sdl::EventSubsystem> = OnceCell::new();
}

pub fn with_sdl<R, F: FnOnce(&Sdl) -> R>(f: F) -> R {
    SDL_CONTEXT.with(|cell| {
        let ctx = cell.get_or_init(init_sdl);
        panic_on_wrong_thread();
        f(ctx)
    })
}

pub fn with_sdl_video<R, F: FnOnce(&VideoSubsystem) -> R>(f: F) -> R {
    SDL_VIDEO.with(|cell| {
        let ctx = cell.get_or_init(init_video);
        panic_on_wrong_thread();
        f(ctx)
    })
}

pub fn with_sdl_events<R, F: FnOnce(&EventSubsystem) -> R>(f: F) -> R {
    SDL_EVENTS.with(|cell| {
        let ctx = cell.get_or_init(init_events);
        panic_on_wrong_thread();
        f(ctx)
    })
}

fn panic_on_wrong_thread() {
    let Some(thread_id) = INIT_THREAD_ID.get().copied() else {
        panic!("SDL is not initialized");
    };

    if std::thread::current().id() != thread_id {
        panic!("Trying to access sdl context outside the main thread!");
    }
}

fn init_sdl() -> sdl::Sdl {
    assert!(
        INIT_THREAD_ID.get().is_none(),
        "Failed to initialize sdl context: already initialized from another thread"
    );

    INIT_THREAD_ID.get_or_init(|| thread::current().id());

    sdl::init().unwrap_or_else(|err| panic!("Could not initialize SDL context: {err}"))
}

fn init_video() -> sdl::VideoSubsystem {
    with_sdl(|sdl| sdl.video())
        .unwrap_or_else(|err| panic!("Failed to initialize SDL video subsystem: {err}"))
}

fn init_events() -> sdl::EventSubsystem {
    with_sdl(|sdl| sdl.event())
        .unwrap_or_else(|err| panic!("Failed to initialize SDL event subsystem: {err}"))
}
