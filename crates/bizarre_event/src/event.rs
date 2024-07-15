pub trait Event: Send + Sync + 'static {}

impl<T> Event for T where T: Sync + Send + 'static {}
