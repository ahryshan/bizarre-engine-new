use std::{
    any::TypeId,
    marker::{PhantomData, PhantomPinned},
};

use super::{
    component_storage::{Component, ComponentStorage},
    query_iterator::QueryIterator,
};

pub trait QueryElement<'a, T> {
    type LockType;
    type Item;
    type QEIterator;

    fn new(component: &'a Component) -> Self;
    fn get_lock(&self) -> Self::LockType;
    fn transform_iter<I>(iter: I) -> Self::QEIterator
    where
        I: Iterator<Item = &'a Component> + Clone;
}

pub trait QueryElementIterator<'a, E, T>
where
    E: QueryElement<'a, T>,
{
    fn from_iter(iter: impl Iterator<Item = &'a Component>) -> Self;
}

pub trait Query<'a, T, Inner, Iterators> {
    fn type_ids() -> Vec<TypeId>;
    fn combine_iters<I>(iters: Vec<I>) -> QueryIterator<'a, Iterators, T, Inner>
    where
        I: Iterator<Item = &'a Component> + Clone;
}

impl<'a, A, AA, AIter> Query<'a, A, AA, AIter> for A
where
    A: QueryElement<'a, AA, QEIterator = AIter>,
    AA: 'static,
    AIter: Iterator<Item = A>,
{
    fn type_ids() -> Vec<TypeId> {
        vec![TypeId::of::<AA>()]
    }

    fn combine_iters<I>(iters: Vec<I>) -> QueryIterator<'a, AIter, A, AA>
    where
        I: Iterator<Item = &'a Component> + Clone,
    {
        let [iter_a] = iters.as_slice() else { panic!() };
        let iters = A::transform_iter(iter_a.clone());
        QueryIterator {
            inner_iters: iters,
            _phantom_inner: Default::default(),
            _phantom_elements: Default::default(),
        }
    }
}

use paste::paste;

macro_rules! impl_query_inner {
    ( $($fetch_t:ident),+;  $($inner_t:ident),+;  $($iter_t:ident),+) => {
        impl<'a, $($fetch_t, $inner_t, $iter_t),+> Query<'a,($($fetch_t),+), ($($inner_t),+), ($($iter_t),+)> for ($($fetch_t),+)
         where
        $(
            $fetch_t: QueryElement<'a, $inner_t, QEIterator = $iter_t>,
            $inner_t: 'static,
            $iter_t: Iterator<Item = $fetch_t>
        ),+
        {
            fn type_ids() -> Vec<TypeId> {
                vec![$(TypeId::of::<$inner_t>()),+]
            }

            fn combine_iters<ComponentIter>(iters: Vec<ComponentIter>) -> QueryIterator<'a, ($($iter_t),+), ($($fetch_t),+), ($($inner_t),+)>
            where
                ComponentIter: Iterator<Item = &'a Component> + Clone,
            {
                let [$(paste!{[<iter_ $fetch_t:lower>]}),+] = iters.as_slice() else {
                    panic!()
                };
                let iters = (
                    $($fetch_t::transform_iter(
                        paste!{[<iter_ $fetch_t:lower>].clone()}
                    )),+
                );
                QueryIterator {
                    inner_iters: iters,
                    _phantom_inner: Default::default(),
                    _phantom_elements: Default::default(),
                }
            }
        }
    }
}

macro_rules! impl_query {
    (
        $f_head:tt, $($f_tail:tt),+;
        $in_head:tt, $($in_tail:tt),+;
        $it_head:tt, $($it_tail:tt),+
    ) => {
        impl_query_inner!(
            $f_head, $($f_tail),+;
            $in_head, $($in_tail),+;
            $it_head, $($it_tail),+
        );
        impl_query!($($f_tail),+; $($in_tail),+; $($it_tail),+);
    };
    ($f_head:tt; $in_head:tt; $it_head:tt) => {
    };
}

impl_query!(
    A,B,C,D,E,F,G,H,I,J,K,L,M;
    AA,BB,CC,DD,EE,FF,GG,HH,II,JJ,KK,LL,MM;
    AIter,BIter,CIter,DIter,EIter,FIter,GIter,HIter,IIter,JIter,KIter,LIter,MIter
);
