use std::any::Any;

use crate::{id::{Id, IdGenerationType, IdIdx, IdInt, IdType},
            stores::FetchDataNode,
            Error};

pub trait BinStorage
    where Self: BinStoragePrivate, {
    fn get<ITEM>(&self, id: Id) -> Result<Option<&ITEM>, Error>
        where ITEM: Any + 'static, {
        let gen = BinStoragePrivate::get_generation::<ITEM>(self);
        let id = id.to_id_internal(gen)?;

        Ok(BinStoragePrivate::get(self, id))
    }
    fn get_mut<ITEM>(&mut self, id: Id) -> Result<Option<&mut ITEM>, Error>
        where ITEM: Any + 'static, {
        let gen = BinStoragePrivate::get_generation::<ITEM>(self);
        let id = id.to_id_internal(gen)?;

        Ok(BinStoragePrivate::get_mut(self, id))
    }
    fn is_empty<ITEM>(&self) -> bool
        where ITEM: Any + 'static, {
        self.fetch_node::<ITEM>()
            .unwrap_or_else(|| panic!("Type of {} not registered!", std::any::type_name::<ITEM>()))
            .data
            .is_empty()
    }
    fn len<ITEM>(&self) -> usize
        where ITEM: Any + 'static, {
        self.fetch_node::<ITEM>()
            .unwrap_or_else(|| panic!("Type of {} not registered!", std::any::type_name::<ITEM>()))
            .data
            .len()
    }
    fn iter<ITEM>(&self) -> ComponentStorageIter<ITEM>
        where ITEM: Any + 'static, {
        if let Some(node) = self.fetch_node() {
            ComponentStorageIter { inner: node.data.as_slice(), pos: 0, }
        } else {
            ComponentStorageIter { inner: &[], pos: 0, }
        }
    }
    fn iter_mut<ITEM>(&mut self) -> ComponentStorageIterMut<ITEM>
        where ITEM: Any + 'static, {
        if let Some(node) = self.fetch_node_mut() {
            ComponentStorageIterMut { inner: &mut node.data, pos: 0, }
        } else {
            ComponentStorageIterMut { inner: &mut [], pos: 0, }
        }
    }
}

