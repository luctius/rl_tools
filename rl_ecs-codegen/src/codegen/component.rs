use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote_spanned};
use syn::{spanned::Spanned, Ident};

use crate::codegen::unique::{CodeGenUnique, CodeGenUniqueNames};
use crate::validation::{
    component::{Child, ChildType, Component},
    AllComponents, AllUniques,
};

pub fn gen_mod_components(ecs: &Ident, all: &AllComponents, uniques: &AllUniques) -> TokenStream {
    let span = ecs.span();

    let component_imports: Vec<TokenStream> = all.values().map(|c| c.gen_imports()).collect();

    let comp_store_atoms: Vec<TokenStream> = all
        .values()
        .map(|c| c.gen_store_atom(all, uniques))
        .collect();

    let comp_ecs_impls: Vec<TokenStream> = all.values().map(|c| c.gen_ecs_impl(ecs, all)).collect();

    quote_spanned! {span =>
        mod components {
            use core::convert::{TryFrom, TryInto};
            use rl_ecs::key::KeyExt;
            use rl_ecs::stores::Store;
            use rl_ecs::stores::{StoreExBasic, StoreExBasicMut,
                StoreExCreate,StoreExGetParent,StoreExSetParent,
                StoreExGetChild, StoreExPurge};
            use rl_ecs::slotmap::Key;
            use rl_ecs::arrayvec::ArrayVec;

            #(#component_imports)*
            #(#comp_store_atoms)*

            #(#comp_ecs_impls)*
        }
    }
}

pub trait CodeGenComponentNames {
    fn to_store_name(&self) -> Ident;
    fn to_child_struct_name(&self) -> Ident;
    fn to_parent_enum_key(&self) -> Ident;
    fn to_parent_enum(&self) -> Ident;
    fn to_key_struct_name(&self) -> Ident;
    fn to_store_struct_name(&self) -> Ident;
}
impl CodeGenComponentNames for Component {
    fn to_child_struct_name(&self) -> Ident {
        let span = self.r#type.span();
        format_ident!("__{}Children", self.name, span = span)
    }
    fn to_parent_enum_key(&self) -> Ident {
        let span = self.r#type.span();
        format_ident!("__{}", self.name, span = span)
    }
    fn to_parent_enum(&self) -> Ident {
        let span = self.r#type.span();
        format_ident!("__{}Parent", self.name, span = span)
    }
    fn to_store_struct_name(&self) -> Ident {
        let span = self.r#type.span();
        let mut name: Vec<char> = self.name.to_lowercase().chars().collect();
        name[0] = name[0].to_uppercase().next().unwrap_or(name[0]);
        let name: String = name.into_iter().collect();
        format_ident!("__{}Store", name, span = span)
    }
    fn to_store_name(&self) -> Ident {
        let span = self.r#type.span();
        format_ident!("__{}_store", self.name.to_lowercase(), span = span)
    }
    fn to_key_struct_name(&self) -> Ident {
        let span = self.r#type.span();
        format_ident!("{}Key", self.name, span = span)
    }
}

pub trait CodeGenComponent {
    fn gen_imports(&self) -> TokenStream;
    fn gen_store(&self) -> TokenStream;
    fn gen_new(&self) -> TokenStream;
    fn gen_key(&self) -> TokenStream;
}
impl CodeGenComponent for Component {
    fn gen_imports(&self) -> TokenStream {
        let span = self.r#type.span();
        let path = &self.r#type;

        quote_spanned! {span =>
            #[allow(unused_import)]
            use super::#path;
        }
    }
    fn gen_store(&self) -> TokenStream {
        let span = self.r#type.span();
        let store_name = self.to_store_name();
        let store_struct_name = self.to_store_struct_name();

        quote_spanned! {span =>
            #store_name: #store_struct_name,
        }
    }
    fn gen_new(&self) -> TokenStream {
        let span = self.r#type.span();
        let store_name = self.to_store_name();
        let store_struct_name = self.to_store_struct_name();

        quote_spanned! {span =>
            #store_name: #store_struct_name::new(),
        }
    }
    fn gen_key(&self) -> TokenStream {
        let span = self.r#type.span();
        let key = &self.to_key_struct_name();
        quote_spanned! {span =>
            new_key_type! { pub struct #key; }
            impl KeyExt for super::keys::#key {
                #[inline]
                fn is_some(&self) -> bool { !self.is_none() }
                #[inline]
                fn is_none(&self) -> bool { self.is_null() }
            }
        }
    }
}

