pub mod access;
use access::Access;
pub use access::{Read, ReadToken, Write, WriteToken};

use std::{any::Any, marker::PhantomData};

use crate::{bin_storage::{BinStorage, BinStoragePrivate},
            id::{Id, IdGenerationType, IdIdx, IdInt},
            id_storage::{IdStorageIter, IdStoragePrivate},
            rl_ecs::{RlEcs, RlEcsPrivate}};

pub trait QueryAtom
    where Self::Item: Any + 'static,
          Self::ComponentOutput: Any + 'static,
          Self::IdIntOutput: Copy,
          Self::IdOutput: Copy, {
    type Item;

    type IdIntOutput;

    type IdOutput;
    fn convert_id<S: BinStorage, const N: usize>(world: &RlEcs<S, { N }>, id: Self::IdIntOutput) -> Self::IdOutput;

    type ComponentOutput;
    fn take_bin<S>(bins: &mut S, id: Self::IdIntOutput) -> Self::ComponentOutput
        where Self::Item: Any + 'static,
              S: BinStorage + BinStoragePrivate;

    fn put_bin<S>(bins: &mut S, id: Self::IdIntOutput, item: Self::ComponentOutput)
        where Self::Item: Any + 'static,
              S: BinStorage + BinStoragePrivate;

    fn get_generation<Store: BinStorage, const N: usize>(world: &RlEcs<Store, { N }>) -> IdGenerationType {
        let node = world.bin.fetch_node::<Self::Item>().unwrap();
        IdStoragePrivate::get_generation(&world.id, node.id)
    }
}

pub trait QATarget: QueryAtom
    where Self::Item: Any + 'static, {
    fn get_id_int<Store: BinStorage, const N: usize>(world: &RlEcs<Store, { N }>, idx: IdIdx) -> Option<IdInt>;
    fn iter<Store: BinStorage, const N: usize>(world: &RlEcs<Store, { N }>) -> IdStorageIter;
    fn len<Store: BinStorage, const N: usize>(world: &RlEcs<Store, { N }>) -> usize;
}

pub struct Target<ITEM>(PhantomData<ITEM>) where ITEM: Any + 'static;
impl<ITEM> QueryAtom for Target<ITEM> where ITEM: Any + 'static, {
    type ComponentOutput = ITEM;
    type IdIntOutput = IdInt;
    type IdOutput = Id;
    type Item = ITEM;

    #[inline]
    fn convert_id<S: BinStorage, const N: usize>(world: &RlEcs<S, { N }>, id: Self::IdIntOutput) -> Self::IdOutput {
        let gen = world.bin.fetch_node::<Self::Item>().unwrap().gen;
        id.to_id(gen)
    }

    #[inline]
    fn take_bin<S>(bins: &mut S, id: Self::IdIntOutput) -> Self::ComponentOutput
        where Self::Item: Any + 'static,
              S: BinStorage + BinStoragePrivate, {
        BinStoragePrivate::take(bins, id).unwrap()
    }

    #[inline]
    fn put_bin<S>(bins: &mut S, id: Self::IdIntOutput, item: Self::ComponentOutput)
        where Self::Item: Any + 'static,
              S: BinStorage + BinStoragePrivate, {
        BinStoragePrivate::put(bins, id, item)
    }
}
impl<ITEM> QATarget for Target<ITEM>
    where Self: QueryAtom,
          ITEM: Any + 'static,
{
    #[inline]
    fn get_id_int<Store: BinStorage, const N: usize>(world: &RlEcs<Store, { N }>, idx: IdIdx) -> Option<IdInt> {
        let node = world.bin.fetch_node::<Self::Item>().unwrap();
        Some(IdInt::new(node.id, idx))
    }

    #[inline]
    fn iter<Store: BinStorage, const N: usize>(world: &RlEcs<Store, { N }>) -> IdStorageIter { world.id_iter::<ITEM>() }

    #[inline]
    fn len<Store: BinStorage, const N: usize>(world: &RlEcs<Store, { N }>) -> usize {
        let node = world.bin.fetch_node::<Self::Item>().unwrap();
        IdStoragePrivate::len(&world.id, node.id)
    }
}

pub trait QueryAtomRel: QueryAtom {
    fn get_relative_id_int<S: BinStorage, const N: usize>(world: &RlEcs<S, { N }>,
                                                          target_id: IdInt)
                                                          -> Option<Self::IdIntOutput>;
}

pub struct GrandParent<ITEM>(PhantomData<ITEM>);
pub struct Parent<ITEM>(PhantomData<ITEM>);
pub struct Child<ITEM>(PhantomData<ITEM>);
pub struct Sibbling<ITEM>(PhantomData<ITEM>);
pub struct Not<S>(S) where S: QueryAtomRel;

