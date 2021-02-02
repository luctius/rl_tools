use std::{any::{Any, TypeId},
          vec::Vec};

use crate::{entity_relation::{EntityChildIter, EntityRelation, EntityRelationType},
            id::{Id, IdGenerationType, IdIdx, IdInt, IdType},
            Error};

#[derive(Clone, Debug)]
pub struct IdTypeStore<const N: usize> {
    pub typeid:          TypeId,
    pub gen:             IdGenerationType,
    pub entity_relation: EntityRelationType,
    pub data:            Vec<EntityRelation<{ N }>>,
}
impl<const N: usize> IdTypeStore<{ N }> {
    pub fn new(typeid: TypeId, ert: EntityRelationType) -> IdTypeStore<{ N }> {
        IdTypeStore { typeid,
                      gen: 0,
                      entity_relation: ert,
                      data: Vec::with_capacity(64) }
    }

    pub fn len(&self) -> usize { self.data.len() }

    pub fn add(&mut self) -> IdIdx {
        let ert = self.entity_relation;
        self.data.push(EntityRelation::new(ert));
        (self.data.len() - 1).into()
    }

    #[inline]
    pub fn contains(&self, idx: IdIdx) -> bool { usize::from(idx) < self.data.len() }

    pub fn reserve(&mut self, len: usize) { self.data.reserve(len); }

    pub fn swap_remove(&mut self, idx: IdIdx) -> Result<Option<IdIdx>, Error> {
        let len = self.data.len();
        if usize::from(idx) < len {
            self.gen = self.gen.wrapping_add(1);
            if usize::from(idx) == len - 1 {
                self.data.remove(usize::from(idx));
                Ok(None)
            } else {
                let last_idx = IdIdx::from(len - 1);
                self.data.swap_remove(usize::from(idx));
                Ok(Some(last_idx))
            }
        } else {
            Err(Error::NoSuchId)
        }
    }

    #[inline]
    #[must_use]
    pub fn get(&self, idx: IdIdx) -> Option<&EntityRelation<{ N }>> {
        if let Some(ert) = self.data.get(usize::from(idx)) { Some(ert) } else { None }
    }

    #[inline]
    #[must_use]
    pub fn get_mut(&mut self, idx: IdIdx) -> Option<&mut EntityRelation<{ N }>> {
        if let Some(ert) = self.data.get_mut(usize::from(idx)) { Some(ert) } else { None }
    }
}

pub(crate) trait IdStoragePrivate<const N: usize> {
    #[allow(clippy::new_ret_no_self)]
    fn new() -> IdStorage<{ N }>;
    fn register_type<T: Any + 'static>(&mut self, typ: IdType, ert: EntityRelationType);
    fn new_id(&mut self, typ: IdType) -> IdInt;
    fn add_internal(&mut self, id: IdInt) -> Id;
    fn add(&mut self, typ: IdType) -> Id;
    fn swap_remove(&mut self, id: IdInt) -> Result<Option<IdInt>, Error>;
    fn get_idts(&self, id: IdInt) -> Option<&EntityRelation<{ N }>>;
    fn get_idts_mut(&mut self, id: IdInt) -> Option<&mut EntityRelation<{ N }>>;
    fn clear_idts(&mut self, id: IdInt) -> Result<Option<IdInt>, Error>;
    fn get_child_id(&self, id: IdInt, child_type: IdType) -> Result<Option<IdInt>, Error>;
    fn get_parent_id(&self, id: IdInt) -> Result<Option<IdInt>, Error>;
    fn attach(&mut self, child_id: IdInt, parent_id: IdInt) -> Result<(), Error>;
    fn detach(&mut self, id: IdInt) -> Result<(), Error>;
    fn get_absolute_parent_id(&self, id: IdInt) -> IdInt;
    fn contains(&self, id: IdInt) -> bool;
    fn len(&self, typeidx: IdType) -> usize;
    fn is_empty(&self, typeidx: IdType) -> bool;
    fn get_generation(&self, typeid: IdType) -> IdGenerationType;
    fn iter(&self, typeidx: IdType) -> IdStorageIter;
    fn child_iter_int(&self, id: IdInt) -> Option<IdStorageChildIter<{ N }>>;
}

