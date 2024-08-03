use anyhow::Result;
use bizarre_engine::ecs::{
    component::Component, entity::Entity, query::Query, system::system_graph::SystemGraph,
    world::World,
};

use bizarre_engine::prelude::*;

#[derive(Component, Debug, PartialEq, Eq)]
struct Health(pub u32);

#[derive(Component, Debug)]
struct Mana(pub u32);

#[derive(Component, Debug)]
struct Strength(pub u32);

fn list_entities(query: Query<(Entity, &Health, &Mana, &Strength)>) {
    for entity in query {
        println!("{entity:?}")
    }
}

fn main() -> Result<()> {
    let mut world = World::new();

    world.register_component::<Health>();
    world.register_component::<Mana>();
    world.register_component::<Strength>();

    let entity = world.spawn_entity((Health(100), Mana(20), Strength(12)));
    let entity = world.spawn_entity((Health(200), Strength(12)));
    let entity = world.spawn_entity((Health(300), Mana(30), Strength(12)));
    let entity = world.spawn_entity((Health(400), Mana(40), Strength(12)));

    let mut sg = SystemGraph::new();

    sg.add_system(list_entities);

    sg.init_systems(&world);

    sg.run_systems(&mut world);

    Ok(())
}
