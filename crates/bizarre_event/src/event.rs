pub trait Event: 'static {}

impl<T> Event for T where T: Send + Sync + 'static {}
