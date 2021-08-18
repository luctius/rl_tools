use proc_macro2::Span;
use syn::Type;

use crate::TypeId;

pub struct Component {
    pub id: TypeId,
    pub r#type: Type,
    pub children: Vec<Child>,
    pub span: Span,
}

pub enum ChildType {
    Single,
    Array(usize),
    Vec,
}

pub struct Child {
    pub id: TypeId,
    pub child_type: ChildType,
    pub span: Span,
}
