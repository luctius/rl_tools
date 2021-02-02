use std::vec::Vec;

use crate::id::{IdIdx, IdInt, IdType};

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub enum EntityRelationType {
    Flag,
    Root,
    UniqueChildren,
    MultipleChildren,
}
#[derive(Clone)]
pub enum EntityRelation<const N: usize> {
    Flag { parent: Option<IdInt>, },
    Root { children: [Option<IdIdx>; N], },
    UniqueChildren { parent: Option<IdInt>, children: [Option<IdIdx>; N], },
    MultipleChildren { parent: Option<IdInt>, children: Vec<IdInt>, },
}
impl<const N: usize> EntityRelation<{ N }> {
    pub fn new(ert: EntityRelationType) -> EntityRelation<{ N }> {
        match ert {
            EntityRelationType::Flag => EntityRelation::Flag { parent: None, },
            EntityRelationType::Root => EntityRelation::Root { children: [None; N], },
            EntityRelationType::UniqueChildren => {
                EntityRelation::UniqueChildren { parent: None, children: [None; N], }
            },
            EntityRelationType::MultipleChildren => {
                EntityRelation::MultipleChildren { parent: None, children: Vec::new(), }
            },
        }
    }

    #[rustfmt::skip]
    pub fn set_parent(&mut self, id: IdInt) -> bool {
        match *self {
            EntityRelation::Root { .. } => false,
            EntityRelation::Flag { ref mut parent }
            | EntityRelation::UniqueChildren { ref mut parent, .. }
            | EntityRelation::MultipleChildren { ref mut parent, .. }
            => {
                *parent = Some(id);
                true
            }
        }
    }

    #[rustfmt::skip]
    pub fn clear_parent(&mut self) {
        match *self {
            EntityRelation::Root { .. } => {}
            EntityRelation::Flag { ref mut parent }
            | EntityRelation::UniqueChildren { ref mut parent, .. }
            | EntityRelation::MultipleChildren { ref mut parent, .. }
            => {
                *parent = None;
            }
        }
    }

    #[rustfmt::skip]
    pub fn get_parent(&self) -> Option<IdInt> {
        match *self {
            EntityRelation::Root { .. } => None,
            EntityRelation::Flag { ref parent }
            | EntityRelation::UniqueChildren { ref parent, .. }
            | EntityRelation::MultipleChildren { ref parent, .. }
            => *parent,
        }
    }

    #[rustfmt::skip]
    pub fn add_child(&mut self, id: IdInt) -> bool {
        match *self {
            EntityRelation::Flag { .. } => false,
            EntityRelation::Root { ref mut children, .. }
            | EntityRelation::UniqueChildren { ref mut children, .. } => {
                let idx = usize::from(id.typeidx);
                if children[idx] == None {
                    children[idx] = Some(id.idx);
                    true
                } else {
                    false
                }
            }
            EntityRelation::MultipleChildren { ref mut children, .. } => {
                children.push(id);
                true
            }
        }
    }

    #[rustfmt::skip]
    #[allow(dead_code)]
    pub fn has_child_type(&self, tid: IdType) -> bool {
        match *self {
            EntityRelation::Flag { .. } => false,
            EntityRelation::Root { ref children, .. } | EntityRelation::UniqueChildren { ref children, .. } => {
                match children.get(usize::from(tid)) {
                    None => false,
                    Some(idopt) => idopt.is_some(),
                }
            }
            EntityRelation::MultipleChildren { ref children, .. } => {
                for cid in children.iter() {
                    if cid.typeidx == tid {
                        return true;
                    }
                }
                false
            }
        }
    }

    #[rustfmt::skip]
    pub fn get_child(&self, tid: IdType) -> Option<IdInt> {
        match *self {
            EntityRelation::Flag { .. } => None,
            EntityRelation::Root { ref children, .. } | EntityRelation::UniqueChildren { ref children, .. } => {
                let idx = usize::from(tid);
                match children.get(idx) {
                    None => None,
                    Some(idopt) => idopt.map(|idx| IdInt::new(tid, idx) ),
                }
            }
            EntityRelation::MultipleChildren { ref children, .. } => {
                for cid in children.iter() {
                    if cid.typeidx == tid {
                        return Some(*cid);
                    }
                }
                None
            }
        }
    }

    #[rustfmt::skip]
    pub fn clear_child(&mut self, id: IdInt) -> bool {
        match *self {
            EntityRelation::Flag { .. } => {false}
            EntityRelation::Root { ref mut children, .. }
            | EntityRelation::UniqueChildren { ref mut children, .. } => {
                let idx = usize::from(id.typeidx);
                if children.len() > idx {
                    if children[idx].is_some() {
                        children[idx] = None;
                        true
                    } else {
                        false
                    }
                }
                else {false}
            }
            EntityRelation::MultipleChildren { ref mut children, .. } => {
                for i in 0..children.len() {
                    if children[i] == id {
                        children.swap_remove(i);
                        return true;
                    }
                }
                false
            }
        }
    }

