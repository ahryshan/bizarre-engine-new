use bizarre_ecs::{
    system::{system_config::IntoSystemConfigs, system_graph::SystemGraph},
    world::World,
};

fn first_system() {
    println!("First system");
}

fn second_system() {
    println!("Second system");
}

fn third_system() {
    println!("Third system");
}

fn fourth_system() {
    println!("Fourth system");
}

fn unrelated_system() {
    println!("Unrelated system");
}

fn main() {
    let mut sg = SystemGraph::new();

    sg.add_systems((
        fourth_system.after(third_system),
        third_system
            .after((second_system, unrelated_system))
            .before(fourth_system),
        unrelated_system.after(first_system),
        second_system.after(first_system).before(fourth_system),
        first_system.before((second_system, third_system, fourth_system)),
    ));

    let mut world = World::new();

    sg.init_systems(&mut world);
    sg.run_systems(&mut world);
}
