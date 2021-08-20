pub mod key;
pub mod stores;

pub mod slotmap {
    pub use slotmap::{basic::Iter, new_key_type, Key, KeyData, SecondaryMap, SlotMap};
}

pub mod arrayvec {
    pub use arrayvec::ArrayVec;
}
pub use rl_ecs_codegen::create_ecs;
