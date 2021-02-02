macro_rules! impl_builders {
    ( $( $iter:ident<$($typename:ident),*>, )+ ) => {
        $(
            mod builders {
                use std::marker::PhantomData;
                use std::any::TypeId;

                #[allow(unused_imports)]
                use crate::query::{
                    Access,
                    QATarget,
                    QueryAtom,
                    QueryAtomRel,
                    Query,
                    QueryPrivate,
                    QueryRun,
                    QueryPurgeAny,
                    QueryBuilder,
                };
                impl<Target, $($typename,)*> QueryBuilder for (Target, $($typename,)*)
                where
                    Target: Access,
                    <Target as Access>::Atom: QATarget,
                    $($typename: Access,)*
                    $(<$typename as Access>::Atom: QueryAtomRel,)*
                {
                    type Output = super::query_impl::QueryImpl<Target, $($typename,)*>;

                    #[inline(never)]
                    fn build_query() -> Self::Output {
                        const CNT: usize = 1 $( +  replace_expr!($typename -> 1) )*;
                        let ids: [TypeId; CNT] = [
                            TypeId::of::< < <Target as Access>::Atom as QueryAtom>::Item>(),
                            $(TypeId::of::< < <$typename as Access>::Atom as QueryAtom>::Item>() ),*
                        ];
                        if (1..ids.len()).any(|i| ids[i..].contains(&ids[i - 1])) {
                            panic!("Duplicate Types detected in build_query!");
                        }

                        super::query_impl::QueryImpl (
                            Vec::new(),
                            None,
                            PhantomData,
                            $( replace_expr!($typename -> PhantomData),)*
                        )
                    }
                }
            }
        )+
    }
}
