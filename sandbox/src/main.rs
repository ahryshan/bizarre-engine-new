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

struct PrintResource;

impl System for PrintResource {
    type QueryData<'q> = Res<'q, NamedRes>;

    fn run<'q>(&mut self, query: Query<'q, Self::QueryData<'q>>) {
        let resource = query.into_iter().next().unwrap();
        println!("{resource:?}")
    }
}

fn print_res(res: &NamedRes) {
    dbg!(res);
}

fn main() {
    let mut world = World::default();

    let res = NamedRes { name: "John" };
    world.insert_resource(res).unwrap();

    world.add_system(PrintResource, "print_resource").unwrap();

    let removed_res = world.remove_resource::<NamedRes>().unwrap();
}

impl Drop for NamedRes {
    fn drop(&mut self) {
        println!("{} was dropped", self.name);
    }
}
