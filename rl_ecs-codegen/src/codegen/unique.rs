use proc_macro2::TokenStream;
use quote::{format_ident, quote_spanned};
use syn::{spanned::Spanned, Ident};

use crate::codegen::component::{CodeGenChild, CodeGenComponent, CodeGenComponentNames};
use crate::validation::{
    component::{Child, ChildType, Component},
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
            use rl_ecs::key::KeyExt;
            use rl_ecs::stores::{StoreExBasic, StoreExBasicMut,
                StoreExGetChild, StoreExPurge,StoreExGetParent,
                StoreExSetParent,UniqueStore};
            use rl_ecs::slotmap::{Key};
            use rl_ecs::arrayvec::{ArrayVec};
            use super::keys::*;

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
            impl KeyExt for #key {
                #[inline]
                fn is_some(&self) -> bool { !self.is_none() }
                #[inline]
                fn is_none(&self) -> bool { self.is_null() }
            }
            impl UniqueStoreKey<#key> for super::#typ {
                fn unique_key() -> #key {
                    #key::null()
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
            .map(|c| CodeGenUniqueChild::gen_get_child_impl(c, &key, &store_struct_name, all))
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
                pub fn new(s: #typ) -> Self {
                    Self (s, #child_atom_name::new() )
                }
            }
            impl StoreExBasic<#typ, #key> for #store_struct_name {
                fn get(&self, k: #key) -> Option<&#typ> {
                    Some(&self.0)
                }
                fn is_empty(&self) -> bool {
                    false
                }
            }
            impl StoreExBasicMut<#typ, #key> for #store_struct_name {
                fn get_mut(&mut self, k: #key) -> Option<&mut #typ> {
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
                    impl StoreExGetChild<#key, #child_key> for super::#ecs {
                        fn get_child(&self, parent: #key) -> Option<std::slice::Iter<#child_key>> {
                            self.#store_name.get_child(parent)
                        }
                        fn set_child(&mut self, parent: #key, child: #child_key) -> bool {
                            self.#store_name.set_child(parent, child)
                        }
                        fn clear_child(&mut self, parent: #key, child: #child_key) -> bool {
                            self.#store_name.clear_child(parent, child)
                        }
                    }
                }
            })
            .collect();

        quote_spanned! {span =>
            impl StoreExBasic<#typ, #key> for super::#ecs {
                #[inline]
                fn get(&self, k: #key) -> Option<&#typ> {
                    self.#store_name.get(k)
                }
                #[inline]
                fn is_empty(&self) -> bool { self.#store_name.is_empty() }
            }
            impl StoreExBasicMut<#typ, #key> for super::#ecs {
                #[inline]
                fn get_mut(&mut self, k: #key) -> Option<&mut #typ> {
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
        let c_store_name = all.get(&self.id).unwrap().to_store_struct_name();

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
                else { id.#cname[0] = #child_key::null(); true }
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
            impl StoreExGetChild<#key, #child_key> for #store_name
                where #key: Key + KeyExt,
                      #child_key: Key + KeyExt, {
                #[inline]
                fn get_child(&self, parent: #key) -> Option<std::slice::Iter<#child_key>> {
                    let id = &self.1;
                    #get_child_body
                }
                #[inline]
                #[doc(hidden)]
                fn set_child(&mut self, parent: #key, child: #child_key) -> bool {
                    let id = &mut self.1;
                    #set_child_body
                }
                #[inline]
                #[doc(hidden)]
                fn clear_child(&mut self, parent: #key, child: #child_key) -> bool {
                    let id = &mut self.1;
                    #clear_child_body
                }
            }
            // impl StoreExGetParent<#child_key, #key> for super::components::#c_store_name {
            //     #[inline]
            //     fn get_parent(&self, child: #child_key) -> Option<#key> {
            //         self.0.id.get(child).map(|id| id.__parent.try_into().ok() ).flatten()
            //     }
            // }
            // #[doc(hidden)]
            // impl StoreExSetParent<#child_key, #key> for super::components::#c_store_name {
            //     #[inline]
            //     #[doc(hidden)]
            //     fn clear_parent(&mut self, child: #child_key, parent: #key) -> bool {
            //         self.0.id.get_mut(child).map(|id| {
            //             if id.__parent == parent.into() {
            //                 id.__parent = Default::default();
            //                 true
            //             }
            //             else {false}
            //         }).unwrap_or(false)
            //     }
            //     #[inline]
            //     #[doc(hidden)]
            //     fn set_parent(&mut self, child: #child_key, parent: #key) -> bool {
            //         self.0.id.get_mut(child).map(|id| {
            //             if id.__parent.is_none() {
            //                 id.__parent = parent.into();
            //                 true
            //             }
            //             else {false}
            //         }).unwrap_or(false)
            //     }
            // }
        }
    }
}
