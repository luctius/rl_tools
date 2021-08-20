pub mod key;
pub mod stores;

pub mod slotmap {
    pub use slotmap::{basic::Iter, new_key_type, Key, KeyData, SecondaryMap, SlotMap};
}

pub mod arrayvec {
    pub use arrayvec::ArrayVec;
}
pub use rl_ecs_codegen::create_ecs;

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub enum Error {
    InternalError,
    NoSuchId,
    InvalidatedId,
    EntityHasParent,
    EntityNoParent,
    ChildSlotFull,
    ChildSlotEmpty,
}

impl std::fmt::Display for Error {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidatedId           => write!(f, "This entity Id has been invalidated; probably because an entity has been deleted."),
            Error::EntityHasParent         => write!(f, "Entity already has a parent, or has no parent slot. Remember that an Root Entity has no parent."),
            Error::EntityNoParent          => write!(f, "Entity has no parent."),
            Error::ChildSlotFull           => write!(f, "The child slot of this type is already taken, or unavailable. Remember that a Flag Entity does not have children."),
            Error::ChildSlotEmpty          => write!(f, "The child slot of this type is is empty, there is nothing to detach."),

            // These should never occur
            Error::InternalError           => write!(f, "Internal Error."),
            Error::NoSuchId                => write!(f, "Entity Id is not known."),
        }
    }
}

impl std::error::Error for Error {}