#[derive(Clone)]
pub struct IdStorage<const N: usize> {
    data: [IdTypeStore<{ N }>; N],
}
impl<const N: usize> core::fmt::Debug for IdStorage<{ N }> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> Result<(), core::fmt::Error> {
        writeln!(f, "IdStorage<{}> [", N)?;
        for s in self.data.iter() {
            writeln!(f, "\t{:#?},", s)?;
        }
        write!(f, "]")
    }
}
impl<const N: usize> IdStoragePrivate<{ N }> for IdStorage<{ N }> {
    fn new() -> IdStorage<{ N }> {
        let data = {
            // See: https://doc.rust-lang.org/stable/std/mem/union.MaybeUninit.html#initializing-an-array-element-by-element
            // Tried to do this with arr_macro and varies other crates, but they do not
            // (yet) work with const generics
            let mut data: [std::mem::MaybeUninit<IdTypeStore<{ N }>>; N] =
                unsafe { std::mem::MaybeUninit::uninit().assume_init() };

            for elem in &mut data[..] {
                *elem = std::mem::MaybeUninit::new(IdTypeStore::new(TypeId::of::<u8>(), EntityRelationType::Root));
            }

            unsafe { core::mem::transmute_copy::<_, [IdTypeStore<{ N }>; N]>(&data) }
        };

        IdStorage { data }
    }

