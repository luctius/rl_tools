#![feature(const_fn)]
//#![feature(min_const_generics)]
#![feature(const_generics)]
#![feature(const_evaluatable_checked)]

#[macro_use]
mod macros;
// mod id_pool;
mod bin_storage;
mod entity_relation;
mod id;
mod id_storage;
mod query;
mod rl_ecs;
pub mod stores;

pub use crate::{bin_storage::BinStorage,
                entity_relation::EntityRelationType,
                id::Id,
                id_storage::IdStorage,
                query::{access::{Read, Ref, RefMut, Write},
                        Child, GrandParent, Parent, Query, QueryBuilder, QueryPurgeAll, QueryPurgeAny, QueryRun,
                        QueryRunWorldMut, Sibbling, Target},
                rl_ecs::RlEcs};

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Error {
    InternalError,
    WrongItemType,
    NoSuchId,
    InvalidatedId,
    EntityHasParent,
    EntityNoParent,
    ChildSlotFull,
    CannotAttachToSameType,
}

impl std::fmt::Display for Error {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::WrongItemType           => write!(f, "Unable to complete the requested action the the given item type."),
            Error::InvalidatedId           => write!(f, "This entity Id has been invalidated; probably because an entity has been deleted."),
            Error::EntityHasParent         => write!(f, "Entity already has a parent, or has no parent slot. Remember that an Root Entity has no parent."),
            Error::EntityNoParent          => write!(f, "Entity has no parent."),
            Error::ChildSlotFull           => write!(f, "The child slot of this type is already taken, or unavailable. Remember that a Flag Entity does not have children."),
            Error::CannotAttachToSameType  => write!(f, "Cannot Attach two components of the same type."),

            // These should never occur
            Error::InternalError           => write!(f, "Internal Error."),
            Error::NoSuchId                => write!(f, "Entity Id is not known."),
        }
    }
}

impl std::error::Error for Error {}
