use proc_macro2::TokenStream;
use quote::{format_ident, quote_spanned};
use syn::{spanned::Spanned, Ident, TypePath};

use crate::validation::{
    component::{Child, ChildType, Component},
    AllComponents,
};

pub trait CodeGenComponent {
    fn to_store_name(&self) -> Ident;
    fn to_child_struct_name(&self) -> Ident;
    fn to_parent_enum_key(&self) -> Ident;
    fn to_parent_enum(&self) -> Ident;
    fn to_key_struct_name(&self) -> Ident;
    fn to_store_struct_name(&self) -> Ident;
    fn gen_key(&self) -> TokenStream;
    fn gen_store_atom(&self, all: &AllComponents) -> TokenStream;
    fn gen_store(&self) -> TokenStream;
    fn gen_new(&self) -> TokenStream;
    fn gen_imports(&self) -> TokenStream;
    fn gen_ecs_impl(&self, ecs: &Ident, all: &AllComponents) -> TokenStream;
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

impl CodeGenComponent for Component {
    fn to_child_struct_name(&self) -> Ident {
        let span = self.r#type.span();
        format_ident!("{}Children", self.name, span = span)
    }
    fn to_parent_enum_key(&self) -> Ident {
        let span = self.r#type.span();
        format_ident!("{}", self.name, span = span)
    }
    fn to_parent_enum(&self) -> Ident {
        let span = self.r#type.span();
        format_ident!("{}Parent", self.name, span = span)
    }
    fn to_store_struct_name(&self) -> Ident {
        let span = self.r#type.span();
        let mut name: Vec<char> = self.name.to_lowercase().chars().collect();
        name[0] = name[0].to_uppercase().next().unwrap_or(name[0]);
        let name: String = name.into_iter().collect();
        format_ident!("{}Store", name, span = span)
    }
    fn to_store_name(&self) -> Ident {
        let span = self.r#type.span();
        format_ident!("{}_store", self.name.to_lowercase(), span = span)
    }
    fn to_key_struct_name(&self) -> Ident {
        let span = self.r#type.span();
        format_ident!("{}Key", self.name, span = span)
    }
    fn gen_key(&self) -> TokenStream {
        let span = self.r#type.span();
        let key = &self.to_key_struct_name();
        quote_spanned! {span =>
            new_key_type! { pub struct #key; }
            impl KeyExt for #key {
                #[inline]
                fn is_some(&self) -> bool { !self.is_none() }
                #[inline]
                fn is_none(&self) -> bool { self.is_null() }
            }
        }
    }
    fn gen_store_atom(&self, all: &AllComponents) -> TokenStream {
        let span = self.r#type.span();
        let typ = &self.r#type;
        let key = &self.to_key_struct_name();
        let child_atom_name = self.to_child_struct_name();
        let store_struct_name = self.to_store_struct_name();

        let mut children = Vec::new();
        self.children
            .iter()
            .for_each(|c| children.push(c.gen_store_entry(all)));

        let mut children_new = Vec::new();
        self.children
            .iter()
            .for_each(|c| children_new.push(c.gen_new(all)));

        let mut children_impl = Vec::new();
        self.children
            .iter()
            .for_each(|c| children_impl.push(c.gen_get_child_impl(key, &store_struct_name, all)));

        let parent = self.to_parent_enum();
        let parents: Vec<TokenStream> = all
            .values()
            .filter(|c| c.children.iter().any(|c| c.id == self.id))
            .map(|c| {
                let enum_key: Ident = c.to_parent_enum_key();
                let key = c.to_key_struct_name();

                quote_spanned! {span =>
                    #enum_key(#key),
                }
            })
            .collect();

        let parents_impl: Vec<TokenStream> = all
            .values()
            .filter(|c| c.children.iter().any(|c| c.id == self.id))
            .map(|c| {
                let enum_key: Ident = c.to_parent_enum_key();
                let pkey = c.to_key_struct_name();

                quote_spanned! {span =>
                    impl From<#pkey> for #parent {
                        fn from(k: #pkey) -> Self {
                            Self::#enum_key(k)
                        }
                    }
                    impl TryFrom<#parent> for #pkey {
                        type Error = ();
                        fn try_from(p: #parent) -> Result<Self, Self::Error> {
                            match p {
                                #parent::#enum_key(k) => Ok(k),
                                _ => Err(())
                            }
                        }
                    }
                    impl StoreExGetParent<#key, #pkey> for #store_struct_name {
                        #[inline]
                        fn get_parent(&self, child: #key) -> Option<#pkey> {
                            self.0.id.get(child).map(|id| id.parent.try_into().ok() ).flatten()
                        }
                        #[inline]
                        fn clear_parent(&mut self, child: #key) -> bool {
                            self.0.id.get_mut(child).map(|id| {
                                id.parent = #parent::None;
                            }).is_some()
                        }
                        #[inline]
                        fn set_parent(&mut self, child: #key, parent: #pkey) -> bool {
                            self.0.id.get_mut(child).map(|id| {
                                if !id.parent.is_some() {
                                    id.parent = parent.into();
                                    true
                                }
                                else {false}
                            }).unwrap_or(false)
                        }
                    }
                }
            })
            .collect();

        quote_spanned! {span =>
            #[derive(Copy,Clone,Eq,PartialEq,Ord,PartialOrd,Hash,Debug)]
            pub(super) enum #parent {
                None,
                #(#parents)*
            }

            #[doc(hidden)]
            impl Default for #parent {
                #[inline]
                fn default() -> Self {
                    Self::None
                }
            }
            #[doc(hidden)]
            impl From<KeyData> for #parent {
                // Note: required by Key, but nonsensical in this context
                #[inline]
                fn from(kd: KeyData) -> Self {
                    unimplemented!()
                }
            }
            impl KeyExt for #parent {
                #[inline]
                fn is_some(&self) -> bool { !self.is_none() }
                #[inline]
                fn is_none(&self) -> bool { *self == #parent::None }
            }
            impl Key for #parent {
                #[inline]
                fn data(&self) -> KeyData {
                    todo!()
                }
            }
            pub(super) struct #child_atom_name {
                parent: #parent,
                #(#children)*
            }
            impl #child_atom_name {
                #[inline]
                pub fn new() -> Self {
                    Self {
                        parent: #parent::None,
                        #(#children_new)*
                    }
                }
            }
            pub(super) struct #store_struct_name(Store<#typ,#child_atom_name,#key>);
            impl #store_struct_name {
                pub fn new() -> Self {
                    Self (
                        Store::new(),
                    )
                }
            }
            impl StoreExBasic<#typ, #key> for #store_struct_name {
                fn get(&self, k: #key) -> Option<&#typ> {
                    self.0.bin.get(k)
                }

                fn get_mut(&mut self, k: #key) -> Option<&mut #typ> {
                    self.0.bin.get_mut(k)
                }

                fn is_empty(&self) -> bool {
                    self.0.bin.is_empty()
                }
            }
            impl StoreExCreate<#typ, #key> for #store_struct_name {
                fn create(&mut self, t: #typ) -> #key {
                    let key = self.0.bin.insert(t);
                    self.0.id.insert(key, #child_atom_name::new());
                    key
                }

                fn remove(&mut self, key: #key) {
                    self.0.id.remove(key);
                    self.0.bin.remove(key);
                }
            }

            #(#parents_impl)*
            #(#children_impl)*
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
    fn gen_imports(&self) -> TokenStream {
        let span = self.r#type.span();
        let path = &self.r#type;

        quote_spanned! {span =>
            #[allow(unused_import)]
            use super::#path;
        }
    }
    fn gen_ecs_impl(&self, ecs: &Ident, all: &AllComponents) -> TokenStream {
        let span = self.r#type.span();
        let typ = &self.r#type;
        let key = &self.to_key_struct_name();
        let child_atom_name = self.to_child_struct_name();
        let store_name = self.to_store_name();
        let store_struct_name = self.to_store_struct_name();
        let parent = self.to_parent_enum();

        let mut get_child_vec = Vec::with_capacity(self.children.len());
        self.children.iter().for_each(|c| {
            get_child_vec.push({
                let ckey = all.get(&c.id).unwrap().to_key_struct_name();

                quote_spanned! {span =>
                    impl StoreExGetChild<#key, #ckey> for super::#ecs {
                        fn get_child(&self, parent: #key) -> Option<std::slice::Iter<#ckey>> {
                            self.#store_name.get_child(parent)
                        }
                        fn set_child(&mut self, parent: #key, child: #ckey) -> bool {
                            self.#store_name.set_child(parent, child)
                        }
                        fn clear_child(&mut self, parent: #key, child: #ckey) -> bool {
                            self.#store_name.clear_child(parent, child)
                        }
                    }
                }
            })
        });

        let parents_impl: Vec<TokenStream> = all
            .values()
            .filter(|c| c.children.iter().any(|c| c.id == self.id))
            .map(|c| {
                let enum_key: Ident = c.to_parent_enum_key();
                let pkey = c.to_key_struct_name();

                quote_spanned! {span =>
                    impl StoreExGetParent<#key, #pkey> for super::#ecs {
                        #[inline]
                        fn get_parent(&self, child: #key) -> Option<#pkey> {
                            self.#store_name.get_parent(child)
                        }
                        #[inline]
                        fn clear_parent(&mut self, child: #key) -> bool {
                            // self.#store_name.clear_parent(child)
                            todo!()
                        }
                        #[inline]
                        fn set_parent(&mut self, child: #key, parent: #pkey) -> bool {
                            self.#store_name.set_parent(child, parent)
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
                fn get_mut(&mut self, k: #key) -> Option<&mut #typ> {
                    self.#store_name.get_mut(k)
                }
                #[inline]
                fn is_empty(&self) -> bool { self.#store_name.is_empty() }
            }

            impl StoreExCreate<#typ, #key> for super::#ecs {
                #[inline]
                fn create(&mut self, t: #typ) -> #key {
                    self.#store_name.create(t)
                }
                #[inline]
                fn remove(&mut self, key: #key) {
                    self.#store_name.remove(key)
                }
            }

            #(#parents_impl)*
            #(#get_child_vec)*
        }
    }
}