impl<ITEM> QueryAtom for Parent<ITEM> where ITEM: Any + 'static, {
    type ComponentOutput = ITEM;
    type IdIntOutput = IdInt;
    type IdOutput = Id;
    type Item = ITEM;

    #[inline]
    fn convert_id<S: BinStorage, const N: usize>(world: &RlEcs<S, { N }>, id: Self::IdIntOutput) -> Self::IdOutput {
        let gen = world.bin.fetch_node::<Self::Item>().unwrap().gen;
        id.to_id(gen)
    }

    #[inline]
    fn take_bin<S>(bins: &mut S, id: Self::IdIntOutput) -> Self::ComponentOutput
        where Self::Item: Any + 'static,
              S: BinStorage + BinStoragePrivate, {
        BinStoragePrivate::take(bins, id).unwrap()
    }

    #[inline]
    fn put_bin<S>(bins: &mut S, id: Self::IdIntOutput, item: Self::ComponentOutput)
        where Self::Item: Any + 'static,
              S: BinStorage + BinStoragePrivate, {
        BinStoragePrivate::put(bins, id, item)
    }
}
impl<ITEM> QueryAtomRel for Parent<ITEM> where ITEM: Any + 'static, {
    #[inline]
    fn get_relative_id_int<S: BinStorage, const N: usize>(world: &RlEcs<S, { N }>,
                                                          target_id: IdInt)
                                                          -> Option<Self::IdIntOutput>
    {
        IdStoragePrivate::get_parent_id(&world.id, target_id).unwrap()
    }
}

impl<ITEM> QueryAtom for Child<ITEM> where ITEM: Any + 'static, {
    type ComponentOutput = ITEM;
    type IdIntOutput = IdInt;
    type IdOutput = Id;
    type Item = ITEM;

    #[inline]
    fn convert_id<S: BinStorage, const N: usize>(world: &RlEcs<S, { N }>, id: Self::IdIntOutput) -> Self::IdOutput {
        let gen = world.bin.fetch_node::<Self::Item>().unwrap().gen;
        id.to_id(gen)
    }

    #[inline]
    fn take_bin<S>(bins: &mut S, id: Self::IdIntOutput) -> Self::ComponentOutput
        where Self::Item: Any + 'static,
              S: BinStorage + BinStoragePrivate, {
        BinStoragePrivate::take(bins, id).unwrap()
    }

    #[inline]
    fn put_bin<S>(bins: &mut S, id: Self::IdIntOutput, item: Self::ComponentOutput)
        where Self::Item: Any + 'static,
              S: BinStorage + BinStoragePrivate, {
        BinStoragePrivate::put(bins, id, item)
    }
}
impl<ITEM> QueryAtomRel for Child<ITEM> where ITEM: Any + 'static, {
    #[inline]
    fn get_relative_id_int<S: BinStorage, const N: usize>(world: &RlEcs<S, { N }>,
                                                          target_id: IdInt)
                                                          -> Option<Self::IdIntOutput>
    {
        world.get_child_id_int::<Self::Item>(target_id).unwrap().map(|id| id.to_id_internal_unchecked())
    }
}

impl<ITEM> QueryAtom for Sibbling<ITEM> where ITEM: Any + 'static, {
    type ComponentOutput = ITEM;
    type IdIntOutput = IdInt;
    type IdOutput = Id;
    type Item = ITEM;

    #[inline]
    fn convert_id<S: BinStorage, const N: usize>(world: &RlEcs<S, { N }>, id: Self::IdIntOutput) -> Self::IdOutput {
        let gen = world.bin.fetch_node::<Self::Item>().unwrap().gen;
        id.to_id(gen)
    }

    #[inline]
    fn take_bin<S>(bins: &mut S, id: Self::IdIntOutput) -> Self::ComponentOutput
        where Self::Item: Any + 'static,
              S: BinStorage + BinStoragePrivate, {
        BinStoragePrivate::take(bins, id).unwrap()
    }

    #[inline]
    fn put_bin<S>(bins: &mut S, id: Self::IdIntOutput, item: Self::ComponentOutput)
        where Self::Item: Any + 'static,
              S: BinStorage + BinStoragePrivate, {
        BinStoragePrivate::put(bins, id, item)
    }
}
impl<ITEM> QueryAtomRel for Sibbling<ITEM> where ITEM: Any + 'static, {
    #[inline]
    fn get_relative_id_int<S: BinStorage, const N: usize>(world: &RlEcs<S, { N }>,
                                                          target_id: IdInt)
                                                          -> Option<Self::IdIntOutput>
    {
        if let Ok(Some(pid)) = IdStoragePrivate::get_parent_id(&world.id, target_id) {
            world.get_child_id_int::<Self::Item>(pid).unwrap().map(|id| id.to_id_internal_unchecked())
        } else {
            None
        }
    }
}

