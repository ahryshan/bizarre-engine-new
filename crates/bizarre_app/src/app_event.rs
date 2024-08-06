#[derive(Clone)]
pub enum AppEvent {
    CloseRequested,
    WillClose,
    PauseRequested,
    Paused,
    ResumeRequested,
    Resumed,
}
