use std::any::Any;

use crate::{
    bin_storage::{BinStorage, BinStoragePrivate},
    id::{Id, IdInt},
    id_storage::{IdStorage, IdStorageIter, IdStoragePrivate},
    stores::FetchResources,
    Error,
};

pub trait RlEcsPrivate<BINSTORE>
where
    BINSTORE: BinStorage,
{
    fn get_child_id_int<ITEM>(&self, parent_id: IdInt) -> Result<Option<Id>, Error>
    where
        ITEM: Any + 'static,
        BINSTORE: BinStoragePrivate;
    fn purge_int(&mut self, id: IdInt) -> Result<(), Error>
    where
        BINSTORE: BinStoragePrivate;
    fn purge(&mut self, id: Id) -> Result<(), Error>
    where
        BINSTORE: BinStoragePrivate;
    fn id_iter<ITEM>(&self) -> IdStorageIter
    where
        ITEM: Any + 'static;
}

#[derive(Clone, Debug)]
pub struct RlEcs<BINSTORE, const N: usize>
where
    BINSTORE: BinStorage,
{
    pub id: IdStorage<{ N }>,
    pub bin: BINSTORE,
}
impl<BINSTORE, const N: usize> RlEcs<BINSTORE, { N }>
where
    BINSTORE: BinStorage + BinStoragePrivate,
{
    pub fn create<ITEM>(&mut self, item: ITEM) -> Id
    where
        ITEM: Any + 'static,
    {
        let typeid = self.bin.get_type_id::<ITEM>();
        let id = self.id.add(typeid);
        BinStoragePrivate::add(&mut self.bin, item);
        id
    }

    pub fn len<ITEM>(&self) -> usize
    where
        ITEM: Any + 'static,
    {
        self.bin.len::<ITEM>()
    }

    pub fn is_empty<ITEM>(&self) -> bool
    where
        ITEM: Any + 'static,
    {
        self.bin.is_empty::<ITEM>()
    }

    pub fn reserve<ITEM>(&mut self, sz: usize)
    where
        ITEM: Any + 'static,
    {
        let typeid = self.bin.get_type_id::<ITEM>();
        self.id.reserve(typeid, sz);
        self.bin.reserve::<ITEM>(sz);
    }

    pub fn contains(&mut self, id: Id) -> Result<bool, Error> {
        self.id.contains(id)
    }

    pub fn attach(&mut self, child_id: Id, parent_id: Id) -> Result<(), Error> {
        self.id.attach(child_id, parent_id)
    }

    pub fn detach(&mut self, child_id: Id) -> Result<(), Error> {
        self.id.detach(child_id)
    }

    pub fn get_child_id<ITEM>(&self, parent_id: Id) -> Result<Option<Id>, Error>
    where
        ITEM: Any + 'static,
    {
        let gen = self.id.get_generation(parent_id.typeidx);
        let id = parent_id.to_id_internal(gen)?;

        self.get_child_id_int::<ITEM>(id)
    }

    pub fn get_parent_id(&mut self, child_id: Id) -> Result<Option<Id>, Error> {
        self.id.get_parent_id(child_id)
    }

    pub fn get_absolute_parent_id(&mut self, child_id: Id) -> Result<Id, Error> {
        self.id.get_absolute_parent_id(child_id)
    }

    pub fn create_and_attach<ITEM>(&mut self, item: ITEM, parent_id: Id) -> Result<Id, Error>
    where
        ITEM: Any + 'static,
    {
        if !self.contains(parent_id).unwrap_or(false) {
            return Err(Error::NoSuchId);
        }
        if self
            .get_child_id::<ITEM>(parent_id)
            .unwrap_or(None)
            .is_some()
        {
            return Err(Error::ChildSlotFull);
        }

        let id = self.create(item);
        self.attach(id, parent_id)?;

        Ok(id)
    }

    pub fn get<ITEM>(&mut self, id: Id) -> Result<Option<&ITEM>, Error>
    where
        ITEM: Any + 'static,
    {
        let typeid = self.bin.get_type_id::<ITEM>();
        if id.typeidx == typeid {
            BinStorage::get(&self.bin, id)
        } else if let Ok(Some(cid)) = self.get_child_id::<ITEM>(id) {
            BinStorage::get(&self.bin, cid)
        } else if let Ok(Some(pid)) = self.get_parent_id(id) {
            BinStorage::get(&self.bin, pid)
        } else {
            Err(Error::WrongItemType)
        }
    }

    pub fn get_mut<ITEM>(&mut self, id: Id) -> Result<Option<&mut ITEM>, Error>
    where
        ITEM: Any + 'static,
    {
        let typeid = self.bin.get_type_id::<ITEM>();
        if id.typeidx == typeid {
            BinStorage::get_mut(&mut self.bin, id)
        } else if let Ok(Some(cid)) = self.get_child_id::<ITEM>(id) {
            BinStorage::get_mut(&mut self.bin, cid)
        } else if let Ok(Some(pid)) = self.get_parent_id(id) {
            BinStorage::get_mut(&mut self.bin, pid)
        } else {
            Err(Error::WrongItemType)
        }
    }
}
impl<BINSTORE, const N: usize> RlEcs<BINSTORE, { N }>
where
    BINSTORE: BinStorage + FetchResources,
{
    pub fn fetch_resource<'a>(&'a self) -> &'a <BINSTORE as FetchResources>::Output {
        self.bin.fetch_resource()
    }

    pub fn fetch_resource_mut<'a>(&'a mut self) -> &'a mut <BINSTORE as FetchResources>::Output {
        self.bin.fetch_resource_mut()
    }
}
impl<BINSTORE, const N: usize> RlEcsPrivate<BINSTORE> for RlEcs<BINSTORE, { N }>
where
    BINSTORE: BinStorage + BinStoragePrivate,
{
    fn get_child_id_int<ITEM>(&self, id: IdInt) -> Result<Option<Id>, Error>
    where
        ITEM: Any + 'static,
    {
        let node = self
            .bin
            .fetch_node::<ITEM>()
            .unwrap_or_else(|| panic!("Type of {} not registered!", std::any::type_name::<ITEM>()));
        let idtype = node.id;

        let child_gen = self.id.get_generation(idtype);
        IdStoragePrivate::get_child_id(&self.id, id, idtype)
            .map(|cido| cido.map(|cid| cid.to_id(child_gen)))
    }

    fn purge_int(&mut self, id: IdInt) -> Result<(), Error> {
        if !IdStoragePrivate::contains(&self.id, id) {
            return Err(Error::NoSuchId);
        }

        if let Err(e) = IdStoragePrivate::detach(&mut self.id, id) {
            if e != Error::EntityNoParent {
                return Err(e);
            }
        }

        loop {
            // remove children. We do this in a loop, since a MultipleChildren Entity can
            // have multiple of the same type.
            let cid_opt = if let Some(mut iter) = IdStoragePrivate::child_iter_int(&self.id, id) {
                iter.next()
            } else {
                None
            };
            if let Some(cid) = cid_opt {
                self.purge_int(cid.to_id_internal_unchecked())?;
                continue;
            }
            break;
        }

        let swapped_id = self.bin.swap_remove_by_id(id)?;
        assert_eq!(self.id.swap_remove(id)?, swapped_id);

        Ok(())
    }

    fn purge(&mut self, id: Id) -> Result<(), Error> {
        let gen = self.id.get_generation(id.typeidx);
        self.purge_int(id.to_id_internal(gen)?)
    }

    fn id_iter<ITEM>(&self) -> IdStorageIter
    where
        ITEM: Any + 'static,
    {
        let node = self
            .bin
            .fetch_node::<ITEM>()
            .unwrap_or_else(|| panic!("Type of {} not registered!", std::any::type_name::<ITEM>()));

        IdStoragePrivate::iter(&self.id, node.id)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        entity_relation::EntityRelationType,
        id::{Id, IdIdx, IdType},
        id_storage::IdStoragePrivate,
        query::{Child, Not, QueryBuilder, QueryPurgeAll, QueryRun, Read, Target, Write},
        rl_ecs::RlEcsPrivate,
        stores::{Builder, FinishBuilding as _},
        Error,
    };

    #[test]
    fn create() {
        struct TEntity {}
        struct TFlag {}
        struct TMChildren {}

        let tentity_id0 = Id::new(IdType::from(0), IdIdx(0), 0);
        let tflag_id0 = Id::new(IdType::from(0), IdIdx(0), 0);
        let tflag_id1 = Id::new(IdType::from(0), IdIdx(1), 0);
        let tmchildren_id0 = Id::new(IdType::from(1), IdIdx(0), 0);
        let tmchildren_id1 = Id::new(IdType::from(1), IdIdx(1), 0);

        let mut ecs = Builder::new().add_component::<TEntity>().finalize();

        // create te0 and check id
        assert_eq!(ecs.create(TEntity {}), tentity_id0);

        // restart, create te2, check id, then create a te0 and te1
        let mut ecs = Builder::new()
            .add_component::<TFlag>()
            .add_component::<TMChildren>()
            .finalize();

        assert_eq!(ecs.create(TFlag {}), tflag_id0);
        assert_eq!(ecs.create(TFlag {}), tflag_id1);
        assert_eq!(ecs.create(TMChildren {}), tmchildren_id0);
        assert_eq!(ecs.create(TMChildren {}), tmchildren_id1);
    }

    #[test]
    fn get() {
        #[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
        struct TEntity {}

        let tentity_id0 = Id::new(IdType::from(0), IdIdx(0), 0);
        let tentity_id1 = Id::new(IdType::from(0), IdIdx(1), 0);

        let mut ecs = Builder::new().add_component::<TEntity>().finalize();

        // create te0 and check id
        assert_eq!(ecs.create(TEntity {}), tentity_id0);
        assert_eq!(ecs.get::<TEntity>(tentity_id0), Ok(Some(&TEntity {})));

        // check that only id0 is created
        assert!(ecs.get::<TEntity>(tentity_id1).unwrap().is_none());
    }

    #[test]
    fn attach() {
        #[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
        struct TEntity {}
        #[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
        struct TMChildren {}

        let tentity_id0 = Id::new(IdType::from(0), IdIdx(0), 0);
        let tmchildren_id0 = Id::new(IdType::from(1), IdIdx(0), 0);

        let mut ecs = Builder::new()
            .add_component::<TEntity>()
            .add_component::<TMChildren>()
            .finalize();

        // create te0 and check id
        assert_eq!(ecs.create(TEntity {}), tentity_id0);
        assert_eq!(
            ecs.create_and_attach(TMChildren {}, tentity_id0),
            Ok(tmchildren_id0)
        );

        // check that all connections are valid
        assert_eq!(ecs.contains(tentity_id0), Ok(true));
        assert_eq!(
            ecs.get_child_id::<TMChildren>(tentity_id0),
            Ok(Some(tmchildren_id0))
        );
        assert_eq!(
            ecs.get::<TMChildren>(tmchildren_id0),
            Ok(Some(&TMChildren {}))
        );
        assert_eq!(ecs.get::<TMChildren>(tentity_id0), Ok(Some(&TMChildren {})));
    }

    #[test]
    #[should_panic(expected = "Endless chain detected!")]
    fn detect_loop() {
        // create an endless loop and check that the ecs will panic

        struct TUChildren1 {}
        struct TUChildren2 {}
        struct TUChildren3 {}
        let c1 = Id::new(IdType::from(0), IdIdx(0), 0);
        let c2 = Id::new(IdType::from(1), IdIdx(0), 0);
        let c3 = Id::new(IdType::from(2), IdIdx(0), 0);

        let mut ecs = Builder::new()
            .add_component_with_type::<TUChildren1>(EntityRelationType::UniqueChildren)
            .add_component_with_type::<TUChildren2>(EntityRelationType::UniqueChildren)
            .add_component_with_type::<TUChildren3>(EntityRelationType::UniqueChildren)
            .finalize();

        assert_eq!(ecs.create(TUChildren1 {}), c1);
        assert_eq!(ecs.create(TUChildren2 {}), c2);
        assert_eq!(ecs.create(TUChildren3 {}), c3);

        assert!(ecs.id.contains(c1).unwrap());
        assert!(ecs.id.contains(c2).unwrap());
        assert!(ecs.id.contains(c3).unwrap());

        assert!(ecs.id.attach(c2, c1).is_ok());
        assert!(ecs.id.attach(c3, c2).is_ok());
        assert!(ecs.id.attach(c1, c3).is_ok()); //Should Fail
    }

    #[test]
    fn create_and_attach() {
        #[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
        struct TEntity {}
        #[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
        struct TFlag {}
        #[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
        struct TUChildren {}

        let tentity_id0 = Id::new(IdType::from(0), IdIdx(0), 0);
        let tflag_id0 = Id::new(IdType::from(1), IdIdx(0), 0);
        let tuchildren_id0 = Id::new(IdType::from(2), IdIdx(0), 0);

        let mut ecs = Builder::new()
            .add_component::<TEntity>()
            .add_component::<TFlag>()
            .add_component_with_type::<TUChildren>(EntityRelationType::UniqueChildren)
            .finalize();

        // create a tree and check that all connections are valid
        assert_eq!(ecs.create(TEntity {}), tentity_id0);
        assert_eq!(
            ecs.create_and_attach(TUChildren {}, tentity_id0).unwrap(),
            tuchildren_id0
        );
        assert_eq!(
            ecs.create_and_attach(TFlag {}, tuchildren_id0).unwrap(),
            tflag_id0
        );

        assert_eq!(ecs.contains(tentity_id0), Ok(true));
        assert_eq!(ecs.contains(tuchildren_id0), Ok(true));
        assert_eq!(ecs.contains(tflag_id0), Ok(true));
        assert_eq!(
            ecs.get::<TUChildren>(tuchildren_id0),
            Ok(Some(&TUChildren {}))
        );
        assert_eq!(ecs.get::<TUChildren>(tentity_id0), Ok(Some(&TUChildren {})));
        assert_eq!(ecs.get::<TFlag>(tflag_id0), Ok(Some(&TFlag {})));
        assert_eq!(ecs.get::<TFlag>(tuchildren_id0), Ok(Some(&TFlag {})));

        assert_eq!(ecs.get_parent_id(tuchildren_id0), Ok(Some(tentity_id0)));
        assert_eq!(ecs.get_parent_id(tflag_id0), Ok(Some(tuchildren_id0)));
        assert_eq!(ecs.get_absolute_parent_id(tflag_id0), Ok(tentity_id0));

        // remove the tree and check that it is all gone
        assert_eq!(ecs.purge(tentity_id0), Ok(()));
        assert_eq!(ecs.id.contains(tentity_id0), Err(Error::InvalidatedId));
        assert_eq!(ecs.id.is_empty(IdType::from(0)), true);
        assert_eq!(ecs.id.is_empty(IdType::from(1)), true);
        assert_eq!(ecs.id.is_empty(IdType::from(2)), true);
    }

    #[test]
    fn remove_simple() {
        #[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
        struct TEntity {
            i: usize,
        }

        let mut ecs = Builder::new().add_component::<TEntity>().finalize();

        // create 2 entities
        let te0 = ecs.create(TEntity { i: 0 });
        let te1 = ecs.create(TEntity { i: 1 });
        assert!(ecs.contains(te0).unwrap());
        assert!(ecs.contains(te1).unwrap());

        // remove the first
        ecs.purge(te0).unwrap();

        // The ID's are now invalidated because of the remove
        assert_eq!(ecs.contains(te0), Err(Error::InvalidatedId));
        assert_eq!(ecs.contains(te1), Err(Error::InvalidatedId));

        // check that the second still exists
        assert!(ecs.id_iter::<TEntity>().next().is_some());
        assert_eq!(
            *ecs.get::<TEntity>(ecs.id_iter::<TEntity>().next().unwrap())
                .unwrap()
                .unwrap(),
            TEntity { i: 1 }
        );
    }

    #[test]
    fn remove_advanced() {
        #[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
        struct TEntity {
            id: u8,
        }
        #[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
        struct TFlag {
            id: u8,
        }

        let mut ecs = Builder::new()
            .add_component::<TEntity>()
            .add_component::<TFlag>()
            .finalize();

        let te0 = ecs.create(TEntity { id: 0 });
        let te1 = ecs.create(TEntity { id: 1 });
        let tf0 = ecs.create_and_attach(TFlag { id: 2 }, te0).unwrap();
        let tf1 = ecs.create_and_attach(TFlag { id: 3 }, te1).unwrap();
        assert_eq!(ecs.contains(te0), Ok(true));
        assert_eq!(ecs.contains(te1), Ok(true));
        assert_eq!(ecs.contains(tf0), Ok(true));
        assert_eq!(ecs.contains(tf1), Ok(true));

        ecs.purge(te0).unwrap();

        let mut count = 0;
        let mut query = <(Read<Target<TEntity>>, Read<Child<TFlag>>)>::build_query();
        query.run(&mut ecs, |te, tf| {
            assert_eq!(te.id, 1);
            assert_eq!(tf.id, 3);
            count += 1;
        });
        assert_eq!(count, 1);
    }

    #[test]
    fn purge() {
        #[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
        struct TEntity {
            id: u8,
        }
        #[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
        struct TFlag {
            id: u8,
        }

        let mut ecs = Builder::new()
            .add_component::<TEntity>()
            .add_component::<TFlag>()
            .finalize();

        let te0 = ecs.create(TEntity { id: 0 });
        let _ = ecs.create(TEntity { id: 1 });
        let tf0 = ecs.create_and_attach(TFlag { id: 2 }, te0).unwrap();
        assert_eq!(ecs.contains(te0), Ok(true));
        assert_eq!(ecs.contains(tf0), Ok(true));

        // Count all tentities
        let mut count = 0;
        <(Read<Target<TEntity>>,)>::build_query().run(&mut ecs, |_| {
            count += 1;
        });
        assert_eq!(count, 2);

        // Count only entities without childres, this will be the only one(s) that
        // survive the purge
        let mut count = 0;
        <(Read<Target<TEntity>>, Read<Not<Child<TFlag>>>)>::build_query().run(
            &mut ecs,
            |te, tf| {
                assert_eq!(te.id, 1);
                assert_eq!(**tf, ());
                count += 1;
            },
        );
        assert_eq!(count, 1);

        ecs.purge(te0).unwrap();

        let mut count = 0;
        <(Read<Target<TEntity>>,)>::build_query().run(&mut ecs, |te| {
            assert_eq!(te.id, 1);
            count += 1;
        });
        assert_eq!(count, 1);
    }

    #[test]
    fn purge_all() {
        #[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
        struct TEntity {
            id: u8,
        }
        #[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
        struct TFlag {
            id: u8,
        }

        let mut ecs = Builder::new()
            .add_component::<TEntity>()
            .add_component::<TFlag>()
            .finalize();

        let te0 = ecs.create(TEntity { id: 0 });
        let _ = ecs.create(TEntity { id: 1 });
        let te2 = ecs.create(TEntity { id: 2 });
        let tf0 = ecs.create_and_attach(TFlag { id: 2 }, te0).unwrap();
        let tf1 = ecs.create_and_attach(TFlag { id: 3 }, te2).unwrap();
        assert_eq!(ecs.contains(te0), Ok(true));
        assert_eq!(ecs.contains(te2), Ok(true));
        assert_eq!(ecs.contains(tf0), Ok(true));
        assert_eq!(ecs.contains(tf1), Ok(true));

        // Count all tentities
        let mut count = 0;
        <(Read<Target<TEntity>>,)>::build_query().run(&mut ecs, |_| {
            count += 1;
        });
        assert_eq!(count, 3);

        // Count only entities without childres, this will be the only one(s) that
        // survive the purge
        let mut count = 0;
        <(Read<Target<TEntity>>, Read<Not<Child<TFlag>>>)>::build_query().run(
            &mut ecs,
            |te, tf| {
                assert_eq!(te.id, 1);
                assert_eq!(**tf, ());
                count += 1;
            },
        );
        assert_eq!(count, 1);

        // Purge all entities with children of type TFlag
        <(Write<Target<TEntity>>, Read<Child<TFlag>>)>::build_query().purge_all(&mut ecs);

        let mut count = 0;
        <(Read<Target<TEntity>>,)>::build_query().run(&mut ecs, |te| {
            assert_eq!(te.id, 1);
            count += 1;
        });
        assert_eq!(count, 1);
    }

    #[test]
    fn remove_tree_1() {
        let _ = simple_logger::init();

        #[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
        struct TEntity {}
        #[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
        struct TUChildren {}
        #[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
        struct TFlag {}

        let mut ecs = Builder::new()
            .add_component::<TEntity>()
            .add_component_with_type::<TUChildren>(EntityRelationType::UniqueChildren)
            .add_component::<TFlag>()
            .finalize();

        // add an small tree of entities
        let parent = ecs.create(TEntity {});
        let middle = ecs.create(TUChildren {});
        let child = ecs.create(TFlag {});
        assert_eq!(ecs.len::<TEntity>(), 1);
        assert_eq!(ecs.len::<TUChildren>(), 1);
        assert_eq!(ecs.len::<TFlag>(), 1);
        ecs.attach(middle, parent).unwrap();
        ecs.attach(child, middle).unwrap();

        // remove the middle; check that the parent is now childlesss
        // and that the entity and it's children are removed
        assert_eq!(ecs.purge(middle), Ok(()));
        assert_eq!(ecs.len::<TEntity>(), 1);
        assert_eq!(ecs.len::<TUChildren>(), 0);
        assert_eq!(ecs.len::<TFlag>(), 0);
        assert_eq!(ecs.get_child_id::<TUChildren>(parent), Ok(None));
    }

    #[test]
    fn remove_tree_2() {
        let _ = simple_logger::init();

        #[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
        struct TEntity {}
        #[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
        struct TUChildren {}
        #[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
        struct TFlag {}

        let mut ecs = Builder::new()
            .add_component::<TEntity>()
            .add_component_with_type::<TUChildren>(EntityRelationType::UniqueChildren)
            .add_component::<TFlag>()
            .finalize();

        // add two trees of entities
        let parent_0 = ecs.create(TEntity {});
        let middle_0 = ecs.create(TUChildren {});
        let child_0 = ecs.create(TFlag {});
        assert_eq!(ecs.len::<TEntity>(), 1);
        assert_eq!(ecs.len::<TUChildren>(), 1);
        assert_eq!(ecs.len::<TFlag>(), 1);
        ecs.attach(middle_0, parent_0).unwrap();
        ecs.attach(child_0, middle_0).unwrap();

        let parent_1 = ecs.create(TEntity {});
        let middle_1 = ecs.create(TUChildren {});
        let child_1 = ecs.create(TFlag {});
        assert_eq!(ecs.len::<TEntity>(), 2);
        assert_eq!(ecs.len::<TUChildren>(), 2);
        assert_eq!(ecs.len::<TFlag>(), 2);
        ecs.attach(middle_1, parent_1).unwrap();
        ecs.attach(child_1, middle_1).unwrap();

        // remove the first.
        assert_eq!(ecs.purge(middle_0), Ok(()));
        assert_eq!(ecs.len::<TEntity>(), 2);
        assert_eq!(ecs.len::<TUChildren>(), 1);
        assert_eq!(ecs.len::<TFlag>(), 1);

        // check the integrity of the only tree
        let child_of_parent = ecs.get_child_id::<TUChildren>(parent_1).unwrap().unwrap();
        let child_of_entity = ecs.get_child_id::<TFlag>(child_of_parent).unwrap().unwrap();
        assert_ne!(
            ecs.get_parent_id(child_of_parent).unwrap().unwrap(),
            parent_1
        );
        assert_eq!(
            ecs.get_parent_id(child_of_parent)
                .unwrap()
                .unwrap()
                .to_id_internal_unchecked(),
            parent_1.to_id_internal_unchecked()
        );
        assert_eq!(
            ecs.get_parent_id(child_of_entity).unwrap().unwrap(),
            child_of_parent
        );
    }
}
