use anyhow::Result;
use bizarre_engine::ecs::system::system_param::{Local, Res, ResMut};
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

#[derive(Resource, Debug)]
struct DeltaTime(f64);

#[derive(Resource, Debug)]
struct RunTime(f64);

fn list_entities(delta: Res<DeltaTime>, query: Query<(Entity, &mut Health, &Mana, &Strength)>) {
    for entity in query {
        println!("{entity:?}");
        let (_, health, ..) = entity;
        health.0 += (delta.0 * 10.0) as u32;
    }
}

fn update_run_time(delta: Res<DeltaTime>, mut run_time: ResMut<RunTime>) {
    run_time.0 += delta.0;
}

fn print_times(delta: Res<DeltaTime>, run_time: Res<RunTime>) {
    println!("Delta: {:?}, runtime: {:?}", &delta, &run_time);
}

fn counter(mut counter: Local<u32>) {
    println!("Counter has been run {} times before", *counter);
    *counter += 1
}

fn main() -> Result<()> {
    let mut world = World::new();

    world.register_component::<Health>();
    world.register_component::<Mana>();
    world.register_component::<Strength>();

    world.insert_resources((DeltaTime(0.16), RunTime(0.0)));

    let entity = world.spawn_entity((Health(100), Mana(20), Strength(12)));
    let entity = world.spawn_entity((Health(200), Strength(12)));
    let entity = world.spawn_entity((Health(300), Mana(30), Strength(12)));
    let entity = world.spawn_entity((Health(400), Mana(40), Strength(12)));

    let mut sg = SystemGraph::new();

    sg.add_system(update_run_time);
    sg.add_system(print_times);
    sg.add_system(list_entities);
    sg.add_system(counter);

    sg.init_systems(&world);

    sg.run_systems(&mut world);
    sg.run_systems(&mut world);
    sg.run_systems(&mut world);
    sg.run_systems(&mut world);
    sg.run_systems(&mut world);

    Ok(())
}
