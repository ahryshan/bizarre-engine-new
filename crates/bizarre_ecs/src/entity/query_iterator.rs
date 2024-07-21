use std::marker::PhantomData;

use super::query::QueryElement;

pub struct QueryIterator<'a, Iter, QElements, Inner> {
    pub(crate) inner_iters: Iter,
    pub(crate) _phantom_elements: PhantomData<&'a QElements>,
    pub(crate) _phantom_inner: PhantomData<Inner>,
}

impl<'a, T, Inner, Iter> Iterator for QueryIterator<'a, Iter, T, Inner>
where
    T: QueryElement<'a, Inner, QEIterator = Iter>,
    Inner: 'static,
    Iter: Iterator<Item = T>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner_iters.next()
    }
}

use paste::paste;

macro_rules! impl_iterator_inner {
    ( $($fetch_t:ident),+;  $($inner_t:ident),+;  $($iter_t:ident),+) => {
        impl<'a, $($fetch_t, $inner_t, $iter_t),+> Iterator for
        QueryIterator<'a, ($($iter_t),+), ($($fetch_t),+), ($($inner_t),+)> where
        $(
            $fetch_t: QueryElement<'a, $inner_t, QEIterator = $iter_t>,
            $inner_t: 'static,
            $iter_t: Iterator<Item = $fetch_t>
        ),+
        {
            type Item = ($($fetch_t),+);

            fn next(&mut self) -> Option<Self::Item> {
                let ($(paste!{[<iter_ $fetch_t:lower>]}),+) = &mut self.inner_iters;
                let ($(paste!{[<item_ $fetch_t:lower>]}),+) = ($(paste!{[<iter_ $fetch_t:lower>].next()}),+);
                match ($(paste!{[<item_ $fetch_t:lower>]}),+) {
                    ($(paste!{Some([<item_ $fetch_t:lower>])}),+) => Some(($(paste!{[<item_ $fetch_t:lower>]}),+)),
                    _ => None,
                }
            }
        }
    }
}

macro_rules! impl_iterator {
    (
        $f_head:tt, $($f_tail:tt),+;
        $in_head:tt, $($in_tail:tt),+;
        $it_head:tt, $($it_tail:tt),+
    ) => {
        impl_iterator_inner!(
            $f_head, $($f_tail),+;
            $in_head, $($in_tail),+;
            $it_head, $($it_tail),+
        );
        impl_iterator!($($f_tail),+; $($in_tail),+; $($it_tail),+);
    };
    ($f_head:tt; $in_head:tt; $it_head:tt) => {
    };
}

// macro_rules! expand_types{
//     ($($t:tt),+) => {
//         $($t),+;
//         $(paste!{[<$t $t>]}),+;
//         $(paste!{[<$t Iter>]}),+
//     };
// }
//
// expand_types!(A, B, C, D, E, F, G, H, I, J, K, L, M);

impl_iterator!(
    A,B,C,D,E,F,G,H,I,J,K,L,M;
    AA,BB,CC,DD,EE,FF,GG,HH,II,JJ,KK,LL,MM;
    AIter,BIter,CIter,DIter,EIter,FIter,GIter,HIter,IIter,JIter,KIter,LIter,MIter
);
