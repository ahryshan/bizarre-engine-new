use anyhow::Result;
use bizarre_engine::ecs::{
    query::{fetch::Fetch, res::Res, Query},
    system::schedule::Schedule,
    world::{commands::Commands, World},
    Component, Entity, Resource, System,
};

pub struct NameComponent(&'static str);

impl Component for NameComponent {}

pub struct GreetEntity;

impl System for GreetEntity {
    type RunData = (Entity, Fetch<NameComponent>);

    fn run(&mut self, query: Query<Self::RunData>, _: &mut Commands) {
        for (entity, name) in query {
            println!("Hello, {} ({entity:?})", name.0);
        }
    }
}

pub struct GreetEntityAgain;

impl System for GreetEntityAgain {
    type RunData = (Entity, Fetch<NameComponent>);

    fn run(&mut self, query: Query<Self::RunData>, _: &mut Commands) {
        for (entity, name) in query {
            println!("Hello again, {} ({entity:?})", name.0);
        }
    }
}

pub struct GreetWorld;

impl System for GreetWorld {
    fn run(&mut self, query: Query<Self::RunData>, _: &mut Commands) {
        println!("Hello World!");
    }
}

pub struct KillAllJohns;

impl System for KillAllJohns {
    type RunData = (Entity, Fetch<NameComponent>);

    fn run(&mut self, query: Query<Self::RunData>, commands: &mut Commands) {
        let e = query
            .into_iter()
            .filter_map(|(e, n)| if n.0 == "John" { Some(e) } else { None })
            .collect::<Vec<_>>();

        commands.kill_entities(e.as_slice());
    }
}

pub struct ReviveJohn;

impl System for ReviveJohn {
    type RunData = Fetch<NameComponent>;

    fn run(&mut self, query: Query<Self::RunData>, commands: &mut Commands) {
        let count = query.into_iter().filter(|name| name.0 == "John").count();

        if count < 1 {
            commands.entity_with(|cmd| {
                cmd.with_component(NameComponent("John"));
            });
        }
    }
}

fn main() -> Result<()> {
    let mut world = World::default();

    world.spawn().with_component(NameComponent("John")).build();
    world.spawn().with_component(NameComponent("Don")).build();
    world.spawn().with_component(NameComponent("Dog")).build();
    world.spawn().with_component(NameComponent("Meat")).build();

    world.flush();

    world.add_system(Schedule::Frame, GreetWorld, "greet_world")?;

    world.add_system_with_dependencies(
        Schedule::Frame,
        GreetEntity,
        "greet_entity",
        &["greet_world"],
    )?;

    // world.add_system_with_dependencies(
    //     Schedule::Frame,
    //     GreetEntityAgain,
    //     "greet_entity_again",
    //     &["greet_world", "greet_entity"],
    // )?;

    world.add_system(Schedule::Frame, KillAllJohns, "kill_all_johns");
    world.add_system(Schedule::Frame, ReviveJohn, "revive_john");

    world.run_schedule(Schedule::Frame);
    world.run_schedule(Schedule::Frame);
    world.run_schedule(Schedule::Frame);

    Ok(())
}
