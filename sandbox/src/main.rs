use anyhow::Result;
use bizarre_engine::ecs::{
    query::{fetch::Fetch, res::Res, Query},
    system::schedule::Schedule,
    world::World,
    Component, Resource, System,
};

pub struct NameComponent(&'static str);

impl Component for NameComponent {}

pub struct GreetEntity;

impl System for GreetEntity {
    type QueryData<'q> = Fetch<'q, NameComponent>;

    fn run<'q>(&mut self, query: Query<'q, Self::QueryData<'q>>) {
        for name in query {
            println!("Hello, {}", name.0);
        }
    }
}

pub struct GreetEntityAgain;

impl System for GreetEntityAgain {
    type QueryData<'q> = Fetch<'q, NameComponent>;

    fn run<'q>(&mut self, query: Query<'q, Self::QueryData<'q>>) {
        for name in query {
            println!("Hello again, {}", name.0);
        }
    }
}

pub struct GreetWorld;

impl System for GreetWorld {
    type QueryData<'q> = ();

    fn run<'q>(&mut self, query: Query<'q, Self::QueryData<'q>>) {
        println!("Hello World!");
    }
}

fn main() -> Result<()> {
    let mut world = World::default();

    world.spawn().with_component(NameComponent("John"));
    world.spawn().with_component(NameComponent("Don"));
    world.spawn().with_component(NameComponent("Dog"));
    world.spawn().with_component(NameComponent("Meat"));

    world.add_system(Schedule::Frame, GreetWorld, "greet_world")?;

    world.add_system_with_dependencies(
        Schedule::Frame,
        GreetEntity,
        "greet_entity",
        &["greet_world"],
    )?;

    world.add_system_with_dependencies(
        Schedule::Frame,
        GreetEntityAgain,
        "greet_entity_again",
        &["greet_world", "greet_entity"],
    )?;

    world.run_schedule(Schedule::Frame);

    Ok(())
}