    fn register_type<T: Any + 'static>(&mut self, typ: IdType, ert: EntityRelationType) {
        let typeid = usize::from(typ);
        self.data[typeid] = IdTypeStore::new(TypeId::of::<T>(), ert);
    }

    #[inline]
    fn add_internal(&mut self, id: IdInt) -> Id {
        let typeid = usize::from(id.typeidx);

        self.data[typeid].add();

        id.to_id(self.data[typeid].gen)
    }

    #[inline]
    fn new_id(&mut self, typ: IdType) -> IdInt {
        IdInt::new(typ, self.data.get(usize::from(typ)).map_or(0.into(), |d| d.len().into()))
    }

    fn add(&mut self, typ: IdType) -> Id {
        let id = self.new_id(typ);

        self.add_internal(id)
    }

    #[inline]
    fn get_idts(&self, id: IdInt) -> Option<&EntityRelation<{ N }>> {
        // from type vector get id vector
        self.data[usize::from(id.typeidx)].get(id.idx)
    }

    #[inline]
    fn get_idts_mut(&mut self, id: IdInt) -> Option<&mut EntityRelation<{ N }>> {
        // from type vector get id vector
        self.data[usize::from(id.typeidx)].get_mut(id.idx)
    }

    fn clear_idts(&mut self, id: IdInt) -> Result<Option<IdInt>, Error> {
        // from type vector get id vector
        match self.data.get_mut(usize::from(id.typeidx)) {
            None => Err(Error::NoSuchId),
            // from id vector get specific id
            Some(idts) => idts.swap_remove(id.idx).map(|op_idx| op_idx.map(|idx| IdInt::new(id.typeidx, idx))),
        }
    }

    #[inline]
    fn contains(&self, id: IdInt) -> bool { self.data[usize::from(id.typeidx)].contains(id.idx) }

    fn swap_remove(&mut self, id: IdInt) -> Result<Option<IdInt>, Error> {
        if IdStoragePrivate::contains(self, id) {
            // Detach from parent
            if let Err(e) = IdStoragePrivate::detach(self, id) {
                if e != Error::EntityNoParent {
                    return Err(e);
                }
            }

            if let Some(swapped_id) = self.clear_idts(id)? {
                // Now update the relatives of the swapped_id
                // to the changed id.
                // which was swapped from swapped_id to id.

                if let Some(pid) = IdStoragePrivate::get_parent_id(self, id)? {
                    let mut parent_updated = false;
                    if let Some(parent_ide) = self.get_idts_mut(pid) {
                        if parent_ide.clear_child(swapped_id) && parent_ide.add_child(id) {
                            parent_updated = true;
                        }
                    }

                    if !parent_updated {
                        return Err(Error::InternalError);
                    }
                }

                if let Some(iter) = IdStoragePrivate::child_iter_int(self, id) {
                    // TODO: remove collecting
                    for child in iter.collect::<Vec<_>>() {
                        if let Some(child_idts) = self.get_idts_mut(child.to_id_internal_unchecked()) {
                            child_idts.clear_parent();
                            if !child_idts.set_parent(id) {
                                return Err(Error::InternalError);
                            }
                        }
                    }
                }

                Ok(Some(swapped_id))
            } else {
                Ok(None)
            }
        } else {
            Err(Error::NoSuchId)
        }
    }

    fn is_empty(&self, typeidx: IdType) -> bool {
        self.data.get(usize::from(typeidx)).map(|v| v.len() == 0).unwrap_or(true)
    }

    fn len(&self, typeidx: IdType) -> usize { self.data.get(usize::from(typeidx)).map(|v| v.len()).unwrap_or(0) }

    fn iter(&self, typeidx: IdType) -> IdStorageIter {
        if let Some(idts) = self.data.get(usize::from(typeidx)) {
            IdStorageIter { pos_back: self.len(typeidx), gen: idts.gen, typ: typeidx, pos_front: 0, }
        } else {
            IdStorageIter { pos_back: 0, typ: typeidx, gen: 0, pos_front: 0, }
        }
    }

    fn child_iter_int(&self, id: IdInt) -> Option<IdStorageChildIter<{ N }>> {
        let gen = self.get_generation(id.typeidx);

        if let Some(idts) = self.get_idts(id) {
            return Some(IdStorageChildIter { inner: idts.children(),
                                             gen });
        }
        None
    }

    fn get_child_id(&self, id: IdInt, child_type: IdType) -> Result<Option<IdInt>, Error> {
        self.get_idts(id).ok_or(Error::NoSuchId).map(|ide| ide.get_child(child_type))
    }

    fn get_parent_id(&self, id: IdInt) -> Result<Option<IdInt>, Error> {
        self.get_idts(id).ok_or(Error::NoSuchId).map(|ide| ide.get_parent())
    }

    fn get_absolute_parent_id(&self, id: IdInt) -> IdInt {
        let mut pid_ret = id;

        while let Ok(Some(pid)) = IdStoragePrivate::get_parent_id(self, pid_ret) {
            assert_ne!(pid, id, "Endless chain detected!");
            pid_ret = pid;
        }

        pid_ret
    }

    fn get_generation(&self, typeid: IdType) -> IdGenerationType {
        // from type vector get id vector
        if let Some(idts) = self.data.get(usize::from(typeid)) { idts.gen } else { 0 }
    }

    fn attach(&mut self, child_id: IdInt, parent_id: IdInt) -> Result<(), Error> {
        if child_id.typeidx == parent_id.typeidx {
            return Err(Error::CannotAttachToSameType);
        }
        if !IdStoragePrivate::contains(self, child_id) {
            return Err(Error::NoSuchId);
        }
        if !IdStoragePrivate::contains(self, parent_id) {
            return Err(Error::NoSuchId);
        }
        if IdStoragePrivate::get_parent_id(self, child_id) != Ok(None) {
            return Err(Error::EntityHasParent);
        }
        if IdStoragePrivate::get_child_id(self, parent_id, child_id.typeidx) != Ok(None) {
            return Err(Error::ChildSlotFull);
        }
        if let Some(ide) = self.get_idts_mut(parent_id) {
            if !ide.add_child(child_id) {
                return Err(Error::ChildSlotFull);
            }
        } else {
            panic!("Unable to get parent id; which must exist")
        }

        if let Some(ide) = self.get_idts_mut(child_id) {
            if ide.set_parent(parent_id) {
                // check for loops
                IdStoragePrivate::get_absolute_parent_id(self, child_id);

                return Ok(());
            }
        }

        // Recover from being unable to set parent
        if let Some(ide) = self.get_idts_mut(parent_id) {
            if !ide.clear_child(child_id) {
                panic!("Internal Error: Unable to clean parent of child after wrongfull birth!")
            } else {
                Err(Error::EntityHasParent)
            }
        } else {
            // We have edited the parent before;
            unreachable!()
        }
    }

    fn detach(&mut self, id: IdInt) -> Result<(), Error> {
        if !IdStoragePrivate::contains(self, id) {
            Err(Error::NoSuchId)
        } else if let Some(pid) = IdStoragePrivate::get_parent_id(self, id).map_err(|err| {
                                                                               if err == Error::NoSuchId {
                                                                                   Error::EntityNoParent
                                                                               } else {
                                                                                   err
                                                                               }
                                                                           })?
        {
            // clear reference from this to parent
            let ide = self.get_idts_mut(id).ok_or(Error::NoSuchId)?;
            ide.clear_parent();

            // Clear reference from parent to this
            // we know the parent exists
            self.get_idts_mut(pid).expect("Internal Error: Id not accepted").clear_child(id);

            Ok(())
        } else {
            Err(Error::EntityNoParent)
        }
    }
}
impl<const N: usize> IdStorage<{ N }> {
    pub fn contains(&self, id: Id) -> Result<bool, Error> {
        let gen = self.get_generation(id.typeidx);
        let id = id.to_id_internal(gen)?;
        Ok(IdStoragePrivate::contains(self, id))
    }