pub trait BinStoragePrivate
    where Self: FetchDataNode, {
    fn get_type_id<ITEM>(&self) -> IdType
        where ITEM: Any + 'static, {
        self.fetch_node::<ITEM>()
            .map(|n| n.id)
            .unwrap_or_else(|| panic!("Type of {} not registered!", std::any::type_name::<ITEM>()))
    }
    fn get_generation<ITEM>(&self) -> IdGenerationType
        where ITEM: Any + 'static, {
        self.fetch_node::<ITEM>().map(|n| n.gen).unwrap_or(0)
    }
    #[inline(never)]
    fn get<ITEM>(&self, id: IdInt) -> Option<&ITEM>
        where ITEM: Any + 'static, {
        let node =
            self.fetch_node().unwrap_or_else(|| panic!("Type of {} not registered!", std::any::type_name::<ITEM>()));

        if id.typeidx == node.id {
            node.data.get(usize::from(id.idx)).map(|i: &Option<_>| i.as_ref().unwrap())
        } else {
            None
        }
    }
    #[inline(never)]
    fn get_mut<ITEM>(&mut self, id: IdInt) -> Option<&mut ITEM>
        where ITEM: Any + 'static, {
        let node = self.fetch_node_mut()
                       .unwrap_or_else(|| panic!("Type of {} not registered!", std::any::type_name::<ITEM>()));
        if id.typeidx == node.id { node.data.get_mut(usize::from(id.idx)).map(|i| i.as_mut().unwrap()) } else { None }
    }
    fn add<ITEM>(&mut self, item: ITEM)
        where ITEM: Any + 'static, {
        let node = self.fetch_node_mut()
                       .unwrap_or_else(|| panic!("Type of {} not registered!", std::any::type_name::<ITEM>()));
        node.data.push(Some(item));
    }
    fn swap_remove<ITEM>(&mut self, id: IdInt) -> Result<Option<IdInt>, Error>
        where ITEM: Any + 'static, {
        let node = self.fetch_node_mut::<ITEM>()
                       .unwrap_or_else(|| panic!("Type of {} not registered!", std::any::type_name::<ITEM>()));
        let len = node.data.len();
        if id.typeidx == node.id && usize::from(id.idx) < len {
            node.gen = node.gen.wrapping_add(1);

            if usize::from(id.idx) == len - 1 {
                node.data.remove(usize::from(id.idx));
                Ok(None)
            } else {
                let last_id = IdInt::new(id.typeidx, IdIdx::from(len - 1));
                node.data.swap_remove(usize::from(id.idx));
                Ok(Some(last_id))
            }
        } else {
            Err(Error::NoSuchId)
        }
    }
    fn swap_remove_by_id(&mut self, id: IdInt) -> Result<Option<IdInt>, Error> { self.remove_by_id(id) }

    #[inline(never)]
    fn take<ITEM>(&mut self, id: IdInt) -> Option<ITEM>
        where ITEM: Any + 'static, {
        let node = self.fetch_node_mut::<ITEM>()
                       .unwrap_or_else(|| panic!("Type of {} not registered!", std::any::type_name::<ITEM>()));

        assert_eq!(id.typeidx, node.id, "Getting wrong Item Type for ID!");
        node.data.get_mut(usize::from(id.idx)).map(|i| i.take().expect("Internal Error: Item already taken"))
    }
    #[inline(never)]
    fn put<ITEM>(&mut self, id: IdInt, item: ITEM)
        where ITEM: Any + 'static, {
        let node = self.fetch_node_mut::<ITEM>()
                       .unwrap_or_else(|| panic!("Type of {} not registered!", std::any::type_name::<ITEM>()));
        assert_eq!(id.typeidx, node.id, "Putting away wrong Item Type for ID!");
        *node.data.get_mut(usize::from(id.idx)).unwrap() = Some(item);
    }
    fn reserve<ITEM>(&mut self, size: usize)
        where ITEM: Any + 'static, {
        let node = self.fetch_node_mut::<ITEM>()
                       .unwrap_or_else(|| panic!("Type of {} not registered!", std::any::type_name::<ITEM>()));
        node.data.reserve(size);
    }
}

#[derive(Debug)]
pub struct ComponentStorageIter<'a, ITEM: 'a>
    where ITEM: Any + 'static, {
    pub(crate) inner: &'a [Option<ITEM>],
    pub(crate) pos:   usize,
}

impl<'a, ITEM> Iterator for ComponentStorageIter<'a, ITEM> where ITEM: Any + 'static, {
    type Item = &'a ITEM;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos < self.inner.len() {
            self.pos += 1;
            match &self.inner[self.pos - 1] {
                None => panic!("Error: Component already taken!"),
                Some(data) => {
                    return Some(&data);
                },
            }
        }

        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) { (0, Some(self.inner.len() - self.pos)) }
}

#[derive(Debug)]
pub struct ComponentStorageIterMut<'a, ITEM: 'a>
    where ITEM: Any + 'static, {
    pub(crate) inner: &'a mut [Option<ITEM>],
    pub(crate) pos:   usize,
}

impl<'a, ITEM> Iterator for ComponentStorageIterMut<'a, ITEM> where ITEM: Any + 'static, {
    type Item = &'a mut ITEM;

    fn next(&mut self) -> Option<Self::Item> {
        let slice = std::mem::replace(&mut self.inner, &mut []);
        if slice.is_empty() {
            return None;
        }
        self.pos += 1;

        let (l, r) = slice.split_at_mut(1);
        self.inner = r;

        match &mut l[0] {
            None => panic!("Error: Component already taken!"),
            Some(data) => Some(data),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) { (0, Some(self.inner.len() - self.pos)) }
}

#[cfg(test)]
mod tests {
    // use crate::rl_ecs::RlEcsBuilder;
    use crate::{bin_storage::{BinStorage, BinStoragePrivate, ComponentStorageIterMut},
                id::{Id, IdIdx, IdInt, IdType},
                stores::{Builder, FinishBuilding as _}};

