pub mod id;
pub mod stores;

use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub enum Error {
    InternalError,
    WrongIdType,
    WrongItemType,
    NoSuchId,
    InvalidatedId,
    EntityHasParent,
    EntityNoParent,
    ChildSlotFull,
    ChildSlotEmpty,
    NoSuchChildSlot,
    CannotAttachToSameType,
}

impl std::fmt::Display for Error {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::WrongIdType             => write!(f, "The Id given does not match the request."),
            Error::WrongItemType           => write!(f, "Unable to complete the requested action to the given item type."),
            Error::InvalidatedId           => write!(f, "This entity Id has been invalidated; probably because an entity has been deleted."),
            Error::EntityHasParent         => write!(f, "Entity already has a parent, or has no parent slot. Remember that an Root Entity has no parent."),
            Error::EntityNoParent          => write!(f, "Entity has no parent."),
            Error::ChildSlotFull           => write!(f, "The child slot of this type is already taken, or unavailable. Remember that a Flag Entity does not have children."),
            Error::ChildSlotEmpty          => write!(f, "The child slot of this type is is empty, there is nothing to detach."),
            Error::CannotAttachToSameType  => write!(f, "Cannot Attach two components of the same type."),

            // These should never occur
            Error::InternalError           => write!(f, "Internal Error."),
            Error::NoSuchId                => write!(f, "Entity Id is not known."),
            Error::NoSuchChildSlot         => write!(f, "The Entity has no option to attach this type of child."),
        }
    }
}

impl std::error::Error for Error {}
