use std::{convert::From, fmt};

use crate::Error;

pub type IdIdxType = u32;
pub type IdTypeType = u16;
pub type IdGenerationType = u16;

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct IdIdx(pub IdIdxType);

impl fmt::Display for IdIdx {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.0) }
}

impl From<IdIdx> for usize {
    fn from(idx: IdIdx) -> Self { idx.0 as usize }
}

impl From<usize> for IdIdx {
    fn from(idx: usize) -> Self { IdIdx(idx as IdIdxType) }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct IdType(pub IdTypeType);

impl fmt::Display for IdType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "{}", self.0) }
}

impl From<IdType> for usize {
    fn from(idx: IdType) -> Self { idx.0 as usize }
}

impl From<usize> for IdType {
    fn from(tp: usize) -> Self { IdType(tp as IdTypeType) }
}

impl From<u32> for IdType {
    fn from(tp: u32) -> Self { IdType(tp as IdTypeType) }
}

impl From<i32> for IdType {
    fn from(tp: i32) -> Self { IdType(tp as IdTypeType) }
}

impl From<u16> for IdType {
    fn from(tp: u16) -> Self { IdType(tp as IdTypeType) }
}

impl From<u8> for IdType {
    fn from(tp: u8) -> Self { IdType(tp as IdTypeType) }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct IdInt {
    pub(crate) typeidx: IdType,
    pub(crate) idx:     IdIdx,
}

impl IdInt {
    #[inline]
    pub(crate) fn new(typeidx: IdType, idx: IdIdx) -> IdInt {
        IdInt { typeidx,
                idx }
    }

    #[inline]
    pub(crate) fn to_id(self, gen: IdGenerationType) -> Id {
        Id { typeidx: self.typeidx, idx: self.idx, generation: gen, }
    }
}

impl fmt::Display for IdInt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { write!(f, "({}:{})", self.typeidx, self.idx) }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Id {
    pub(crate) typeidx:    IdType,
    pub(crate) idx:        IdIdx,
    pub(crate) generation: IdGenerationType,
}

impl Id {
    #[inline]
    pub(crate) fn new(typeidx: IdType, idx: IdIdx, generation: IdGenerationType) -> Id {
        Id { typeidx,
             idx,
             generation }
    }

    #[inline]
    pub(crate) fn to_id_internal(self, gen: IdGenerationType) -> Result<IdInt, Error> {
        if self.generation == gen { Ok(self.to_id_internal_unchecked()) } else { Err(Error::InvalidatedId) }
    }

    #[inline]
    pub(crate) fn to_id_internal_unchecked(self) -> IdInt { IdInt { typeidx: self.typeidx, idx: self.idx, } }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}:{};{})", self.typeidx, self.idx, self.generation)
    }
}
