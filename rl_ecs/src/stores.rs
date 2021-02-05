use crate::{entity_relation::EntityRelationType,
            id::{IdGenerationType, IdIdx, IdInt, IdType},
            id_storage::{IdStorage, IdStoragePrivate},
            rl_ecs::RlEcs,
            Error};

use crate::bin_storage::{BinStorage, BinStoragePrivate};

use std::{any::{Any, TypeId},
          marker::PhantomData};

pub trait ChainSize {
    const SIZE: usize;
}

pub trait FetchDataNode {
    fn fetch_node<T: Any + 'static>(&self) -> Option<&DataNode<T>>;
    fn fetch_node_mut<T: Any + 'static>(&mut self) -> Option<&mut DataNode<T>>;
    fn remove_by_id(&mut self, victim: IdInt) -> Result<Option<IdInt>, Error>;
}

pub trait FetchResources {
    type Output;
    fn fetch_resource<'a>(&'a self) -> &'a Self::Output;
    fn fetch_resource_mut<'a>(&'a mut self) -> &'a mut Self::Output;
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Builder {}
impl Builder {
    pub const fn new() -> Self { Builder {} }

    pub const fn add_component<T>(self) -> BuilderNode<T> { BuilderNode::new_with_type(EntityRelationType::Root) }

    pub const fn add_component_with_type<T>(self, ert: EntityRelationType) -> BuilderNode<T> {
        BuilderNode::new_with_type(ert)
    }
}

pub trait FinishBuildingPrivate
    where Self::Output: Sized + BinStorage, {
    type Output;
    fn finish_priv(self, sz: usize) -> Self::Output;
    fn register_self<const N: usize>(&self, ids: &mut IdStorage<{ N }>, sz: usize);
}

pub trait FinishBuilding
    where Self: FinishBuildingPrivate + ChainSize + Sized, {
    fn finalize(self) -> RlEcs<<Self as FinishBuildingPrivate>::Output, { Self::SIZE }> {
        let size = Self::SIZE - 1;
        let mut ids = IdStorage::new();

        self.register_self(&mut ids, size);

        RlEcs { id: ids, bin: self.finish_priv(size), }
    }

    fn finalize_with_resource<T>(self,
                                 resource: T)
                                 -> RlEcs<ResourceNode<<Self as FinishBuildingPrivate>::Output, T>, { Self::SIZE }>
    {
        let size = Self::SIZE - 1;
        let mut ids = IdStorage::new();

        self.register_self(&mut ids, size);

        RlEcs { id:  ids,
                bin: ResourceNode { node: self.finish_priv(size),
                                    resource }, }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct BuilderNode<A>(EntityRelationType, PhantomData<A>);
impl<A> BuilderNode<A> {
    const fn new() -> Self { Self(EntityRelationType::Flag, PhantomData) }

    const fn new_with_type(ert: EntityRelationType) -> Self { Self(ert, PhantomData) }

    pub const fn add_component<T: Any + 'static>(self) -> BuilderChainNode2<BuilderNode<A>, BuilderNode<T>> {
        BuilderChainNode2 { node_l: self, node_r: BuilderNode::new(), }
    }

    pub const fn add_component_with_type<T: Any + 'static>(self,
                                                           ert: EntityRelationType)
                                                           -> BuilderChainNode2<BuilderNode<A>, BuilderNode<T>>
    {
        BuilderChainNode2 { node_l: self, node_r: BuilderNode::new_with_type(ert), }
    }
}
impl<A: 'static> FinishBuildingPrivate for BuilderNode<A> {
    type Output = DataNode<A>;

    fn finish_priv(self, sz: usize) -> Self::Output { Self::Output::new(IdType::from(sz)) }

    fn register_self<const N: usize>(&self, ids: &mut IdStorage<{ N }>, sz: usize) {
        ids.register_type::<A>(IdType::from(sz), self.0);
    }
}
impl<A> FinishBuilding for BuilderNode<A> where Self: FinishBuildingPrivate + Sized {}
impl<A> ChainSize for BuilderNode<A> {
    const SIZE: usize = 1;
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct BuilderChainNode2<A, B> {
    node_l: A,
    node_r: B,
}
impl<A, B> BuilderChainNode2<A, B>
    where A: Any + 'static,
          B: Any + 'static,
{
    pub const fn add_component<T: Any + 'static>(self) -> BuilderChainNode2<BuilderChainNode2<A, B>, BuilderNode<T>> {
        BuilderChainNode2 { node_l: self, node_r: BuilderNode::new(), }
    }

