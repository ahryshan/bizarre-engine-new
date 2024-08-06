use std::time::Instant;

use anyhow::Result;
use bizarre_engine::ecs::commands::Commands;
use bizarre_engine::ecs::system::schedule::Schedule;
use bizarre_engine::ecs::system::system_config::IntoSystemConfigs;
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

fn spawn(mut commands: Commands) {
    for _ in 0..100000 {
        commands.spawn_empty();
    }

    for _ in 0..80000 {
        commands.spawn(Strength(100));
    }
}

fn count_entities(strength: Query<&Strength>, all: Query<Entity>) {
    let all = all.into_iter().collect::<Vec<_>>();
    let all_count = all.len();
    let weak = strength.clone().into_iter().filter(|s| s.0 < 100).count();
    let strong = strength.clone().into_iter().filter(|s| s.0 >= 100).count();

    println!("There are {all_count} entities overall, {weak} are weak and {strong} are strong");
}

fn kill_weak(
    mut start_counter: Local<usize>,
    mut commands: Commands,
    query: Query<(Entity, &Strength)>,
) {
    if *start_counter < 5 {
        *start_counter += 1;
        return;
    }

    let iter = query.into_iter().filter(|(e, s)| s.0 < 100);

    let count = iter.clone().count();
    let count = ((count as f32 * 0.5).floor() + 1.0) as usize;

    for (e, _) in iter.clone().take(count) {
        commands.entity(e).kill();
    }
}

fn main() {
    let mut world = World::new();

    world.register_component::<Strength>();

    world.add_schedule(Schedule::Update);

    world.add_systems(Schedule::Update, (spawn, count_entities));

    world.init_schedule(Schedule::Update);

    world.run_schedule(Schedule::Update);
    world.run_schedule(Schedule::Update);
    world.run_schedule(Schedule::Update);
}
