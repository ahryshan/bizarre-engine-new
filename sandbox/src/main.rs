use bizarre_engine::ecs::{
    query::{res::Res, Query},
    world::World,
    Resource, System,
};

#[derive(Debug)]
pub struct NamedRes {
    name: &'static str,
}

impl Resource for NamedRes {}

struct PrintRes;

impl System for PrintRes {
    type QueryData<'q> = Res<'q, NamedRes>;

    fn run<'q>(&mut self, query: Query<'q, Self::QueryData<'q>>) {
        let res = query.into_iter().next().unwrap();
        println!("{res:?}")
    }
}

fn main() {
    let mut world = World::default();

    let res = NamedRes { name: "John" };
    world.insert_resource(res).unwrap();

    world.add_system(PrintRes, "print_resource").unwrap();

    world.run_systems();

    let removed_res = world.remove_resource::<NamedRes>().unwrap();
}

impl Drop for NamedRes {
    fn drop(&mut self) {
        println!("{} was dropped", self.name);
    }
}
