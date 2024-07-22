use crate::{
    entity::{entities::Entities, query::Query, query_data::QueryData},
    resource::{
        registry::{ResourceReadLock, ResourceRegistry, ResourceResult, ResourceWriteLock},
        IntoResource,
    },
};

#[derive(Default)]
pub struct World {
    pub(crate) resources: ResourceRegistry,
    pub(crate) entities: Entities,
}

impl World {
    pub fn get_resource<T>(&self) -> ResourceResult<ResourceReadLock<T>>
    where
        T: 'static,
    {
        self.resources.get()
    }

    pub fn get_resource_mut<T>(&self) -> ResourceResult<ResourceWriteLock<T>>
    where
        T: 'static,
    {
        self.resources.get_mut()
    }

    pub fn with_resource<T>(&self, closure: impl Fn(&T)) -> ResourceResult<()>
    where
        T: 'static,
    {
        self.resources.with_resource(closure)
    }

    pub fn with_resource_mut<T>(&self, closure: impl Fn(&mut T)) -> ResourceResult<()>
    where
        T: 'static,
    {
        self.resources.with_resource_mut(closure)
    }

    pub fn insert_resource<T>(&mut self, resource: T) -> ResourceResult<()>
    where
        T: 'static + IntoResource,
    {
        self.resources.insert(resource)
    }

    pub fn remove_resource<T>(&mut self) -> ResourceResult<T>
    where
        T: 'static,
    {
        self.resources.remove()
    }

    pub fn query<'q, D: QueryData<'q>>(&'q self) -> Query<'q, D> {
        Query::<'q, D>::new(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::entity::{fetch::Fetch, query::Query, query_element::QueryElement};

    use super::World;

    #[derive(Debug)]
    struct Health(u32);
    #[derive(Debug)]
    struct Velocity(f64);
    #[derive(Debug)]
    struct Acceleration(f64);
    #[derive(Debug)]
    struct Name(&'static str);

    #[test]
    fn should_query_one_component() {
        let mut world = World::default();

        world.entities.spawn().with_component(Health(100));
        world.entities.spawn().with_component(Health(100));
        world.entities.spawn().with_component(Health(100));
        world.entities.spawn().with_component(Health(100));
        world.entities.spawn().with_component(Health(100));
        world.entities.spawn().with_component(Health(100));
        world.entities.spawn().with_component(Health(100));

        let query: Query<Fetch<Health>> = world.query::<Fetch<Health>>();

        for data in query {
            let health = data.get_lock();
            dbg!(health);
        }
    }

    #[test]
    fn should_query_multiple_components() {
        let mut world = World::default();

        world
            .entities
            .spawn()
            .with_component(Name("With Health and Acceleration"))
            .with_component(Health(100))
            .with_component(Acceleration(200.0));

        world
            .entities
            .spawn()
            .with_component(Name("With Health and Velocity"))
            .with_component(Health(100))
            .with_component(Velocity(109.0));

        world
            .entities
            .spawn()
            .with_component(Name("With Velocity and Acceleration"))
            .with_component(Acceleration(3000.0))
            .with_component(Velocity(20.1));

        let query: Query<(Fetch<Name>, Fetch<Acceleration>)> = world.query();

        for (i, (name, acc)) in query.into_iter().enumerate() {
            eprintln!("#{i} Query ({:?}, {:?})", &name.get_lock(), &acc.get_lock());
        }
    }
}