    pub fn get_parent_id(&self, id: Id) -> Result<Option<Id>, Error> {
        let gen = self.get_generation(id.typeidx);
        let id = id.to_id_internal(gen)?;

        IdStoragePrivate::get_parent_id(self, id).map(|op_id| op_id.map(|id| id.to_id(gen)))
    }

    pub fn get_absolute_parent_id(&self, id: Id) -> Result<Id, Error> {
        let gen = self.get_generation(id.typeidx);
        let id = id.to_id_internal(gen)?;

        let pid = IdStoragePrivate::get_absolute_parent_id(self, id);
        let pid_gen = self.get_generation(pid.typeidx);
        Ok(pid.to_id(pid_gen))
    }

    pub fn attach(&mut self, child_id: Id, parent_id: Id) -> Result<(), Error> {
        let child_gen = self.get_generation(child_id.typeidx);
        let child_id = child_id.to_id_internal(child_gen)?;
        let parent_gen = self.get_generation(parent_id.typeidx);
        let parent_id = parent_id.to_id_internal(parent_gen)?;

        IdStoragePrivate::attach(self, child_id, parent_id)
    }

    pub fn detach(&mut self, id: Id) -> Result<(), Error> {
        let gen = self.get_generation(id.typeidx);
        let id = id.to_id_internal(gen)?;

        IdStoragePrivate::detach(self, id)
    }

    pub fn reserve(&mut self, idtype: IdType, len: usize) { self.data[usize::from(idtype)].reserve(len); }

    pub fn child_iter(&self, id: Id) -> Result<Option<IdStorageChildIter<{ N }>>, Error> {
        let gen = self.get_generation(id.typeidx);
        let id = id.to_id_internal(gen)?;

        if !IdStoragePrivate::contains(self, id) {
            Err(Error::NoSuchId)
        } else {
            Ok(IdStoragePrivate::child_iter_int(self, id))
        }
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct IdStorageIter {
    typ:       IdType,
    gen:       IdGenerationType,
    pos_front: usize,
    pos_back:  usize,
}

impl Iterator for IdStorageIter {
    type Item = Id;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.pos_front;
        if result < self.pos_back {
            self.pos_front += 1;
            Some(Id::new(self.typ, (result as usize).into(), self.gen))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) { (0, Some(self.pos_back - self.pos_front)) }
}

impl DoubleEndedIterator for IdStorageIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        let result = self.pos_back;
        if result > self.pos_front {
            self.pos_back -= 1;
            Some(Id::new(self.typ, (result - 1).into(), self.gen))
        } else {
            None
        }
    }
}

#[derive(Clone, Debug)]
pub struct IdStorageChildIter<'a, const N: usize> {
    inner: EntityChildIter<'a, { N }>,
    gen:   IdGenerationType,
}

