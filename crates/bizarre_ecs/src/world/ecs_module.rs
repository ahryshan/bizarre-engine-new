use super::World;

pub trait EcsModule {
    fn apply(self, world: &mut World);
}
