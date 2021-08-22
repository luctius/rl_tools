use crate::key::KeyExt;
use serde::{Deserialize, Serialize};
use slotmap::{basic::Iter, SecondaryMap, SlotMap};

use slotmap::Key;

// TODO: make Serde optional
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Store<T, CIDT, KEY>
    where KEY: Key, {
    pub bin: SlotMap<KEY, T>,
    pub id:  SecondaryMap<KEY, CIDT>,
}
impl<T, CIDT, KEY> Store<T, CIDT, KEY> where KEY: Key, {
    #[must_use]
    pub fn new() -> Self { Self { bin: SlotMap::with_key(), id: SecondaryMap::new(), } }
}
impl<T, CIDT, KEY> Default for Store<T, CIDT, KEY> where KEY: Key, {
    fn default() -> Self { Self::new() }
}

pub trait StoreExGetParent<KEY, PKEY>
    where KEY: Key + KeyExt,
          PKEY: Key + KeyExt, {
    fn get_parent(&self, child: KEY) -> Option<PKEY>;
}

#[doc(hidden)]
pub trait StoreExSetParent<KEY, PKEY>
    where KEY: Key + KeyExt,
          PKEY: Key + KeyExt, {
    fn set_parent(&mut self, child: KEY, parent: PKEY) -> bool;
    fn clear_parent(&mut self, child: KEY, parent: PKEY) -> bool;
}

pub trait StoreExGetChild<KEY, CKEY>
    where KEY: Key + KeyExt,
          CKEY: Key + KeyExt, {
    fn get_child(&self, parent: KEY) -> Option<std::slice::Iter<CKEY>>;
    #[doc(hidden)]
    fn set_child(&mut self, parent: KEY, child: CKEY) -> bool;
    #[doc(hidden)]
    fn clear_child(&mut self, parent: KEY, child: CKEY) -> bool;
}

pub trait StoreExBasic<T, KEY> {
    fn get(&self, k: KEY) -> Option<&T>;
    fn is_empty(&self) -> bool;
}

pub trait StoreExBasicMut<T, KEY> {
    fn get_mut(&mut self, k: KEY) -> Option<&mut T>;
}

pub trait StoreExCreate<T, KEY>
    where KEY: Key, {
    fn create(&mut self, t: T) -> KEY;
    fn remove(&mut self, key: KEY) -> Option<()>;
}

pub trait StoreExAttach<PKEY, CKEY>
    where PKEY: Key + KeyExt,
          CKEY: Key + KeyExt, {
    fn attach(&mut self, pkey: PKEY, ckey: CKEY) -> bool;
    fn detach(&mut self, pkey: PKEY, ckey: CKEY) -> bool;
}
impl<PKEY, CKEY, S> StoreExAttach<PKEY, CKEY> for S
    where S: StoreExGetChild<PKEY, CKEY> + StoreExSetParent<CKEY, PKEY>,
          PKEY: Key + KeyExt,
          CKEY: Key + KeyExt,
{
    fn attach(&mut self, pkey: PKEY, ckey: CKEY) -> bool { self.set_parent(ckey, pkey) && self.set_child(pkey, ckey) }

    fn detach(&mut self, pkey: PKEY, ckey: CKEY) -> bool {
        self.clear_parent(ckey, pkey);
        self.clear_child(pkey, ckey)
    }
}

pub trait StoreExCreateAttach<T, PKEY, CKEY>
    where PKEY: Key + KeyExt,
          CKEY: Key + KeyExt, {
    fn create_and_attach(&mut self, parent: PKEY, item: T) -> Option<CKEY>;
}
impl<T, PKEY, CKEY, S> StoreExCreateAttach<T, PKEY, CKEY> for S
    where S: StoreExCreate<T, CKEY>
              + StoreExAttach<PKEY, CKEY>
              + StoreExGetChild<PKEY, CKEY>
              + StoreExGetParent<CKEY, PKEY>,
          PKEY: Key + KeyExt,
          CKEY: Key + KeyExt,
{
    fn create_and_attach(&mut self, parent: PKEY, item: T) -> Option<CKEY> {
        let key: CKEY = self.create(item);
        self.attach(parent, key).then(|| key)
    }
}

pub trait StoreExPurge<KEY>
    where KEY: Key, {
    fn purge(&mut self, key: KEY);
}

// pub trait Query<KEYTYPE, KEYIDX, ID: Id<Type = KEYTYPE, Idx = KEYIDX>, ECS,
// T>: Iterator<Item = T>     where ECS: Ecs<KEYTYPE, KEYIDX, ID>, {
//     fn purge_all(ecs: &mut ECS);
//     fn purge_some<FUNC>(ecs: &mut ECS, func: FUNC)
//         where FUNC: FnMut(T) -> bool;
//     fn run<FUNC>(ecs: &mut ECS, func: FUNC)
//         where FUNC: FnMut(T);
// }

pub trait ResourceStore<T> {
    fn get_unique(&self) -> &T;
    fn get_unique_mut(&mut self) -> &mut T;
}