trait CodeGenComponentPriv {
    fn gen_store_atom(&self, all: &AllComponents, uniques: &AllUniques) -> TokenStream;
    fn gen_ecs_impl(&self, ecs: &Ident, all: &AllComponents) -> TokenStream;
    fn gen_ecs_purge_impl(&self, ecs: &Ident, all: &AllComponents) -> TokenStream;
    fn gen_parents_impl(&self, parent: &Ident) -> TokenStream;
    fn gen_parents_enum_impl(&self, parent: &Ident, parents: &[TokenStream]) -> TokenStream;
    fn gen_get_parents_impl(&self, parent_key: &Ident, span: Span) -> TokenStream;
}

pub trait CodeGenChild {
    fn to_child_name(&self, all: &AllComponents) -> Ident;
    fn gen_store_entry(&self, all: &AllComponents) -> TokenStream;
    fn gen_new(&self, all: &AllComponents) -> TokenStream;
    fn gen_get_child_impl(
        &self,
        key: &Ident,
        store_name: &Ident,
        all: &AllComponents,
    ) -> TokenStream;
}

impl CodeGenComponentPriv for Component {
    fn gen_parents_impl(&self, parent: &Ident) -> TokenStream {
        let span = parent.span();
        let enum_key: Ident = self.to_parent_enum_key();
        let parent_key = self.to_key_struct_name();

        quote_spanned! {span =>
            impl From<super::keys::#parent_key> for #parent {
                fn from(k: super::keys::#parent_key) -> Self {
                    Self::#enum_key(k)
                }
            }
            impl TryFrom<#parent> for super::keys::#parent_key {
                type Error = ();
                fn try_from(p: #parent) -> Result<Self, Self::Error> {
                    match p {
                        #parent::#enum_key(k) => Ok(k),
                        _ => Err(())
                    }
                }
            }
        }
    }
    fn gen_parents_enum_impl(&self, parent: &Ident, parents: &[TokenStream]) -> TokenStream {
        let span = self.r#type.span();

        if parents.is_empty() {
            quote_spanned! {span => }
        } else {
            quote_spanned! {span =>
                #[doc(hidden)]
                #[derive(Copy,Clone,Eq,PartialEq,Ord,PartialOrd,Hash,Debug)]
                pub(super) enum #parent {
                    None,
                    #(#parents)*
                }
                #[doc(hidden)]
                impl #parent {
                    #[doc(hidden)]
                    #[inline]
                    fn is_none(&self) -> bool {
                        if *self == #parent::None {true} else {false}
                    }
                }

                #[doc(hidden)]
                impl Default for #parent {
                    #[inline]
                    fn default() -> Self {
                        Self::None
                    }
                }
            }
        }
    }
    fn gen_store_atom(&self, all: &AllComponents, uniques: &AllUniques) -> TokenStream {
        let span = self.r#type.span();
        let typ = &self.r#type;
        let key = &self.to_key_struct_name();
        let child_atom_name = self.to_child_struct_name();
        let store_struct_name = self.to_store_struct_name();

        let children: Vec<_> = self
            .children
            .iter()
            .map(|c| c.gen_store_entry(all))
            .collect();

        let children_new: Vec<_> = self.children.iter().map(|c| c.gen_new(all)).collect();

        let children_impl: Vec<_> = self
            .children
            .iter()
            .map(|c| c.gen_get_child_impl(key, &store_struct_name, all))
            .collect();

        let parent = self.to_parent_enum();
        let mut parents: Vec<TokenStream> = all
            .values()
            .filter(|c| c.children.iter().any(|c| c.id == self.id))
            .map(|c| {
                let enum_key: Ident = c.to_parent_enum_key();
                let key = c.to_key_struct_name();

                quote_spanned! {span =>
                    #enum_key(super::keys::#key),
                }
            })
            .collect();
        parents.extend(
            uniques
                .values()
                .filter(|c| c.children.iter().any(|c| c.id == self.id))
                .map(|c| {
                    let enum_key: Ident = c.to_parent_enum_key();
                    let key = c.to_key_struct_name();

                    quote_spanned! {span =>
                        #enum_key(super::keys::#key),
                    }
                })
                .collect::<Vec<TokenStream>>(),
        );

        let parent_enum_impl = self.gen_parents_enum_impl(&parent, &parents);
        let mut parents_impl: Vec<TokenStream> = all
            .values()
            .filter(|c| c.children.iter().any(|c| c.id == self.id))
            .map(|c| c.gen_parents_impl(&parent))
            .collect();
        parents_impl.extend(
            uniques
                .values()
                .filter(|c| c.children.iter().any(|c| c.id == self.id))
                .map(|c| c.gen_parents_impl(&parent))
                .collect::<Vec<TokenStream>>(),
        );

        let mut get_parents_impl: Vec<TokenStream> = all
            .values()
            .filter(|c| c.children.iter().any(|c| c.id == self.id))
            .map(|c| {
                let span = c
                    .children
                    .iter()
                    .find_map(|c| (c.id == self.id).then(|| c.span))
                    .unwrap();
                self.gen_get_parents_impl(&c.to_key_struct_name(), span)
            })
            .collect();
        get_parents_impl.extend(
            uniques
                .values()
                .filter(|c| c.children.iter().any(|c| c.id == self.id))
                .map(|c| {
                    let span = c
                        .children
                        .iter()
                        .find_map(|c| (c.id == self.id).then(|| c.span))
                        .unwrap();
                    self.gen_get_parents_impl(&c.to_key_struct_name(), span)
                })
                .collect::<Vec<TokenStream>>(),
        );

        let parent_enum_type = if parents.is_empty() {
            quote_spanned! {span => }
        } else {
            quote_spanned! {span =>
                __parent: #parent,
            }
        };

        let parent_enum_init = if parents.is_empty() {
            quote_spanned! {span => }
        } else {
            quote_spanned! {span =>
                __parent: #parent::None,
            }
        };

        quote_spanned! {span =>
            #parent_enum_impl

            #[doc(hidden)]
            pub(super) struct #child_atom_name {
                #parent_enum_type
                #(#children)*
            }
            impl #child_atom_name {
                #[inline]
                pub fn new() -> Self {
                    Self {
                        #parent_enum_init
                        #(#children_new)*
                    }
                }
            }
            #[doc(hidden)]
            pub(super) struct #store_struct_name(Store<#typ,#child_atom_name,super::keys::#key>);
            impl #store_struct_name {
                pub fn new() -> Self {
                    Self (
                        Store::new(),
                    )
                }
            }
            impl StoreExBasic<#typ, super::keys::#key> for #store_struct_name {
                fn get(&self, k: super::keys::#key) -> Option<&#typ> {
                    self.0.bin.get(k)
                }
                fn is_empty(&self) -> bool {
                    self.0.bin.is_empty()
                }
            }
            impl StoreExBasicMut<#typ, super::keys::#key> for #store_struct_name {
                fn get_mut(&mut self, k: super::keys::#key) -> Option<&mut #typ> {
                    self.0.bin.get_mut(k)
                }
            }
            impl StoreExCreate<#typ, super::keys::#key> for #store_struct_name {
                fn create(&mut self, t: #typ) -> super::keys::#key {
                    let key = self.0.bin.insert(t);
                    self.0.id.insert(key, #child_atom_name::new());
                    key
                }

                fn remove(&mut self, key: super::keys::#key) -> Option<()> {
                    self.0.id.remove(key);
                    self.0.bin.remove(key).map(|_| ())
                }
            }

            #(#parents_impl)*
            #(#children_impl)*
            #(#get_parents_impl)*
        }
    }
    fn gen_ecs_purge_impl(&self, ecs: &Ident, all: &AllComponents) -> TokenStream {
        let span = self.r#type.span();
        let key = &self.to_key_struct_name();
        let store_name = self.to_store_name();

        let clear_parents: Vec<TokenStream> = all
            .values()
            .filter(|c| c.children.iter().any(|c| c.id == self.id))
            .map(|c| {
                let parent_key = c.to_key_struct_name();

                quote_spanned! {span =>
                    let parent_key: Option<super::keys::#parent_key> = self.get_parent(key);
                    if let Some(parent_key) = parent_key {
                        self.clear_parent(key, parent_key);
                    }
                }
            })
            .collect();

        let clear_children: Vec<TokenStream> = self
            .children
            .iter()
            .map(|c| {
                let child_key = all.get(&c.id).unwrap().to_key_struct_name();

                quote_spanned! {span =>
                    let mut counter = 0;
                    loop {
                        let child: Option<super::keys::#child_key> =
                            self.get_children(key)
                            .map(|k| k.map(|k| *k).nth(counter))
                            .flatten();
                        if let Some(child_key) = child {
                            self.purge(child_key);
                            counter += 1;
                        }
                        else {break;}
                    }
                }
            })
            .collect();

        quote_spanned! {span =>
            impl StoreExPurge<super::keys::#key> for super::#ecs {
                fn purge(&mut self, key: super::keys::#key) {
                    #(#clear_parents)*

                    #(#clear_children)*

                    self.#store_name.remove(key);
                }
            }
        }
    }

    fn gen_ecs_impl(&self, ecs: &Ident, all: &AllComponents) -> TokenStream {
        let span = self.r#type.span();
        let typ = &self.r#type;
        let key = &self.to_key_struct_name();
        let store_name = self.to_store_name();

        let get_child_vec: Vec<_> = self
            .children
            .iter()
            .map(|c| {
                let child_key = all.get(&c.id).unwrap().to_key_struct_name();

                quote_spanned! {span =>
                    impl StoreExGetChild<super::keys::#key, super::keys::#child_key> for super::#ecs {
                        fn get_children(&self, parent: super::keys::#key)
                            -> Option<std::slice::Iter<super::keys::#child_key>> {
                            self.#store_name.get_children(parent)
                        }
                        fn set_child(&mut self, parent: super::keys::#key, child: super::keys::#child_key) -> bool {
                            self.#store_name.set_child(parent, child)
                        }
                        fn clear_child(&mut self, parent: super::keys::#key, child: super::keys::#child_key) -> bool {
                            self.#store_name.clear_child(parent, child)
                        }
                    }
                }
            })
            .collect();

        let parents_impl: Vec<TokenStream> = self
            .children
            .iter()
            .map(|c| {
                let child_key = all.get(&c.id).unwrap().to_key_struct_name();
                let c_store_name = all.get(&c.id).unwrap().to_store_name();

                quote_spanned! {span =>
                    impl StoreExGetParent<super::keys::#child_key, super::keys::#key> for super::#ecs {
                        #[inline]
                        fn get_parent(&self, child: super::keys::#child_key) -> Option<super::keys::#key> {
                            self.#c_store_name.get_parent(child)
                        }
                    }
                    #[doc(hidden)]
                    impl StoreExSetParent<super::keys::#child_key, super::keys::#key> for super::#ecs {
                        #[doc(hidden)]
                        #[inline]
                        fn clear_parent(&mut self, child: super::keys::#child_key, parent: super::keys::#key) -> bool {
                            self.#c_store_name.clear_parent(child, parent)
                        }
                        #[doc(hidden)]
                        #[inline]
                        fn set_parent(&mut self, child: super::keys::#child_key, parent: super::keys::#key) -> bool {
                            self.#c_store_name.set_parent(child, parent)
                        }
                    }
                }
            })
            .collect();

        let purge_impl = self.gen_ecs_purge_impl(ecs, all);

        quote_spanned! {span =>
            impl StoreExBasic<#typ, super::keys::#key> for super::#ecs {
                #[inline]
                fn get(&self, k: super::keys::#key) -> Option<&#typ> {
                    self.#store_name.get(k)
                }
                #[inline]
                fn is_empty(&self) -> bool { self.#store_name.is_empty() }
            }
            impl StoreExBasicMut<#typ, super::keys::#key> for super::#ecs {
                #[inline]
                fn get_mut(&mut self, k: super::keys::#key) -> Option<&mut #typ> {
                    self.#store_name.get_mut(k)
                }
            }
            impl StoreExCreate<#typ, super::keys::#key> for super::#ecs {
                #[inline]
                fn create(&mut self, t: #typ) -> super::keys::#key {
                    self.#store_name.create(t)
                }
                #[inline]
                fn remove(&mut self, key: super::keys::#key) -> Option<()> {
                    self.#store_name.remove(key)
                }
            }

            #(#parents_impl)*
            #(#get_child_vec)*
            #purge_impl
        }
    }
    fn gen_get_parents_impl(&self, parent_key: &Ident, span: Span) -> TokenStream {
        let store_name = self.to_store_struct_name();
        let key = self.to_key_struct_name();

        quote_spanned! {span =>
            impl StoreExGetParent<super::keys::#key, super::keys::#parent_key> for #store_name {
                #[inline]
                fn get_parent(&self, child: super::keys::#key) -> Option<super::keys::#parent_key> {
                    self.0.id.get(child).map(|id| id.__parent.try_into().ok() ).flatten()
                }
            }
            #[doc(hidden)]
            impl StoreExSetParent<super::keys::#key, super::keys::#parent_key> for #store_name {
                #[inline]
                #[doc(hidden)]
                fn clear_parent(&mut self, child: super::keys::#key, parent: super::keys::#parent_key) -> bool {
                    self.0.id.get_mut(child).map(|id| {
                        if id.__parent == parent.into() {
                            id.__parent = Default::default();
                            true
                        }
                        else {false}
                    }).unwrap_or(false)
                }
                #[inline]
                #[doc(hidden)]
                fn set_parent(&mut self, child: super::keys::#key, parent: super::keys::#parent_key) -> bool {
                    self.0.id.get_mut(child).map(|id| {
                        if id.__parent.is_none() {
                            id.__parent = parent.into();
                            true
                        }
                        else {false}
                    }).unwrap_or(false)
                }
            }
        }
    }
}