impl<'a, const N: usize> Iterator for IdStorageChildIter<'a, { N }> {
    type Item = Id;

    fn next(&mut self) -> Option<Self::Item> { self.inner.next().map(|idint| idint.to_id(self.gen)) }

    fn size_hint(&self) -> (usize, Option<usize>) { self.inner.size_hint() }
}

#[cfg(test)]
mod tests {
    use crate::{entity_relation::EntityRelationType,
                id::{Id, IdIdx, IdInt, IdType},
                id_storage::{IdStorage, IdStoragePrivate, IdTypeStore},
                Error};
    use std::any::TypeId;

    #[test]
    fn idts_add() {
        let _ = simple_logger::init();

        // Add items and check if the exist
        let mut idts: IdTypeStore<2> = IdTypeStore::new(TypeId::of::<usize>(), EntityRelationType::Flag);
        let idx0 = idts.add();
        assert_eq!(idts.len(), 1);
        let idx1 = idts.add();
        assert_eq!(idts.len(), 2);
        assert!(idts.contains(idx0));
        assert!(idts.contains(idx1));
        assert!(idts.get(idx0).is_some());
        assert!(idts.get(idx1).is_some());
        assert!(idts.get_mut(idx0).is_some());
        assert!(idts.get_mut(idx1).is_some());

        // Check failure modes
        assert_eq!(idts.contains(IdIdx::from(2)), false);
        assert!(idts.get(IdIdx::from(2)).is_none());
        assert!(idts.get_mut(IdIdx::from(2)).is_none());
    }

    #[test]
    fn idts_remove() {
        let _ = simple_logger::init();

        // add first item
        let mut idts: IdTypeStore<2> = IdTypeStore::new(TypeId::of::<usize>(), EntityRelationType::Flag);
        let idx0 = idts.add();

        // check that adding does not affect generation
        assert_eq!(idts.gen, 0);

        // remove nonexisting item; check that is does error correctly and that is does
        // not affect generation
        assert_eq!(idts.swap_remove(IdIdx::from(2)), Err(Error::NoSuchId));
        assert_eq!(idts.gen, 0);

        // remove only item, check that it returns Ok(None) and it does bump generation
        assert_eq!(idts.swap_remove(idx0), Ok(None));
        assert_eq!(idts.gen, 1);

        // add two items, remove the first
        // the returned idx should be 1, check that it now exists on place 0
        let idx0 = idts.add();
        let idx1 = idts.add();
        assert_eq!(idts.swap_remove(idx0), Ok(Some(idx1)));
        assert!(idts.contains(idx0));
        assert_eq!(idts.contains(idx1), false);
    }

