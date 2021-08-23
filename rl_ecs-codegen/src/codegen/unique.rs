use proc_macro2::TokenStream;
use quote::{format_ident, quote_spanned};
use syn::{spanned::Spanned, Ident};

use crate::codegen::component::{CodeGenChild, CodeGenComponent, CodeGenComponentNames};
use crate::validation::{
    component::{Child, ChildType},
    unique::Unique,
    AllComponents, AllUniques,
};

pub fn gen_mod_uniques(ecs: &Ident, all: &AllComponents, unique: &AllUniques) -> TokenStream {
    let span = ecs.span();

    let component_imports: Vec<TokenStream> = all.values().map(|c| c.gen_imports()).collect();
    let unique_imports: Vec<TokenStream> = unique.values().map(|c| c.gen_imports()).collect();

    let unique_store_atoms: Vec<TokenStream> =
        unique.values().map(|c| c.gen_store_atom(all)).collect();

    let unique_ecs_impls: Vec<TokenStream> =
        unique.values().map(|c| c.gen_ecs_impl(ecs, all)).collect();

    quote_spanned! {span =>
        mod unique {
            use core::convert::{TryFrom, TryInto};
            use rl_ecs::stores::{StoreExBasic, StoreExBasicMut,
                StoreExGetChild, StoreExPurge,StoreExGetParent,
                StoreExSetParent,UniqueStore};
            use rl_ecs::slotmap::{Key};
            use rl_ecs::arrayvec::{ArrayVec};

            #(#component_imports)*
            #(#unique_imports)*
            #(#unique_store_atoms)*

            #(#unique_ecs_impls)*
        }
    }
}

pub trait CodeGenUniqueNames {
    fn to_type_name(&self) -> Ident;
    fn to_store_name(&self) -> Ident;
    fn to_child_struct_name(&self) -> Ident;
    fn to_store_struct_name(&self) -> Ident;
    fn to_key_struct_name(&self) -> Ident;
    fn to_parent_enum_key(&self) -> Ident;
}
impl CodeGenUniqueNames for Unique {
    fn to_type_name(&self) -> Ident {
        let span = self.r#type.span();
        format_ident!("{}", self.name.to_lowercase(), span = span)
    }
    fn to_child_struct_name(&self) -> Ident {
        let span = self.r#type.span();
        format_ident!("__{}Children", self.name, span = span)
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
    fn to_parent_enum_key(&self) -> Ident {
        let span = self.r#type.span();
        format_ident!("__{}", self.name, span = span)
    }
}

pub trait CodeGenUnique {
    fn gen_imports(&self) -> TokenStream;
    fn gen_store(&self) -> TokenStream;
    fn gen_new(&self) -> TokenStream;
    fn gen_key(&self) -> TokenStream;
    fn gen_new_args(&self) -> TokenStream;
    fn gen_parents_impl(&self, parent: &Ident) -> TokenStream;
}
impl CodeGenUnique for Unique {
    fn gen_imports(&self) -> TokenStream {
        let span = self.r#type.span();
        let path = &self.r#type;

        quote_spanned! {span =>
            #[allow(unused_import)]
            use super::#path;
        }
    }
    fn gen_new_args(&self) -> TokenStream {
        let span = self.r#type.span();
        let typ = &self.r#type;
        let name = self.to_type_name();

        quote_spanned! {span =>
            #name: #typ,
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
        let type_name = self.to_type_name();

        quote_spanned! {span =>
            #store_name: #store_struct_name::new(#type_name),
        }
    }
    fn gen_key(&self) -> TokenStream {
        let span = self.r#type.span();
        let typ = &self.r#type;
        let key = &self.to_key_struct_name();
        quote_spanned! {span =>
            new_key_type! { pub struct #key; }
            impl KeyExt for super::keys::#key {
                #[inline]
                fn is_some(&self) -> bool { !self.is_none() }
                #[inline]
                fn is_none(&self) -> bool { self.is_null() }
            }
            impl UniqueStoreKey<super::keys::#key> for super::#typ {
                fn unique_key() -> super::keys::#key {
                    super::keys::#key::null()
                }
            }
        }
    }
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
}

trait CodeGenUniquePriv {
    fn gen_store_atom(&self, all: &AllComponents) -> TokenStream;
    fn gen_ecs_impl(&self, ecs: &Ident, all: &AllComponents) -> TokenStream;
}

