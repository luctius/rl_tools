use super::QueryAtom;
use std::{any::Any,
          ops::{Deref, DerefMut}};

#[derive(PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Ref<T>(T);
#[derive(PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct RefMut<T>(T);

impl<T> Deref for Ref<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl<T> Deref for RefMut<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target { &self.0 }
}

impl<T> DerefMut for RefMut<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.0 }
}

pub struct ReadToken<T>(pub(crate) T);
pub struct WriteToken<T>(pub(crate) T);

pub trait Access
    where Self::Atom: QueryAtom,
          <Self::Atom as QueryAtom>::Item: Any + 'static, {
    type Atom;
    type RefOutput;
    type Token;

    fn wrap(item: <Self::Atom as QueryAtom>::ComponentOutput) -> Self::RefOutput;
    fn unwrap(guard: Self::RefOutput) -> <Self::Atom as QueryAtom>::ComponentOutput;

    fn to_token(id: <Self::Atom as QueryAtom>::IdIntOutput) -> Self::Token;
}

pub struct Read<S>(S)
    where S: QueryAtom,
          <S as QueryAtom>::Item: Any + 'static;
pub struct Write<S>(S)
    where S: QueryAtom,
          <S as QueryAtom>::Item: Any + 'static;

impl<ITEM> Access for Read<ITEM> where ITEM: QueryAtom, {
    type Atom = ITEM;
    type RefOutput = Ref<<Self::Atom as QueryAtom>::ComponentOutput>;
    type Token = ReadToken<<Self::Atom as QueryAtom>::IdIntOutput>;

    #[inline]
    fn wrap(item: <Self::Atom as QueryAtom>::ComponentOutput) -> Self::RefOutput { Ref(item) }

    #[inline]
    fn unwrap(guard: Self::RefOutput) -> <Self::Atom as QueryAtom>::ComponentOutput { guard.0 }

    #[inline]
    fn to_token(id: <Self::Atom as QueryAtom>::IdIntOutput) -> Self::Token { ReadToken(id) }
}

impl<ITEM> Access for Write<ITEM> where ITEM: QueryAtom, {
    type Atom = ITEM;
    type RefOutput = RefMut<<Self::Atom as QueryAtom>::ComponentOutput>;
    type Token = WriteToken<<Self::Atom as QueryAtom>::IdIntOutput>;

    #[inline]
    fn wrap(item: <Self::Atom as QueryAtom>::ComponentOutput) -> Self::RefOutput { RefMut(item) }

    #[inline]
    fn unwrap(guard: Self::RefOutput) -> <Self::Atom as QueryAtom>::ComponentOutput { guard.0 }

    #[inline]
    fn to_token(id: <Self::Atom as QueryAtom>::IdIntOutput) -> Self::Token { WriteToken(id) }
}
