use std::marker::PhantomData;

use crate::world::World;

use super::query_data::QueryData;

pub struct QueryIterator<'q, D: QueryData<'q>, I> {
    pub(crate) iter: I,
    pub(crate) _phantom: PhantomData<&'q D>,
}

impl<'q, D: QueryData<'q>, I> Iterator for QueryIterator<'q, D, I>
where
    I: Iterator<Item = D::Item>,
{
    type Item = D::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}
