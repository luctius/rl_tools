use crate::{id::Id, Error};
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
    pub fn new() -> Self { Self { bin: SlotMap::with_key(), id: SecondaryMap::new(), } }
}
impl<T, CIDT, KEY> Default for Store<T, CIDT, KEY> where KEY: Key, {
    fn default() -> Self { Self::new() }
}

pub trait Ecs<KEYTYPE, KEYIDX, ID: Id<Type = KEYTYPE, Idx = KEYIDX>> {
    fn attach(&mut self, parent: ID, child: ID) -> Result<(), Error>;
    fn detach(&mut self, parent: ID, child: ID) -> Result<(), Error>;
    fn purge(&mut self, k: ID) -> Result<(), Error>;
    fn get_parent(&self, child: ID) -> Result<Option<ID>, Error>;
}

pub trait StoreEx<KEYTYPE, KEYIDX, ID: Id<Type = KEYTYPE, Idx = KEYIDX>> {
    fn attach(&mut self, parent: ID, child: ID) -> Result<(), Error>;
    fn detach(&mut self, parent: ID, child: ID) -> Result<(), Error>;
    fn get_parent(&self, child: ID) -> Result<Option<ID>, Error>;
    fn get_child(&self, parent: ID, child_type: KEYTYPE) -> Result<Option<ID>, Error>;
    fn get_children(&self, parent: ID) -> Result<Vec<ID>, Error>;
    fn remove(&mut self, key: ID) -> Result<(), Error>;
}

pub trait RlEcsStore<ID, KEYTYPE, T, KEY>
    where ID: Id<Idx = KEY, Type = KEYTYPE>,
          KEY: Key, {
    fn create(&mut self, t: T) -> KEY;
    fn create_and_attach(&mut self, parent: KEY, t: T) -> Result<KEY, Error>;
    fn get(&self, k: KEY) -> Result<Option<&T>, Error>;
    fn get_mut(&mut self, k: KEY) -> Result<Option<&mut T>, Error>;
    fn is_empty(&self) -> bool;
}

pub trait Query<KEYTYPE, KEYIDX, ID: Id<Type = KEYTYPE, Idx = KEYIDX>, ECS, T>: Iterator<Item = T>
    where ECS: Ecs<KEYTYPE, KEYIDX, ID>, {
    fn purge_all(ecs: &mut ECS);
    fn purge_some<FUNC>(ecs: &mut ECS, func: FUNC)
        where FUNC: FnMut(T) -> bool;
    fn run<FUNC>(ecs: &mut ECS, func: FUNC)
        where FUNC: FnMut(T);
}

pub trait ResourceStore<T> {
    fn get_unique(&self) -> &T;
    fn get_unique_mut(&mut self) -> &mut T;
}
