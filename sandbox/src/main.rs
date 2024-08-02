use anyhow::Result;
use bizarre_engine::ecs::{
    entity::Entity, query::Query, system::system_graph::SystemGraph, world::World,
};

#[derive(Debug)]
struct Health(pub u32);

fn list_entities(query: Query<(Entity, &Health)>) {
    for entity in query {
        println!("{entity:?}")
    }
}

fn main() -> Result<()> {
    let mut world = World::new();

    world.register_component::<Health>();

    let entity = world.create_entity();
    world.insert_component(entity, Health(100));

    world.create_entity();

    let entity = world.create_entity();
    world.insert_component(entity, Health(200));

    println!("{:?}", world.component::<Health>(entity));

    world.create_entity();
    world.create_entity();

    let mut sg = SystemGraph::new();

    sg.add_system(list_entities);

    sg.init_systems(&world);

    sg.run_systems(&mut world);

    Ok(())
}
