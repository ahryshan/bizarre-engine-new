use crate::{component::Component, resource::Resource};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Health(pub usize);

impl Component for Health {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mana(pub usize);

impl Component for Mana {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Motd(pub &'static str);

impl Resource for Motd {}
