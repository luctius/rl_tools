macro_rules! impl_query_impl {
    ( $( $iter:ident<$($typename:ident),*>, )+ ) => {
        $(
            mod query_impl {
                use std::marker::PhantomData;
                use std::any::Any;

                use crate::bin_storage::{BinStorage, BinStoragePrivate,};
                use crate::id::{IdInt, IdGenerationType};
                use crate::rl_ecs::{RlEcs, RlEcsPrivate};

                #[allow(unused_imports)]
                use crate::query::{
                    Access,
                    QATarget,
                    QueryAtom,
                    QueryAtomRel,
                    Query,
                    QueryPrivate,
                    QueryRun,
                    QueryRunWithCommands,
                    QueryRunWorldMut,
                    QueryPurgeAny,
                    QueryPurgeAll,
                    QueryBuilder,
                    ReadToken,
                    WriteToken,
                    RunCommands,
                };

                pub struct QueryImpl<Target, $($typename,)*> (
                    pub(crate) Vec<(IdInt, $(< <$typename as Access>::Atom as QueryAtom>::IdIntOutput,)*)>,
                    pub(crate) Option<(IdGenerationType, $( replace_type!( $typename -> IdGenerationType), )* )>,
                    pub(crate) PhantomData<Target>,
                    $(pub(crate) PhantomData<$typename>),*
                )
                where
                    Target: Access,
                    <Target as Access>::Atom: QATarget,
                    $($typename: Access,)*
                    $(<$typename as Access>::Atom: QueryAtomRel,)*;

                impl<'a, Store, Target, $($typename,)* const SIZE: usize> QueryPrivate<'a, Store, {SIZE}> for QueryImpl<Target, $($typename,)*>
                where
                    Store: BinStorage +'a,
                    Target: Access,
                    <Target as Access>::Atom: QATarget,
                    $($typename: Access,)*
                    $(<$typename as Access>::Atom: QueryAtomRel,)*
                {
                    type Item = super::iterator::MatcherIterInt<'a, Store, <Target as Access>::Atom, $(<$typename as Access>::Atom,)* {SIZE}>;

                    #[inline(never)]
                    fn idint_iter(&self, world: &'a RlEcs<Store, {SIZE}>) -> Self::Item {
                        super::iterator::MatcherIterInt (
                            world,
                            <Target as Access>::Atom::iter(world),
                            PhantomData,
                            $( replace_expr!($typename -> PhantomData),)*
                        )
                    }
                }

                impl<'a, Store, Target, $($typename,)* const SIZE: usize> Query<'a, Store, {SIZE}> for QueryImpl<Target, $($typename,)*>
                where
                    Store: BinStorage + 'a,
                    Self: QueryPrivate<'a, Store, {SIZE}>,
                    <Self as QueryPrivate<'a, Store, {SIZE}> >::Item: Iterator<Item=(IdInt, $(< <$typename as Access>::Atom as QueryAtom>::IdIntOutput),* )>,
                    Target: Access,
                    <Target as Access>::Atom: QATarget,
                    $($typename: Access,)*
                    $(<$typename as Access>::Atom: QueryAtomRel,)*
                {
                    type Item = super::iterator::MatcherIter<'a, Store, <Target as Access>::Atom, $(<$typename as Access>::Atom,)* {SIZE}>;

                    #[inline(never)]
                    fn update(&mut self, ecs: &'a RlEcs<Store, { SIZE }>) {
                        let mut up_to_date = false;

                        #[allow(non_snake_case)]
                        if let Some( (target, $( $typename),* ) ) = self.1 {
                            if target == <Target as Access>::Atom::get_generation(ecs)
                                $( && $typename == < <$typename as Access>::Atom as QueryAtom>::get_generation(ecs) )* {
                                up_to_date = true;
                            }
                        }

                        if !up_to_date {
                            self.0 = self.idint_iter(ecs).collect();
                            self.1 = Some( (
                                <Target as Access>::Atom::get_generation(ecs),
                                $( <$typename as Access>::Atom::get_generation(ecs), )*
                            ) );
                        }
                    }
                }
                impl<Store, Target, $($typename,)* const SIZE: usize> QueryPurgeAll<Store, {SIZE}> for QueryImpl<Target, $($typename,)*>
                where
                    Store: BinStorage,
                    Target: Access<Token=WriteToken<IdInt>>,
                    <Target as Access>::Atom: QATarget,
                    $($typename: Access,)*
                    $(<$typename as Access>::Atom: QueryAtomRel,)*
                {
                    #[inline(never)]
                    fn purge_all(&self, ecs: &mut RlEcs<Store, {SIZE}>)
                    {
                        for target_id in <Target as Access>::Atom::iter(&ecs).rev() {
                            let target_id_int = target_id.to_id_internal_unchecked();
                            $( if <$typename as Access>::Atom::get_relative_id_int(&ecs, target_id_int).is_none() { continue; } )*
                            ecs.purge_int(target_id_int).expect("Internal Error: Id not accepted");
                        }
                    }
                }

                impl<Func, BINSTORE, Target, $($typename,)* const SIZE: usize> QueryPurgeAny<BINSTORE, Func, {SIZE}> for QueryImpl<Target, $($typename,)*>
                where
                    BINSTORE: BinStorage + BinStoragePrivate,
                    Func: Fn(
                        (Target::Token, &< <Target as Access>::Atom as QueryAtom>::Item),
                        $( ($typename::Token, &< <$typename as Access>::Atom as QueryAtom>::ComponentOutput) ),*
                        ) -> Option<WriteToken<IdInt> >,
                    Target: Access,
                    <Target as Access>::Atom: QATarget,
                    < <Target as Access>::Atom as QueryAtom>::IdIntOutput: From<IdInt>,
                    $($typename: Access,)*
                    $(<$typename as Access>::Atom: QueryAtomRel,)*
                {
                    #[inline(never)]
                    fn purge_any(&self, ecs: &mut RlEcs<BINSTORE, {SIZE}>, func: Func) {
                        for target_id in <Target as Access>::Atom::iter(&ecs).rev() {
                            let target_id_int = target_id.to_id_internal_unchecked();

                            let target_token = Target::to_token(target_id_int.into() );

                            $(
                                #[allow(non_snake_case)]
                                let $typename = match <$typename as Access>::Atom::get_relative_id_int(&ecs, target_id_int) {
                                    None => continue,
                                    Some(id) => id,
                                };
                            )*

                            let target: < <Target as Access>::Atom as QueryAtom>::Item = BinStoragePrivate::take(&mut ecs.bin, target_id_int).unwrap();
                            $(
                                #[allow(non_snake_case)]
                                let $typename = ($typename, $typename::to_token($typename), <$typename as Access>::Atom::take_bin(&mut ecs.bin, $typename) );
                            )*

                            let opt_del_id = func( (target_token, &target), $( ($typename.1, &$typename.2) ),*);
                            BinStoragePrivate::put(&mut ecs.bin, target_id_int, target);
                            $( <$typename as Access>::Atom::put_bin(&mut ecs.bin, $typename.0, $typename.2); )*

                            if let Some(del_id) = opt_del_id{
                                ecs.purge_int(del_id.0).expect("Id not accepted");
                            }
                        }
                    }
                }

                impl<Func, BINSTORE, Target, $($typename,)* const SIZE: usize> QueryRun<Func, BINSTORE, {SIZE}> for QueryImpl<Target, $($typename,)*>
                where
                    BINSTORE: BinStorage + BinStoragePrivate,
                    Func: FnMut(&mut <Target as Access>::RefOutput, $( &mut <$typename as Access>::RefOutput),*),
                    Target: Access,
                    <Target as Access>::Atom: QATarget,
                    < <Target as Access>::Atom as QueryAtom>::IdIntOutput: From<IdInt>,
                    $($typename: Access,)*
                    $(<$typename as Access>::Atom: QueryAtomRel,)*
                {
                    #[inline(never)]
                    fn run(&mut self, ecs: &mut RlEcs<BINSTORE, {SIZE}>, mut func: Func) {
                        self.update(ecs);

                        #[allow(non_snake_case)]
                        for (target_id, $( $typename,)* ) in &self.0 {
                            let mut target = Target::wrap(<Target as Access>::Atom::take_bin(&mut ecs.bin, (*target_id).into() ) );
                            $(
                                #[allow(non_snake_case)]
                                let mut $typename = ($typename, $typename::wrap(<$typename as Access>::Atom::take_bin(&mut ecs.bin, *$typename) ) );
                            )*

                            func(&mut target, $( &mut $typename.1 ),*);

                            BinStoragePrivate::put(&mut ecs.bin, *target_id, Target::unwrap(target) );
                            $( <$typename as Access>::Atom::put_bin(&mut ecs.bin, *$typename.0, $typename::unwrap($typename.1) ); )*
                        }
                    }
                }

                impl<TYPE, Func, BINSTORE, Target, $($typename,)* const SIZE: usize> QueryRunWithCommands<TYPE, Func, BINSTORE, {SIZE}> for QueryImpl<Target, $($typename,)*>
                where
                    BINSTORE: BinStorage + BinStoragePrivate,
                    TYPE: Any + 'static,
                    Func: FnMut(
                        (Target::Token, &mut <Target as Access>::RefOutput),
                        $( ($typename::Token, &mut <$typename as Access>::RefOutput) ),*
                        ) -> Option<RunCommands<TYPE> >,
                    Target: Access,
                    <Target as Access>::Atom: QATarget,
                    < <Target as Access>::Atom as QueryAtom>::IdIntOutput: From<IdInt>,
                    $($typename: Access,)*
                    $(<$typename as Access>::Atom: QueryAtomRel,)*
                {
                    #[inline(never)]
                    fn run_with_commands(&mut self, ecs: &mut RlEcs<BINSTORE, {SIZE}>, mut func: Func) {
                        self.update(ecs);

                        #[allow(non_snake_case)]
                        for (target_id, $( $typename,)* ) in &self.0 {

                            let target_token = Target::to_token((*target_id).into() );

                            let mut target = Target::wrap(<Target as Access>::Atom::take_bin(&mut ecs.bin, (*target_id).into() ) );
                            $(
                                #[allow(non_snake_case)]
                                let mut $typename = ($typename, $typename::to_token(*$typename), $typename::wrap(<$typename as Access>::Atom::take_bin(&mut ecs.bin, *$typename) ) );
                            )*

                            let command_opt = func( (target_token, &mut target), $( ($typename.1, &mut $typename.2) ),*);

                            BinStoragePrivate::put(&mut ecs.bin, *target_id, Target::unwrap(target) );
                            $( <$typename as Access>::Atom::put_bin(&mut ecs.bin, *$typename.0, $typename::unwrap($typename.2) ); )*

                            if let Some(command) = command_opt {
                                command.execute(ecs);
                            }
                        }
                    }
                }

                impl<Func, BINSTORE, Target, $($typename,)* const SIZE: usize> QueryRunWorldMut<Func, BINSTORE, {SIZE}> for QueryImpl<Target, $($typename,)*>
                where
                    BINSTORE: BinStorage + BinStoragePrivate,
                    Func: FnMut(&mut RlEcs<BINSTORE, {SIZE}>,
                        (< <Target as Access>::Atom as QueryAtom>::IdOutput, &mut <Target as Access>::RefOutput),
                        $( (< <$typename as Access>::Atom as QueryAtom>::IdOutput, &mut <$typename as Access>::RefOutput), )*
                        ),
                    Target: Access,
                    <Target as Access>::Atom: QATarget,
                    < <Target as Access>::Atom as QueryAtom>::IdIntOutput: From<IdInt>,
                    $($typename: Access,)*
                    $(<$typename as Access>::Atom: QueryAtomRel,)*
                {
                    #[inline(never)]
                    fn run_with_world_mut(&mut self, ecs: &mut RlEcs<BINSTORE, {SIZE}>, mut func: Func) {
                        self.update(ecs);

                        #[allow(non_snake_case)]
                        for (target_id, $( $typename,)* ) in &self.0 {
                            let mut target = ( < <Target as Access>::Atom as QueryAtom>::convert_id(ecs, (*target_id).into() ), Target::wrap(<Target as Access>::Atom::take_bin(&mut ecs.bin, (*target_id).into() ) ) );
                            $(
                                #[allow(non_snake_case)]
                                let mut $typename = ($typename, < <$typename as Access>::Atom as QueryAtom>::convert_id(&ecs, *$typename), $typename::wrap(<$typename as Access>::Atom::take_bin(&mut ecs.bin, *$typename) ) );
                            )*

                            func(ecs, (target.0, &mut target.1), $( ($typename.1, &mut $typename.2) ),*);

                            BinStoragePrivate::put(&mut ecs.bin, *target_id, Target::unwrap(target.1) );
                            $( <$typename as Access>::Atom::put_bin(&mut ecs.bin, *$typename.0, $typename::unwrap($typename.2) ); )*
                        }
                    }
                }
            }
        )+
    }
}
