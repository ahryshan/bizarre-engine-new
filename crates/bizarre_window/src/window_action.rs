/// Allowed window actions
///
/// Modeled after X11 `_NET_WM_ALLOWED_ACTIONS` Atom
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowAction {
    Move,
    Resize,
    Minimize,
    Shade,
    Stick,
    Maximize,
    Fullscreen,
    ChangeDesktop,
    Close,
}

impl WindowAction {
    pub fn all() -> Vec<Self> {
        vec![
            WindowAction::Move,
            WindowAction::Resize,
            WindowAction::Minimize,
            WindowAction::Shade,
            WindowAction::Stick,
            WindowAction::Maximize,
            WindowAction::Fullscreen,
            WindowAction::ChangeDesktop,
            WindowAction::Close,
        ]
    }
}