impl<ITEM> QueryAtom for GrandParent<ITEM> where ITEM: Any + 'static, {
    type ComponentOutput = ITEM;
    type IdIntOutput = IdInt;
    type IdOutput = Id;
    type Item = ITEM;

    #[inline]
    fn convert_id<S: BinStorage, const N: usize>(world: &RlEcs<S, { N }>, id: Self::IdIntOutput) -> Self::IdOutput {
        let gen = world.bin.fetch_node::<Self::Item>().unwrap().gen;
        id.to_id(gen)
    }

    #[inline]
    fn take_bin<S>(bins: &mut S, id: Self::IdIntOutput) -> Self::ComponentOutput
        where Self::Item: Any + 'static,
              S: BinStorage + BinStoragePrivate, {
        BinStoragePrivate::take(bins, id).unwrap()
    }

    #[inline]
    fn put_bin<S>(bins: &mut S, id: Self::IdIntOutput, item: Self::ComponentOutput)
        where Self::Item: Any + 'static,
              S: BinStorage + BinStoragePrivate, {
        BinStoragePrivate::put(bins, id, item)
    }
}
impl<ITEM> QueryAtomRel for GrandParent<ITEM> where ITEM: Any + 'static, {
    #[inline]
    fn get_relative_id_int<S: BinStorage, const N: usize>(world: &RlEcs<S, { N }>,
                                                          target_id: IdInt)
                                                          -> Option<Self::IdIntOutput>
    {
        if let Ok(Some(pid)) = IdStoragePrivate::get_parent_id(&world.id, target_id) {
            let typeid = world.bin.get_type_id::<Self::Item>();

            let mut test_gpid = pid;
            let mut ret_pid = None;
            while let Ok(Some(gpid)) = IdStoragePrivate::get_parent_id(&world.id, test_gpid) {
                test_gpid = gpid;
                if gpid.typeidx == typeid {
                    ret_pid = Some(gpid);
                }
            }

            if test_gpid != pid {
                return ret_pid;
            }
        }
        None
    }
}

impl<ITEM> QueryAtom for Not<ITEM> where ITEM: QueryAtom + QueryAtomRel, {
    type ComponentOutput = ();
    type IdIntOutput = ();
    type IdOutput = ();
    type Item = <ITEM as QueryAtom>::Item;

    #[inline]
    fn convert_id<S: BinStorage, const N: usize>(_: &RlEcs<S, { N }>, _: Self::IdIntOutput) -> Self::IdOutput {}

    #[inline]
    fn take_bin<S>(_: &mut S, id: Self::IdIntOutput) -> Self::ComponentOutput
        where Self::Item: Any + 'static,
              S: BinStorage + BinStoragePrivate, {
        id
    }

    #[inline]
    fn put_bin<S>(_: &mut S, _: Self::IdIntOutput, _: Self::ComponentOutput)
        where Self::Item: Any + 'static,
              S: BinStorage + BinStoragePrivate, {
    }
}
impl<ITEM> QueryAtomRel for Not<ITEM> where ITEM: QueryAtom + QueryAtomRel + Any + 'static, {
    #[inline]
    fn get_relative_id_int<S: BinStorage, const N: usize>(world: &RlEcs<S, { N }>,
                                                          target_id: IdInt)
                                                          -> Option<Self::IdIntOutput>
    {
        if ITEM::get_relative_id_int(&world, target_id).is_some() { None } else { Some(()) }
    }
}

pub trait QueryBuilder {
    type Output;
    fn build_query() -> Self::Output;
}

