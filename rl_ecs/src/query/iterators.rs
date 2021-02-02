macro_rules! impl_query_iterators {
    ( $( $iter:ident<$($typename:ident),*>, )+ ) => {
        $(
            pub mod iterator {
                use std::marker::PhantomData;

                use crate::id_storage::{IdStorageIter, };
                use crate::bin_storage::{BinStorage,};
                use crate::id::{IdInt, Id};
                use crate::rl_ecs::{RlEcs};

                #[allow(unused_imports)]
                use crate::query::{
                    QATarget,
                    QueryAtom,
                    QueryAtomRel,
                    Query,
                    QueryPrivate,
                    QueryRun,
                    QueryPurgeAny,
                    QueryBuilder,
                };

                #[derive(Clone, Debug)]
                pub struct MatcherIterInt<'a, Store, Target, $($typename,)* const SIZE: usize>
                (
                    pub(crate) &'a RlEcs<Store, {SIZE}>,
                    pub(crate) IdStorageIter,
                    pub(crate) PhantomData<Target>,
                    $(pub(crate) PhantomData<$typename>),*
                )
                where
                    Store: BinStorage,
                    Target: QATarget,
                    $($typename: QueryAtom + QueryAtomRel,)*;

                impl<'a, Store, Target, $($typename,)* const SIZE: usize> Iterator for MatcherIterInt<'a, Store, Target, $($typename,)* {SIZE}>
                where
                    Store: BinStorage,
                    Target: QATarget,
                    $($typename: QueryAtom + QueryAtomRel,)*
                {
                    type Item = (IdInt, $($typename::IdIntOutput),* );

                    fn next(&mut self) -> Option<Self::Item> {
                        #[allow(clippy::never_loop)]
                        while let Some(target_id) = self.1.next() {
                            let target_id_int = target_id.to_id_internal_unchecked();
                            //For Matcher0 this never loops.
                            return Some( ( target_id_int,
                                $(match $typename::get_relative_id_int(&self.0, target_id_int) {
                                    None => continue,
                                    Some(id) => id,
                                },)* )
                            );
                        }
                        None
                    }

                    fn size_hint(&self) -> (usize, Option<usize>) {
                        self.1.size_hint()
                    }
                }

                #[derive(Clone, Debug)]
                pub struct MatcherIter<'a, Store, Target, $($typename,)* const SIZE: usize>
                (
                    pub(crate) &'a RlEcs<Store, {SIZE}>,
                    pub(crate) IdStorageIter,
                    pub(crate) PhantomData<Target>,
                    $(pub(crate) PhantomData<$typename>),*
                )
                where
                    Store: BinStorage,
                    Target: QATarget,
                    $($typename: QueryAtom + QueryAtomRel,)*;

                impl<'a, Store, Target, $($typename,)* const SIZE: usize> Iterator for MatcherIter<'a, Store, Target, $($typename,)* {SIZE}>
                where
                    Store: BinStorage,
                    Target: QATarget,
                    $($typename: QueryAtom + QueryAtomRel,)*
                {
                    type Item = (Id, $($typename::IdOutput),* );

                    fn next(&mut self) -> Option<Self::Item> {
                        #[allow(clippy::never_loop)]
                        while let Some(target_id) = self.1.next() {
                            let _target_id_int = target_id.to_id_internal_unchecked();
                            //For Matcher0 this never loops.
                            $(
                                #[allow(non_snake_case)]
                                let $typename = match $typename::get_relative_id_int(&self.0, _target_id_int) {
                                    None => continue,
                                    Some(id) => id,
                                };
                            )*
                            $(
                                #[allow(non_snake_case)]
                                let $typename = $typename::convert_id(&self.0, $typename);
                            )*
                            return Some( (target_id, $($typename,)* ) );
                        }
                        None
                    }

                    fn size_hint(&self) -> (usize, Option<usize>) {
                        self.1.size_hint()
                    }
                }
            }
        )+
    }
}