    #[test]
    fn idstorage_add() {
        let _ = simple_logger::init();

        let mut ids: IdStorage<4> = IdStorage::new();
        assert!(ids.is_empty(IdType(0)));
        assert!(ids.is_empty(IdType(3)));
        assert_eq!(ids.len(IdType(0)), 0);
        assert_eq!(ids.len(IdType(3)), 0);

        ids.register_type::<u8>(IdType(0), EntityRelationType::Flag);
        ids.register_type::<u16>(IdType(1), EntityRelationType::Flag);
        ids.register_type::<u32>(IdType(2), EntityRelationType::Flag);
        ids.register_type::<u64>(IdType(3), EntityRelationType::Flag);

        // add two items and check their existance
        let id0_0 = ids.add(IdType(0));
        assert_eq!(id0_0, Id::new(IdType(0), IdIdx::from(0), 0));
        assert_eq!(ids.contains(id0_0), Ok(true));
        let id0_1 = ids.add(IdType(0));
        assert_eq!(id0_1, Id::new(IdType(0), IdIdx::from(1), 0));
        assert_eq!(ids.contains(id0_1), Ok(true));
        assert_eq!(ids.len(IdType(0)), 2);

        // add two items from a different type, atleast 1 removed from the previous, and
        // check their existance
        let id3_0 = ids.add(IdType(3));
        assert_eq!(id3_0, Id::new(IdType(3), IdIdx::from(0), 0));
        assert_eq!(ids.contains(id3_0), Ok(true));
        let id3_1 = ids.add(IdType(3));
        assert_eq!(id3_1, Id::new(IdType(3), IdIdx::from(1), 0));
        assert_eq!(ids.contains(id3_1), Ok(true));
        assert_eq!(ids.len(IdType(3)), 2);

        // check that a nonexisting but corrent id results in a false
        let id3_2 = Id::new(IdType(3), IdIdx::from(2), 0);
        assert_eq!(ids.contains(id3_2), Ok(false));

        // check that an existing but incorrent id results in an error
        let id3_1f = Id::new(IdType(3), IdIdx::from(1), 2);
        assert_eq!(ids.contains(id3_1f), Err(Error::InvalidatedId));
    }
    #[test]
    fn idstorage_attach() {
        let _ = simple_logger::init();

        let mut ids: IdStorage<2> = IdStorage::new();

        ids.register_type::<u8>(IdType(0), EntityRelationType::Root);
        ids.register_type::<u16>(IdType(1), EntityRelationType::Flag);

        // add two entities, attach them and check for success
        let id0_0 = ids.add(IdType(0));
        let id1_0 = ids.add(IdType(1));
        assert_eq!(ids.attach(id1_0, id0_0), Ok(()));
        assert_eq!(ids.get_parent_id(id1_0), Ok(Some(id0_0)));
        assert_eq!(IdStoragePrivate::get_child_id(&ids, id0_0.to_id_internal_unchecked(), id1_0.typeidx),
                   Ok(Some(id1_0.to_id_internal_unchecked())));

        // attach a non-existing child to enitity and check for error
        let id1_1 = Id::new(IdType(1), IdIdx::from(1), 0);
        assert_eq!(ids.attach(id1_1, id0_0), Err(Error::NoSuchId));
        assert_eq!(ids.get_parent_id(id1_1), Err(Error::NoSuchId));

        // attach an existing entity to a non-existing parent and check for error
        let id0_1 = Id::new(IdType(0), IdIdx::from(1), 0);
        let id1_1 = ids.add(IdType(1));
        assert_eq!(ids.attach(id1_1, id0_1), Err(Error::NoSuchId));
        assert_eq!(IdStoragePrivate::get_child_id(&ids, id0_1.to_id_internal_unchecked(), id1_0.typeidx),
                   Err(Error::NoSuchId));

        // attach invalid child to existing parent
        let id1_0f = Id::new(IdType(1), IdIdx::from(0), 1);
        assert_eq!(ids.attach(id1_0f, id0_0), Err(Error::InvalidatedId));

        // attach valid child to invalid parent
        let id0_0f = Id::new(IdType(0), IdIdx::from(0), 1);
        assert_eq!(ids.attach(id1_0, id0_0f), Err(Error::InvalidatedId));
        assert_eq!(ids.get_parent_id(id0_0f), Err(Error::InvalidatedId));

        // add new entity and try to add child to second parent
        let id0_1 = ids.add(IdType(0));
        assert_eq!(ids.attach(id1_0, id0_1), Err(Error::EntityHasParent));

        // add new entity and try to add second child of same type to parent
        let id1_1 = ids.add(IdType(1));
        assert_eq!(ids.attach(id1_1, id0_0), Err(Error::ChildSlotFull));
    }

    #[test]
    fn idstorage_detach() {
        let _ = simple_logger::init();

        let mut ids: IdStorage<2> = IdStorage::new();
        ids.register_type::<u8>(IdType(0), EntityRelationType::Root);
        ids.register_type::<u16>(IdType(1), EntityRelationType::Flag);

        // Set-up to attached entities, and check if detaching succeeds
        let id0_0 = ids.add(IdType(0));
        let id1_0 = ids.add(IdType(1));
        ids.attach(id1_0, id0_0).unwrap();
        assert_eq!(ids.detach(id1_0), Ok(()));
        assert_eq!(ids.get_parent_id(id1_0), Ok(None));
        assert_eq!(IdStoragePrivate::get_child_id(&ids, id1_0.to_id_internal_unchecked(), id1_0.typeidx), Ok(None));

        // detach again
        assert_eq!(ids.detach(id1_0), Err(Error::EntityNoParent));

        // detach with a non-existing id
        let id1_1 = Id::new(IdType(1), IdIdx::from(1), 0);
        assert_eq!(ids.detach(id1_1), Err(Error::NoSuchId));

        // detach with a invalid id
        let id1_0f = Id::new(IdType(1), IdIdx::from(0), 1);
        assert_eq!(ids.detach(id1_0f), Err(Error::InvalidatedId));
    }

