pub trait Event: Clone + Send + Sync + 'static {}

impl<T> Event for T where T: Clone + Sync + Send + 'static {}