    #[test]
    fn add() {
        #[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
        struct Test1 {}
        #[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
        struct Test2 {}

        let mut ecs = Builder::new().add_component::<Test1>().add_component::<Test2>().finalize();

        let t1 = Test1 {};
        let t2 = Test2 {};

        BinStoragePrivate::add(&mut ecs.bin, t1);
        BinStoragePrivate::add(&mut ecs.bin, t2);
        assert_eq!(BinStorage::len::<Test1>(&ecs.bin), 1);
        assert_eq!(BinStorage::len::<Test2>(&ecs.bin), 1);
    }

    #[test]
    fn get() {
        #[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
        struct Test1 {}
        #[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
        struct Test2 {}

        let mut ecs = Builder::new().add_component::<Test1>().add_component::<Test2>().finalize();

        let t1 = Test1 {};
        let t2 = Test2 {};

        BinStoragePrivate::add(&mut ecs.bin, t1);
        BinStoragePrivate::add(&mut ecs.bin, t2);
        assert_eq!(BinStorage::get::<Test1>(&ecs.bin, Id::new(IdType::from(0), IdIdx::from(0), 0)).map(|opt| {
                                                                                                      opt.map(|t| *t)
                                                                                                  }),
                   Ok(Some(Test1 {})));
        assert_eq!(BinStorage::get::<Test2>(&ecs.bin, Id::new(IdType::from(1), IdIdx::from(0), 0)).map(|opt| {
                                                                                                      opt.map(|t| *t)
                                                                                                  }),
                   Ok(Some(Test2 {})));
    }

    #[test]
    fn iter() {
        #[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
        struct Test1 {
            i: usize,
        }

        let mut ecs = Builder::new().add_component::<Test1>().finalize();

        let t1 = Test1 { i: 0, };
        let t2 = Test1 { i: 1, };

        BinStoragePrivate::add(&mut ecs.bin, t1);
        BinStoragePrivate::add(&mut ecs.bin, t2);

        {
            let mut iter = BinStorage::iter(&ecs.bin);
            assert_eq!(iter.next().map(|t| *t), Some(Test1 { i: 0, }));
            assert_eq!(iter.next().map(|t| *t), Some(Test1 { i: 1, }));
            assert_eq!(iter.next(), None);
        }

        {
            let mut iter_mut: ComponentStorageIterMut<'_, Test1> = BinStorage::iter_mut(&mut ecs.bin);
            let mut t = iter_mut.next().unwrap();
            t.i = 10;
            let mut t = iter_mut.next().unwrap();
            t.i = 20;
            assert_eq!(iter_mut.next(), None);
        }

        {
            let mut iter = BinStorage::iter(&ecs.bin);
            assert_eq!(iter.next().map(|t| *t), Some(Test1 { i: 10, }));
            assert_eq!(iter.next().map(|t| *t), Some(Test1 { i: 20, }));
            assert_eq!(iter.next(), None);
        }
    }

    #[test]
    fn remove() {
        #[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
        struct Test1 {
            i: usize,
        }

        let mut ecs = Builder::new().add_component::<Test1>().finalize();

        let t1 = Test1 { i: 0, };
        let t2 = Test1 { i: 1, };

        BinStoragePrivate::add(&mut ecs.bin, t1);
        BinStoragePrivate::add(&mut ecs.bin, t2);

        {
            let mut iter = BinStorage::iter(&ecs.bin);
            assert_eq!(iter.next().map(|t| *t), Some(Test1 { i: 0, }));
            assert_eq!(iter.next().map(|t| *t), Some(Test1 { i: 1, }));
            assert_eq!(iter.next(), None);
        }

        let tid0 = IdInt::new(IdType::from(0), IdIdx::from(0));
        BinStoragePrivate::swap_remove::<Test1>(&mut ecs.bin, tid0).unwrap();

        {
            let mut iter = BinStorage::iter(&ecs.bin);
            assert_eq!(iter.next().map(|t| *t), Some(Test1 { i: 1, }));
            assert_eq!(iter.next(), None);
        }
    }

    #[test]
    #[should_panic(expected = "Type of rl_ecs::bin_storage::tests::type_not_registered::T2 not registered!")]
    fn type_not_registered() {
        struct T1 {}
        struct T2 {}

        let mut ecs = Builder::new().add_component::<T1>().finalize();

        BinStoragePrivate::add(&mut ecs.bin, T2 {});
    }
}