pub trait QueryPrivate<'a, Store, const N: usize>
    where Store: BinStorage + BinStoragePrivate,
          Self::Item: Iterator, {
    type Item;

    fn idint_iter(&self, ecs: &'a RlEcs<Store, { N }>) -> Self::Item;
}

pub trait Query<'a, Store, const N: usize>
    where Store: BinStorage + BinStoragePrivate,
          Self::Item: Iterator, {
    type Item;

    fn update(&mut self, ecs: &'a RlEcs<Store, { N }>);
}

pub trait QueryPurgeAny<BINSTORE, Func, const N: usize>
    where BINSTORE: BinStorage + BinStoragePrivate, {
    fn purge_any(&self, ecs: &mut RlEcs<BINSTORE, { N }>, func: Func);
}

pub trait QueryPurgeAll<BINSTORE, const N: usize>
    where BINSTORE: BinStorage + BinStoragePrivate, {
    fn purge_all(&self, ecs: &mut RlEcs<BINSTORE, { N }>);
}

pub trait QueryRun<Func, BINSTORE, const N: usize>
    where BINSTORE: BinStorage + BinStoragePrivate, {
    fn run(&mut self, ecs: &mut RlEcs<BINSTORE, { N }>, func: Func);
}

pub trait QueryRunWithCommands<T, Func, BINSTORE, const N: usize>
    where BINSTORE: BinStorage + BinStoragePrivate, {
    fn run_with_commands(&mut self, ecs: &mut RlEcs<BINSTORE, { N }>, func: Func);
}

pub trait QueryRunWorldMut<Func, BINSTORE, const N: usize>
    where BINSTORE: BinStorage + BinStoragePrivate, {
    fn run_with_world_mut(&mut self, ecs: &mut RlEcs<BINSTORE, { N }>, func: Func);
}

pub enum RunCommands<T>
    where T: Any + 'static, {
    CreateAndAttach(T, WriteToken<IdInt>),
    Detach(WriteToken<IdInt>),
}
impl<T> RunCommands<T> {
    pub(crate) fn execute<BINSTORE, const N: usize>(self, world: &mut RlEcs<BINSTORE, { N }>)
        where BINSTORE: BinStorage, {
        match self {
            RunCommands::CreateAndAttach(typ, token) => {
                let new_id = world.create(typ);
                IdStoragePrivate::attach(&mut world.id, new_id.to_id_internal_unchecked(), token.0).unwrap();
            },
            RunCommands::Detach(token) => {
                IdStoragePrivate::detach(&mut world.id, token.0).unwrap();
            },
        }
    }
}

#[macro_use]
mod builders;
#[macro_use]
mod iterators;
#[macro_use]
mod query_impl;

macro_rules! impl_matcher {
    ( $( $iter:ident<$($typename:ident),*>, )+ ) => {
        $(
            pub mod $iter {
                impl_builders!($iter<$($typename),*>,);
                impl_query_iterators!($iter<$($typename),*>,);
                impl_query_impl!($iter<$($typename),*>,);
            }
        )+
    }
}

pub mod query_impls {
    #[rustfmt::skip]
    impl_matcher!(
        query0 <>,
        query1 <T0>,
        query2 <T0, T1>,
        query3 <T0, T1, T2>,
        query4 <T0, T1, T2, T3>,
        query5 <T0, T1, T2, T3, T4>,
        query6 <T0, T1, T2, T3, T4, T5>,
        query7 <T0, T1, T2, T3, T4, T5, T6>,
        query8 <T0, T1, T2, T3, T4, T5, T6, T7>,
        query9 <T0, T1, T2, T3, T4, T5, T6, T7, T8>,
        query10<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9>,
        query11<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10>,
        query12<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11>,
        query13<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12>,
        query14<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13>,
        query15<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14>,
        query16<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15>,
        query17<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16>,
        query18<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17>,
        query19<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18>,
        query20<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19>,
        query21<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20>,
        query22<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21>,
        query23<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21, T22>,
        query24<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21, T22, T23>,
        query25<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21, T22, T23, T24>,
        query26<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21, T22, T23, T24, T25>,
        query27<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21, T22, T23, T24, T25, T26>,
        query28<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21, T22, T23, T24, T25, T26, T27>,
        query29<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21, T22, T23, T24, T25, T26, T27, T28>,
        query30<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21, T22, T23, T24, T25, T26, T27, T28, T29>,
        query31<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21, T22, T23, T24, T25, T26, T27, T28, T29, T30>,
        query32<T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16, T17, T18, T19, T20, T21, T22, T23, T24, T25, T26, T27, T28, T29, T30, T31>,
    );
}
#[cfg(test)]
mod tests {
    use crate::{entity_relation::EntityRelationType,
                query::{Child, GrandParent, Parent, QueryBuilder, QueryRun, Read, Sibbling, Target, Write},
                rl_ecs::RlEcsPrivate,
                stores::{Builder, FinishBuilding as _}};

    #[test]
    fn run() {
        #[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
        struct TEntity {
            i: usize,
        }

        let mut ecs = Builder::new().add_component::<TEntity>().finalize();

        // create 2 entities
        let te0 = ecs.create(TEntity { i: 0, });
        let te1 = ecs.create(TEntity { i: 1, });
        assert!(ecs.contains(te0).unwrap());
        assert!(ecs.contains(te1).unwrap());

        let mut count = 0;
        <(Read<Target<TEntity>>,)>::build_query().run(&mut ecs, |target| {
                                                     assert_eq!(target.i, count);
                                                     count += 1;
                                                 });
        assert_eq!(count, 2);
    }

    #[test]
    #[should_panic(expected = "Duplicate Types detected in build_query!")]
    fn run_same_comp_twice() {
        #[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
        struct T1 {}
        #[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
        struct T2 {}

        let mut ecs = Builder::new().add_component::<T1>().add_component::<T2>().finalize();

        // create 2 entities
        let pid = ecs.create(T1 {});
        let _ = ecs.create(T2 {});
        ecs.create_and_attach(T2 {}, pid).unwrap();

        <(Read<Target<T1>>, Read<Child<T2>>, Read<Child<T2>>)>::build_query().run(&mut ecs, |_, _, _| {});
    }
    #[test]
    fn matcher_sibling() {
        use crate::bin_storage::BinStorage;

        #[derive(Clone, PartialEq, Eq, Debug)]
        pub struct TEntityBool {
            boolean: bool,
        }
        #[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
        struct TFlagBool {
            boolean: bool,
        }
        #[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
        struct TUChildren {
            x: usize,
        }

        let mut ecs = Builder::new().add_component::<TEntityBool>()
                                    .add_component::<TFlagBool>()
                                    .add_component::<TUChildren>()
                                    .finalize();

        let max = 10;
        let div = 2;
        for i in 0..max {
            let e = ecs.create(TEntityBool { boolean: false, });
            if i % div == 0 {
                ecs.create_and_attach(TFlagBool { boolean: true, }, e).unwrap();
                ecs.create_and_attach(TUChildren { x: 1, }, e).unwrap();
            }
        }
        <(Read<Target<TFlagBool> >, Write<Parent<TEntityBool>>, Write<Sibbling<TUChildren>>)>::build_query().run(&mut ecs, |tfb, teb, tuc| {
            if tfb.boolean {
                tuc.x = 2;
            }
            teb.boolean = tfb.boolean;
        });

        for id in ecs.id_iter::<TUChildren>() {
            assert_eq!(ecs.bin.get::<TUChildren>(id).unwrap().unwrap().x, 2);
        }
    }

    #[test]
    fn matcher_grandparent() {
        use crate::bin_storage::BinStorage;

        #[derive(Clone, PartialEq, Eq, Debug)]
        pub struct TEntityBool {
            boolean: bool,
        }
        #[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
        struct TFlagBool {
            boolean: bool,
        }
        #[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
        struct TUChildren1 {
            x: usize,
        }
        #[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
        struct TUChildren2 {
            x: usize,
        }

        let mut ecs = Builder::new().add_component::<TEntityBool>()
                                    .add_component::<TFlagBool>()
                                    .add_component_with_type::<TUChildren1>(EntityRelationType::UniqueChildren)
                                    .add_component_with_type::<TUChildren2>(EntityRelationType::UniqueChildren)
                                    .finalize();

        let max = 10;
        for i in 0..max {
            if i % 3 == 0 {
                let g = ecs.create(TEntityBool { boolean: false, });
                ecs.create_and_attach(TFlagBool { boolean: true, }, g).unwrap();
            } else if i % 2 == 0 {
                let g = ecs.create(TEntityBool { boolean: false, });
                let f = ecs.create_and_attach(TUChildren1 { x: 1, }, g).unwrap();
                ecs.create_and_attach(TFlagBool { boolean: true, }, f).unwrap();
            } else {
                let g = ecs.create(TUChildren1 { x: 2, });
                let f = ecs.create_and_attach(TUChildren2 { x: 1, }, g).unwrap();
                ecs.create_and_attach(TFlagBool { boolean: true, }, f).unwrap();
            }
        }

        let mut count = 0;
        <(Read<Target<TFlagBool> >, Read<Parent<TUChildren1>>, Write<GrandParent<TEntityBool>>)>::build_query().run(&mut ecs, |_target, _parent, grand_parent| {
                grand_parent.boolean = true;
                count += 1;
            });
        assert_eq!(count, 3);

        for id in ecs.id_iter::<TEntityBool>() {
            let b = ecs.bin.get::<TEntityBool>(id).unwrap().unwrap().boolean;
            let mut test = false;
            if ecs.get_child_id::<TUChildren1>(id).unwrap().is_some() {
                test = true;
            }
            assert_eq!(b, test);
        }
    }
}
