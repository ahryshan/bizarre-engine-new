use std::{fmt::Display, ops::Deref, time::Duration};

use bizarre_ecs::prelude::*;

#[derive(Resource, Debug)]
pub struct DeltaTime(pub(crate) Duration);

impl Deref for DeltaTime {
    type Target = Duration;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for DeltaTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

#[derive(Resource, Debug)]
pub struct AppRunTime(pub(crate) Duration);

impl Deref for AppRunTime {
    type Target = Duration;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for AppRunTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