    #[test]
    fn idstorage_remove() {
        let _ = simple_logger::init();

        let mut ids: IdStorage<2> = IdStorage::new();
        ids.register_type::<u8>(IdType(0), EntityRelationType::Root);
        ids.register_type::<u16>(IdType(1), EntityRelationType::Flag);

        // remove a non-existing id
        let id1_1 = IdInt::new(IdType(1), IdIdx::from(1));
        assert_eq!(ids.swap_remove(id1_1), Err(Error::NoSuchId));

        // add entity and remove it, check that it is gone
        let id0_0 = ids.add(IdType(0));
        assert_eq!(ids.len(IdType(0)), 1);
        assert_eq!(ids.swap_remove(id0_0.to_id_internal_unchecked()), Ok(None));
        assert_eq!(ids.len(IdType(0)), 0);

        // check that the generation has been changed
        let id0_0 = Id::new(IdType(0), IdIdx::from(0), 0);
        assert_eq!(ids.contains(id0_0), Err(Error::InvalidatedId));

        // add 2 entities, remove the first, check that the second is returned.
        let id0_0 = ids.add(IdType(0));
        let id0_1 = ids.add(IdType(0));
        assert_eq!(ids.len(IdType(0)), 2);
        assert_eq!(ids.swap_remove(id0_0.to_id_internal_unchecked()), Ok(Some(id0_1.to_id_internal_unchecked())));
        assert_eq!(ids.len(IdType(0)), 1);
    }

    #[test]
    fn idstorage_iter() {
        let _ = simple_logger::init();

        let mut ids: IdStorage<1> = IdStorage::new();
        ids.register_type::<u8>(IdType(0), EntityRelationType::Root);

        let id0_0 = ids.add(IdType(0));
        let id0_1 = ids.add(IdType(0));
        let id0_2 = ids.add(IdType(0));
        let mut iter = IdStoragePrivate::iter(&ids, IdType(0));
        assert_eq!(iter.next(), Some(id0_0));
        assert_eq!(iter.next(), Some(id0_1));
        assert_eq!(iter.next(), Some(id0_2));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn idstorage_child_iter() {
        let _ = simple_logger::init();

        let mut ids: IdStorage<3> = IdStorage::new();
        ids.register_type::<u8>(IdType(0), EntityRelationType::Root);
        ids.register_type::<u16>(IdType(1), EntityRelationType::UniqueChildren);
        ids.register_type::<u32>(IdType(2), EntityRelationType::Flag);

        let parent = ids.add(IdType(0));
        let entity = ids.add(IdType(1));
        let child = ids.add(IdType(2));
        ids.attach(entity, parent).unwrap();
        ids.attach(child, parent).unwrap();

        let mut iter = ids.child_iter(parent).unwrap().unwrap();
        assert_eq!(iter.next(), Some(entity));
        assert_eq!(iter.next(), Some(child));
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);

        // request iter for an id without children
        let mut iter = ids.child_iter(child).unwrap().unwrap();
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);

        // request iter for a non-existing id
        let id1_100 = Id::new(IdType(1), IdIdx::from(100), 0);
        assert_eq!(ids.child_iter(id1_100).unwrap_err(), Error::NoSuchId);

        // request iter for an invalid id
        let id0_0f = Id::new(IdType(0), IdIdx::from(0), 8);
        assert_eq!(ids.child_iter(id0_0f).unwrap_err(), Error::InvalidatedId);
    }
}