    pub const fn add_component_with_type<T: Any + 'static>(
        self,
        ert: EntityRelationType)
        -> BuilderChainNode2<BuilderChainNode2<A, B>, BuilderNode<T>>
    {
        BuilderChainNode2 { node_l: self, node_r: BuilderNode::new_with_type(ert), }
    }
}
impl<A, B> ChainSize for BuilderChainNode2<A, B>
    where A: ChainSize,
          B: ChainSize,
{
    const SIZE: usize = A::SIZE + B::SIZE;
}
impl<A, B> FinishBuildingPrivate for BuilderChainNode2<A, B>
    where A: FinishBuildingPrivate + 'static,
          B: FinishBuildingPrivate + 'static,
{
    type Output = DataChainNode2<<A as FinishBuildingPrivate>::Output, <B as FinishBuildingPrivate>::Output>;

    fn finish_priv(self, sz: usize) -> Self::Output {
        let node_l = self.node_l.finish_priv(sz - 1);
        let node_r = self.node_r.finish_priv(sz);
        Self::Output { node_l,
                       node_r }
    }

    fn register_self<const N: usize>(&self, ids: &mut IdStorage<{ N }>, sz: usize) {
        self.node_l.register_self(ids, sz - 1);
        self.node_r.register_self(ids, sz);
    }
}
impl<A, B> FinishBuilding for BuilderChainNode2<A, B>
    where Self: FinishBuildingPrivate + Sized,
          A: ChainSize,
          B: ChainSize,
{
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct DataNode<A>
    where A: Any + 'static, {
    pub id:   IdType,
    pub gen:  IdGenerationType,
    pub data: Vec<Option<A>>,
}
impl<A> DataNode<A> {
    const fn new(id: IdType) -> Self {
        Self { id,
               gen: 0,
               data: Vec::new() }
    }
}
impl<A> ChainSize for DataNode<A> {
    const SIZE: usize = 1;
}
impl<A> FetchDataNode for DataNode<A>
    where A: Any + 'static,
          DataNode<A>: Any + 'static,
{
    fn fetch_node<T: Any + 'static>(&self) -> Option<&DataNode<T>> {
        if TypeId::of::<T>() == TypeId::of::<A>() { Any::downcast_ref::<DataNode<T>>(self) } else { None }
    }

    fn fetch_node_mut<T: Any + 'static>(&mut self) -> Option<&mut DataNode<T>> {
        if TypeId::of::<T>() == TypeId::of::<A>() { Any::downcast_mut::<DataNode<T>>(self) } else { None }
    }

    fn remove_by_id(&mut self, id: IdInt) -> Result<Option<IdInt>, Error> {
        let len = self.data.len();

        if id.typeidx == self.id && usize::from(id.idx) < len {
            self.gen = self.gen.wrapping_add(1);

            if usize::from(id.idx) == len - 1 {
                self.data.remove(usize::from(id.idx));
                Ok(None)
            } else {
                let last_id = IdInt::new(id.typeidx, IdIdx::from(len - 1));
                self.data.swap_remove(usize::from(id.idx));
                Ok(Some(last_id))
            }
        } else {
            Err(Error::NoSuchId)
        }
    }
}
impl<A> BinStoragePrivate for DataNode<A> where Self: FetchDataNode {}
impl<A> BinStorage for DataNode<A> where Self: FetchDataNode {}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct DataChainNode2<A, B> {
    node_l: A,
    node_r: B,
}
impl<A, B> ChainSize for DataChainNode2<A, B>
    where A: ChainSize,
          B: ChainSize,
{
    const SIZE: usize = A::SIZE + B::SIZE;
}
impl<A, B> FetchDataNode for DataChainNode2<A, B>
    where A: Any + 'static + FetchDataNode,
          B: Any + 'static + FetchDataNode,
{
    fn fetch_node<T: Any + 'static>(&self) -> Option<&DataNode<T>> {
        self.node_l.fetch_node().or_else(|| self.node_r.fetch_node())
    }

    fn fetch_node_mut<T: Any + 'static>(&mut self) -> Option<&mut DataNode<T>> {
        if let Some(n) = self.node_l.fetch_node_mut() {
            Some(n)
        } else if let Some(n) = self.node_r.fetch_node_mut() {
            Some(n)
        } else {
            None
        }
    }

    fn remove_by_id(&mut self, victim: IdInt) -> Result<Option<IdInt>, Error> {
        if let Ok(n) = self.node_l.remove_by_id(victim) {
            Ok(n)
        } else if let Ok(n) = self.node_r.remove_by_id(victim) {
            Ok(n)
        } else {
            Err(Error::NoSuchId)
        }
    }
}
impl<A, B> BinStoragePrivate for DataChainNode2<A, B> where Self: FetchDataNode {}
impl<A, B> BinStorage for DataChainNode2<A, B> where Self: FetchDataNode {}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct ResourceNode<A, B>
    where A: BinStorage, {
    node:     A,
    resource: B,
}
impl<A, B> ChainSize for ResourceNode<A, B> where A: BinStorage + ChainSize, {
    const SIZE: usize = A::SIZE;
}
impl<A, B> FetchDataNode for ResourceNode<A, B> where A: BinStorage + FetchDataNode, {
    fn fetch_node<T: Any + 'static>(&self) -> Option<&DataNode<T>> { self.node.fetch_node() }

    fn fetch_node_mut<T: Any + 'static>(&mut self) -> Option<&mut DataNode<T>> { self.node.fetch_node_mut() }

    fn remove_by_id(&mut self, victim: IdInt) -> Result<Option<IdInt>, Error> { self.node.remove_by_id(victim) }
}
impl<A, B> FetchResources for ResourceNode<A, B> where A: BinStorage, {
    type Output = B;

    fn fetch_resource<'a>(&'a self) -> &'a Self::Output { &self.resource }

    fn fetch_resource_mut<'a>(&'a mut self) -> &'a mut Self::Output { &mut self.resource }
}
impl<A, B> BinStoragePrivate for ResourceNode<A, B>
    where Self: FetchDataNode,
          A: BinStorage,
{
}
impl<A, B> BinStorage for ResourceNode<A, B>
    where Self: FetchDataNode,
          A: BinStorage,
{
}

#[cfg(test)]
mod tests {
    use super::{Builder, FetchDataNode, FinishBuilding};

    #[test]
    fn build() {
        #[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
        struct T1 {}
        #[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
        struct T2 {}
        #[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
        struct T3 {}
        #[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
        struct T4 {}
        #[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
        struct T5 {}

        let world = Builder::new().add_component::<T1>()
                                  .add_component::<T2>()
                                  .add_component::<T3>()
                                  .add_component::<T4>()
                                  .add_component::<T5>();

        println!("{:#?}", world);
        let world = world.finalize();
        println!("{:#?}", world);
        println!("{:#?}", world.bin.fetch_node::<T2>());
    }
}