impl CodeGenChild for Child {
    fn to_child_name(&self, all: &AllComponents) -> Ident {
        let span = self.span;
        let name = &all.get(&self.id).unwrap().name;
        format_ident!("__{}", name.to_lowercase(), span = span)
    }
    fn gen_new(&self, all: &AllComponents) -> TokenStream {
        let span = self.span;
        let name = self.to_child_name(all);

        match self.child_type {
            ChildType::Single => {
                let key = all.get(&self.id).unwrap().to_key_struct_name();
                quote_spanned! {span =>
                    #name: [super::keys::#key::null()],
                }
            }
            ChildType::Array(_) => {
                quote_spanned! {span =>
                    #name: ArrayVec::new(),
                }
            }
            ChildType::Vec => {
                quote_spanned! {span =>
                    #name: Vec::new(),
                }
            }
        }
    }
    fn gen_store_entry(&self, all: &AllComponents) -> TokenStream {
        let span = self.span;
        let name = self.to_child_name(all);
        let key = all.get(&self.id).unwrap().to_key_struct_name();
        let typ = quote_spanned! {span => super::keys::#key};

        match self.child_type {
            ChildType::Single => {
                quote_spanned! {span => #name: [super::keys::#key;1], }
            }
            ChildType::Array(sz) => {
                quote_spanned! {span => #name: ArrayVec::<#typ,#sz>, }
            }
            ChildType::Vec => {
                quote_spanned! {span => #name: Vec<#typ>, }
            }
        }
    }
    fn gen_get_child_impl(
        &self,
        key: &Ident,
        store_name: &Ident,
        all: &AllComponents,
    ) -> TokenStream {
        let span = self.span;
        let child_key = all.get(&self.id).unwrap().to_key_struct_name();
        let cname = self.to_child_name(all);

        let get_child_body = match self.child_type {
            ChildType::Single => quote_spanned! {span =>
                if cidt.#cname[0].is_null() {None} else { Some(cidt.#cname.iter())}
            },
            ChildType::Array(_) | ChildType::Vec => quote_spanned! {span =>
                if cidt.#cname.is_empty() {None} else { Some(cidt.#cname.iter())}
            },
        };

        let set_child_body = match self.child_type {
            ChildType::Single => quote_spanned! { span =>
                if id.#cname[0].is_null() { id.#cname[0] = child; true }
                else { false }
            },
            ChildType::Array(_) => quote_spanned! {span =>
                if id.#cname.contains(&child) || id.#cname.is_full() { false }
                else { id.#cname.push(child); true }
            },
            ChildType::Vec => quote_spanned! {span =>
                if id.#cname.contains(&child) { false }
                else { id.#cname.push(child); true }
            },
        };

        let clear_child_body = match self.child_type {
            ChildType::Single => quote_spanned! {span =>
                if id.#cname[0].is_null() { false }
                else { id.#cname[0] = super::keys::#child_key::null(); true }
            },
            ChildType::Array(_) | ChildType::Vec => quote_spanned! {span =>
                if id.#cname.contains(&_child) {
                    let idx = id.#cname.iter().enumerate().find(|(_i,_k)| **_k == _child).map(|(_i,_k)|_i);
                    if let Some(idx) = idx { id.#cname.swap_remove(idx); true }
                    else {false}
                } else { false }
            },
        };

        quote_spanned! {span =>
            impl StoreExGetChild<super::keys::#key, super::keys::#child_key> for #store_name {
                #[inline]
                fn get_children(&self, parent: super::keys::#key) -> Option<std::slice::Iter<super::keys::#child_key>> {
                    self.0.id.get(parent).map(|cidt|
                        #get_child_body
                    ).flatten()
                }
                #[inline]
                #[doc(hidden)]
                fn set_child(&mut self, parent: super::keys::#key, child: super::keys::#child_key) -> bool {
                    self.0.id.get_mut(parent).map(|id| {
                        #set_child_body
                    }).unwrap_or(false)
                }
                #[inline]
                #[doc(hidden)]
                fn clear_child(&mut self, parent: super::keys::#key, _child: super::keys::#child_key) -> bool {
                    self.0.id.get_mut(parent).map(|id| {
                        #clear_child_body
                    }).unwrap_or(false)
                }
            }
        }
    }
}
