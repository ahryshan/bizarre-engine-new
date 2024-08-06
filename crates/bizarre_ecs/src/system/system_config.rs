use std::{any::type_name, fmt::Debug};

use bizarre_utils::mass_impl;

use super::{system_graph::SystemGraph, IntoSystem, System, WorldAccess};

#[derive(Debug, Clone)]
pub struct SystemMeta {
    pub(crate) name: &'static str,
    pub(crate) before: Vec<&'static str>,
    pub(crate) after: Vec<&'static str>,
    pub(crate) access: Box<[WorldAccess]>,
}

impl SystemMeta {
    pub fn new<M, T: IntoSystem<M>>() -> Self {
        Self {
            name: type_name::<T>(),
            access: T::System::access(),
            before: Default::default(),
            after: Default::default(),
        }
    }
}

pub struct SystemConfig {
    pub meta: SystemMeta,
    pub system: Box<dyn System>,
}

impl Debug for SystemConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SystemConfig")
            .field("meta", &self.meta)
            .field("system", &"BoxedSystem")
            .finish()
    }
}

#[derive(Debug)]
pub enum SystemConfigs {
    Config(SystemConfig),
    Configs(Vec<SystemConfigs>),
}

impl SystemConfigs {
    pub fn names(&self) -> Vec<&'static str> {
        match self {
            SystemConfigs::Config(conf) => vec![conf.meta.name],
            SystemConfigs::Configs(confs) => confs.iter().flat_map(|c| c.names()).collect(),
        }
    }

    pub fn after_inner(&mut self, names: &[&'static str]) {
        match self {
            SystemConfigs::Config(conf) => conf.meta.after.extend(names),
            SystemConfigs::Configs(confs) => confs.iter_mut().for_each(|c| c.after_inner(names)),
        }
    }

    pub fn before_inner(&mut self, names: &[&'static str]) {
        match self {
            SystemConfigs::Config(conf) => conf.meta.before.extend(names),
            SystemConfigs::Configs(confs) => confs.iter_mut().for_each(|c| c.before_inner(names)),
        }
    }
}

pub trait IntoSystemConfigs<Marker>
where
    Self: Sized,
{
    fn into_system_configs(self) -> SystemConfigs;

    fn after<M>(self, other: impl IntoSystemConfigs<M>) -> SystemConfigs {
        self.into_system_configs().after(other)
    }

    fn before<M>(self, other: impl IntoSystemConfigs<M>) -> SystemConfigs {
        self.into_system_configs().before(other)
    }
}

impl IntoSystemConfigs<()> for SystemConfig {
    fn into_system_configs(self) -> SystemConfigs {
        SystemConfigs::Config(self)
    }
}

impl IntoSystemConfigs<()> for SystemConfigs {
    fn into_system_configs(self) -> SystemConfigs {
        self
    }

    fn after<M>(mut self, other: impl IntoSystemConfigs<M>) -> SystemConfigs {
        let names = other.into_system_configs().names();
        self.after_inner(&names);
        self
    }

    fn before<M>(mut self, other: impl IntoSystemConfigs<M>) -> SystemConfigs {
        let names = other.into_system_configs().names();
        self.before_inner(&names);
        self
    }
}

impl<M, T> IntoSystemConfigs<M> for T
where
    T: IntoSystem<M>,
{
    fn into_system_configs(self) -> SystemConfigs {
        SystemConfigs::Config(SystemConfig {
            meta: SystemMeta::new::<M, T>(),
            system: Box::new(self.into_system()),
        })
    }
}

macro_rules! impl_into_system_configs {
    ($(($config:tt, $marker:tt)),+) => {
        #[allow(non_snake_case)]
        impl<$($config: IntoSystemConfigs<$marker>, $marker),+> IntoSystemConfigs<($($marker,)+)> for ($($config,)+) {
            fn into_system_configs(self) -> SystemConfigs {
                let ($($config,)+) = self;
                SystemConfigs::Configs(
                    vec![$($config.into_system_configs()),+]
                )
            }
        }
    };
}

mass_impl!(impl_into_system_configs, 16, C, M);