impl CodeGenChild for Child {
    fn to_child_name(&self, all: &AllComponents) -> Ident {
        let span = self.span;
        let name = &all.get(&self.id).unwrap().name;
        format_ident!("{}", name.to_lowercase(), span = span)
    }
    fn gen_new(&self, all: &AllComponents) -> TokenStream {
        let span = self.span;
        let name = self.to_child_name(all);
        let key = all.get(&self.id).unwrap().to_key_struct_name();

        match self.child_type {
            ChildType::Array(sz) => {
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
        let typ = quote_spanned! {span => #key};

        match self.child_type {
            ChildType::Array(sz) => {
                quote_spanned! {span =>
                    #name: ArrayVec::<#typ,#sz>,
                }
            }
            ChildType::Vec => {
                quote_spanned! {span =>
                    #name: Vec<#typ>,
                }
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
        let ckey = all.get(&self.id).unwrap().to_key_struct_name();
        let cname = self.to_child_name(all);

        quote_spanned! {span =>
            impl StoreExGetChild<#key, #ckey> for #store_name
                where #key: Key + KeyExt,
                      #ckey: Key + KeyExt, {
                fn get_child(&self, parent: #key) -> Option<std::slice::Iter<#ckey>> {
                    self.0.id.get(parent).map(|cidt| if cidt.#cname.is_empty() {None} else { Some(cidt.#cname.iter())} ).flatten()
                }
                fn set_child(&mut self, parent: #key, child: #ckey) -> bool {
                    let id_store = &mut self.0.id;
                    self.0.id.get_mut(parent).map(|id| {
                        if id.#cname.contains(&child) {
                            false
                        }
                        else {
                            id.#cname.push(child);
                            true
                        }
                    }).unwrap_or(false)
                }
                fn clear_child(&mut self, parent: #key, child: #ckey) -> bool {
                    let id_store = &mut self.0.id;
                    self.0.id.get_mut(parent).map(|id| {
                        if id.#cname.contains(&child) {
                            let idx = id.#cname.iter().enumerate().find(|(i,k)| **k == child).map(|(i,k)|i);
                            if let Some(idx) = idx {
                                id.#cname.swap_remove(idx);
                                true
                            }
                            else {false}
                        }
                        else {
                            false
                        }
                    }).unwrap_or(false)
                }
            }
        }
    }
}
