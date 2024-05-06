use std::{ops::Deref, sync::LazyLock};

pub struct X11Connection {
    pub connection: xcb::Connection,
    pub screen_num: i32,
}

impl Deref for X11Connection {
    type Target = xcb::Connection;

    fn deref(&self) -> &Self::Target {
        &self.connection
    }
}

pub fn get_x11_connection() -> &'static X11Connection {
    static CONNECTION: LazyLock<X11Connection> = LazyLock::new(|| {
        xcb::Connection::connect(None)
            .map_err(|err| panic!("Cannot connect to X11 display: {:?}", err))
            .map(|(connection, screen_num)| X11Connection {
                connection,
                screen_num,
            })
            .unwrap()
    });

    &CONNECTION
}