impl CodeGenUniquePriv for Unique {
    fn gen_store_atom(&self, all: &AllComponents) -> TokenStream {
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
            .map(|c| CodeGenUniqueChild::gen_get_child_impl(c, key, &store_struct_name, all))
            .collect();

        quote_spanned! {span =>
            #[doc(hidden)]
            pub(super) struct #child_atom_name {
                #(#children)*
            }
            impl #child_atom_name {
                #[inline]
                pub fn new() -> Self {
                    Self {
                        #(#children_new)*
                    }
                }
            }
            #[doc(hidden)]
            pub(super) struct #store_struct_name(#typ,#child_atom_name);
            impl #store_struct_name {
                pub fn new(store: #typ) -> Self {
                    Self (store, #child_atom_name::new() )
                }
            }
            impl StoreExBasic<#typ, super::keys::#key> for #store_struct_name {
                fn get(&self, _k: super::keys::#key) -> Option<&#typ> {
                    Some(&self.0)
                }
                fn is_empty(&self) -> bool {
                    false
                }
            }
            impl StoreExBasicMut<#typ, super::keys::#key> for #store_struct_name {
                fn get_mut(&mut self, _k: super::keys::#key) -> Option<&mut #typ> {
                    Some(&mut self.0)
                }
            }

            #(#children_impl)*
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
                        fn set_child(&mut self, parent: super::keys::#key, child: super::keys::#child_key)
                                -> bool {
                            self.#store_name.set_child(parent, child)
                        }
                        fn clear_child(&mut self, parent: super::keys::#key, child: super::keys::#child_key)
                                -> bool {
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
                        fn get_parent(&self, child: super::keys::#child_key)
                                -> Option<super::keys::#key> {
                            self.#c_store_name.get_parent(child)
                        }
                    }
                    #[doc(hidden)]
                    impl StoreExSetParent<super::keys::#child_key, super::keys::#key> for super::#ecs {
                        #[doc(hidden)]
                        #[inline]
                        fn clear_parent(&mut self, child: super::keys::#child_key, parent: super::keys::#key)
                                -> bool {
                            self.#c_store_name.clear_parent(child, parent)
                        }
                        #[doc(hidden)]
                        #[inline]
                        fn set_parent(&mut self, child: super::keys::#child_key, parent: super::keys::#key)
                                -> bool {
                            self.#c_store_name.set_parent(child, parent)
                        }
                    }
                }
            })
            .collect();

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
            impl UniqueStore<#typ> for super::#ecs {
                fn get_unique(&self) -> &#typ {
                    &self.#store_name.0
                }
                fn get_unique_mut(&mut self) -> &mut #typ {
                    &mut self.#store_name.0
                }
            }

            #(#parents_impl)*
            #(#get_child_vec)*
        }
    }
}

trait CodeGenUniqueChild {
    fn gen_get_child_impl(
        &self,
        key: &Ident,
        store_name: &Ident,
        all: &AllComponents,
    ) -> TokenStream;
}

impl CodeGenUniqueChild for Child {
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
                if id.#cname[0].is_null() {None} else { Some(id.#cname.iter())}
            },
            ChildType::Array(_) | ChildType::Vec => quote_spanned! {span =>
                if id.#cname.is_empty() {None} else { Some(id.#cname.iter())}
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
                if id.#cname.contains(&child) {
                    let idx = id.#cname.iter().enumerate().find(|(i,k)| **k == child).map(|(i,k)|i);
                    if let Some(idx) = idx { id.#cname.swap_remove(idx); true }
                    else {false}
                } else { false }
            },
        };

        quote_spanned! {span =>
            impl StoreExGetChild<super::keys::#key, super::keys::#child_key> for #store_name {
                #[inline]
                fn get_children(&self, _parent: super::keys::#key)
                    -> Option<std::slice::Iter<super::keys::#child_key>> {
                    let id = &self.1;
                    #get_child_body
                }
                #[inline]
                #[doc(hidden)]
                fn set_child(&mut self, _parent: super::keys::#key, child: super::keys::#child_key)
                        -> bool {
                    let id = &mut self.1;
                    #set_child_body
                }
                #[inline]
                #[doc(hidden)]
                fn clear_child(&mut self, _parent: super::keys::#key, _child: super::keys::#child_key)
                        -> bool {
                    let id = &mut self.1;
                    #clear_child_body
                }
            }
        }
    }
}