    #[rustfmt::skip]
    pub fn children(&self) -> EntityChildIter< {N} > {
        EntityChildIter {
            inner: self,
            pos: 0,
        }
    }
}
impl<const N: usize> core::fmt::Debug for EntityRelation<{ N }> {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> Result<(), core::fmt::Error> {
        match *self {
            EntityRelation::Flag { parent, } => write!(f, "Flag: Parent: {:?}", parent),
            EntityRelation::Root { ref children } => {
                write!(f, "Root: Children: [")?;
                for c in children.iter() {
                    if let Some(c) = c {
                        write!(f, "{:?},", c)?;
                    }
                }
                write!(f, "]")
            },
            EntityRelation::UniqueChildren { ref parent, ref children } => {
                write!(f, "UniqueChildren: Parent: {:?}, ", parent)?;
                write!(f, "Children: [")?;
                for c in children.iter() {
                    if let Some(c) = c {
                        write!(f, "{:?},", c)?;
                    }
                }
                write!(f, "]")
            }, EntityRelation::MultipleChildren { ref parent, ref children } => {
                write!(f, "MultipleChildren: Parent: {:?}, ", parent)?;
                write!(f, "Children: [")?;
                for c in children.iter() {
                    write!(f, "{:?},", c)?;
                }
                write!(f, "]")
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct EntityChildIter<'a, const N: usize> {
    inner: &'a EntityRelation<{ N }>,
    pos:   usize,
}

impl<'a, const N: usize> Iterator for EntityChildIter<'a, { N }> {
    type Item = IdInt;

    #[rustfmt::skip]
    fn next(&mut self) -> Option<Self::Item> {
        match self.inner {
            EntityRelation::Flag { .. } => None,
            EntityRelation::Root { ref children, .. } | EntityRelation::UniqueChildren { ref children, .. } => {
                while self.pos < children.len() {
                    self.pos += 1;
                    if let Some(Some(cidx) ) = children.get(self.pos - 1) {
                        return Some(IdInt::new(IdType::from(self.pos - 1), *cidx));
                    }
                }
                None
            },
            EntityRelation::MultipleChildren { ref children, .. } => {
                if self.pos < children.len() {
                    self.pos += 1;
                }
                children.get(self.pos).copied()
            },
        }
    }

    #[rustfmt::skip]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = match self.inner {
            EntityRelation::Flag { .. } => 0,
            EntityRelation::Root { ref children, .. } | EntityRelation::UniqueChildren { ref children, .. } => {
                children.len()
            },
            EntityRelation::MultipleChildren { ref children, .. } => {
                children.len()
            },
        };
        (0, Some(len - self.pos))
    }
}

#[cfg(test)]
mod tests {
    use crate::{entity_relation::{EntityRelation, EntityRelationType},
                id::{IdIdx, IdInt, IdType}};

    #[derive(Clone, PartialEq, Eq, Debug)]
    struct TE0 {}

    #[test]
    fn parent() {
        let parent = IdInt::new(IdType(0), IdIdx(0));

        let mut er: EntityRelation<1> = EntityRelation::new(EntityRelationType::Flag);
        er.set_parent(parent);
        assert_eq!(er.get_parent(), Some(parent));
        er.clear_parent();
        assert_eq!(er.get_parent(), None);

        let mut er: EntityRelation<1> = EntityRelation::new(EntityRelationType::Root);
        er.set_parent(parent);
        assert_eq!(er.get_parent(), None);
        er.clear_parent();
        assert_eq!(er.get_parent(), None);

        let mut euc: EntityRelation<1> = EntityRelation::new(EntityRelationType::UniqueChildren);
        euc.set_parent(parent);
        assert_eq!(euc.get_parent(), Some(parent));
        euc.clear_parent();
        assert_eq!(euc.get_parent(), None);
        let mut emc: EntityRelation<1> = EntityRelation::new(EntityRelationType::MultipleChildren);
        emc.set_parent(parent);
        assert_eq!(emc.get_parent(), Some(parent));
        emc.clear_parent();
        assert_eq!(emc.get_parent(), None);
    }
    #[test]
    fn add_child() {
        let ctype = IdType(0);
        let child = IdInt::new(ctype, IdIdx(0));
        let ochild = IdInt::new(IdType(1), IdIdx(1));

        let mut er: EntityRelation<1> = EntityRelation::new(EntityRelationType::Flag);
        assert_eq!(er.add_child(child), false);
        assert_eq!(er.has_child_type(ctype), false);
        assert_eq!(er.get_child(ctype), None);
        assert_eq!(er.clear_child(ochild), false);

        assert_eq!(er.clear_child(child), false);
        assert_eq!(er.has_child_type(ctype), false);
        assert_eq!(er.get_child(ctype), None);

        let mut er: EntityRelation<1> = EntityRelation::new(EntityRelationType::Root);
        assert_eq!(er.add_child(child), true);
        assert_eq!(er.has_child_type(ctype), true);
        assert_eq!(er.get_child(ctype), Some(child));
        assert_eq!(er.clear_child(ochild), false);

        assert_eq!(er.clear_child(child), true);
        assert_eq!(er.has_child_type(ctype), false);
        assert_eq!(er.get_child(ctype), None);

        let mut er: EntityRelation<1> = EntityRelation::new(EntityRelationType::UniqueChildren);
        assert_eq!(er.add_child(child), true);
        assert_eq!(er.has_child_type(ctype), true);
        assert_eq!(er.get_child(ctype), Some(child));
        assert_eq!(er.clear_child(ochild), false);

        assert_eq!(er.clear_child(child), true);
        assert_eq!(er.has_child_type(ctype), false);
        assert_eq!(er.get_child(ctype), None);

        let mut er: EntityRelation<1> = EntityRelation::new(EntityRelationType::MultipleChildren);
        assert_eq!(er.add_child(child), true);
        assert_eq!(er.has_child_type(ctype), true);
        assert_eq!(er.get_child(ctype), Some(child));
        assert_eq!(er.clear_child(ochild), false);

        assert_eq!(er.clear_child(child), true);
        assert_eq!(er.has_child_type(ctype), false);
        assert_eq!(er.get_child(ctype), None);
    }
}
