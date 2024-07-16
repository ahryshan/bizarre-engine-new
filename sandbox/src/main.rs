use bizarre_engine::ecs::world::World;

#[derive(Debug)]
pub struct Res {
    name: &'static str,
}

fn print_res(res: &Res) {
    dbg!(res);
}

fn main() {
    let mut world = World::default();

    let res = Res { name: "John" };
    world.insert_resource(res).unwrap();

    world.with_resource(print_res).unwrap();
    let _ = world.with_resource_mut(|res: &mut Res| res.name = "George");
    let _ = world.with_resource(print_res);

    let removed_res = world.remove_resource::<Res>().unwrap();
    dbg!(removed_res);
}

impl Drop for Res {
    fn drop(&mut self) {
        println!("{} was dropped", self.name);
    }
}
