use std::sync::LazyLock;

use xcb::x;

use crate::WindowAction;

use super::connection::get_x11_connection;

xcb::atoms_struct! {
#[derive(Clone, Copy, Debug)]
pub(crate) struct X11WindowActionAtoms {
        pub move_action => b"_NET_WM_ACTION_MOVE",
        pub resize => b"_NET_WM_ACTION_RESIZE",
        pub minimize => b"_NET_WM_ACTION_MINIMIZE",
        pub shade => b"_NET_WM_ACTION_SHADE",
        pub stick => b"_NET_WM_ACTION_STICK",
        pub maximize => b"_NET_WM_ACTION_MAXIMIZE",
        pub fullscreen => b"_NET_WM_ACTION_FULLSCREEN",
        pub change_desktop => b"_NET_WM_ACTION_CHANGE_DESKTOP",
        pub close => b"_NET_WM_ACTION_CLOSE",
    }
}

static WINDOW_ACTION_ATOMS: LazyLock<X11WindowActionAtoms> = LazyLock::new(|| {
    let conn = get_x11_connection();
    X11WindowActionAtoms::intern_all(conn).unwrap()
});

impl From<WindowAction> for x::Atom {
    fn from(value: WindowAction) -> Self {
        match value {
            WindowAction::Move => WINDOW_ACTION_ATOMS.move_action,
            WindowAction::Resize => WINDOW_ACTION_ATOMS.resize,
            WindowAction::Minimize => WINDOW_ACTION_ATOMS.minimize,
            WindowAction::Shade => WINDOW_ACTION_ATOMS.shade,
            WindowAction::Stick => WINDOW_ACTION_ATOMS.stick,
            WindowAction::Maximize => WINDOW_ACTION_ATOMS.maximize,
            WindowAction::Fullscreen => WINDOW_ACTION_ATOMS.fullscreen,
            WindowAction::ChangeDesktop => WINDOW_ACTION_ATOMS.change_desktop,
            WindowAction::Close => WINDOW_ACTION_ATOMS.close,
        }
    }
}
