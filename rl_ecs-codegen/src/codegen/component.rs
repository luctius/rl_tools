use proc_macro2::TokenStream;
use quote::{format_ident, quote_spanned};
use syn::{spanned::Spanned, Ident, TypePath};

use crate::validation::{
    component::{Child, ChildType, Component},
    AllComponents,
};

pub trait CodeGenComponent {
    fn to_store_name(&self) -> Ident;
    fn to_atom_name(&self) -> Ident;
    fn to_unique_name(&self) -> Ident;
    fn to_store_atom(&self, all: &AllComponents) -> TokenStream;
    fn to_store(&self, key: Ident) -> TokenStream;
    fn to_new(&self) -> TokenStream;
    fn to_unique_new_argument(&self) -> TokenStream;
    fn to_imports(&self) -> TokenStream;
}

pub trait CodeGenChild {
    fn to_child_name(&self, all: &AllComponents) -> Ident;
    fn to_store_entry(&self, all: &AllComponents) -> TokenStream;
    fn to_new(&self, all: &AllComponents) -> TokenStream;
}

impl CodeGenComponent for Component {
    fn to_atom_name(&self) -> Ident {
        let span = self.r#type.span();
        format_ident!("{}Atom", self.name, span = span)
    }
    fn to_store_name(&self) -> Ident {
        let span = self.r#type.span();
        format_ident!("{}_store", self.name.to_lowercase(), span = span)
    }
    fn to_unique_name(&self) -> Ident {
        if !self.unique {
            panic!(
                "Internal Error: This component is not a unique component!: {}",
                self.name
            );
        }
        let span = self.r#type.span();
        format_ident!("{}_store", self.name.to_lowercase(), span = span)
    }
    fn to_store_atom(&self, all: &AllComponents) -> TokenStream {
        let span = self.r#type.span();
        let name = self.to_atom_name();

        let mut children = Vec::new();
        self.children
            .iter()
            .for_each(|c| children.push(c.to_store_entry(all)));

        let mut children_new = Vec::new();
        self.children
            .iter()
            .for_each(|c| children_new.push(c.to_new(all)));

        quote_spanned! {span =>
            pub struct #name {
                #(#children)*
            }
            impl #name {
                pub fn new() -> Self {
                    Self {
                        #(#children_new)*
                    }
                }
            }
            impl Default for #name {
                fn default() -> Self {
                    Self::new()
                }
            }
        }
    }
    fn to_store(&self, key: Ident) -> TokenStream {
        let span = self.r#type.span();
        let store_name = self.to_store_name();
        let atom_name = self.to_atom_name();
        let typ = &self.r#type;

        if !self.unique {
            quote_spanned! {span =>
                #store_name: Store<#typ,#atom_name, #key>,
            }
        } else {
            quote_spanned! {span =>
                #store_name: (#typ,#atom_name),
            }
        }
    }
    fn to_new(&self) -> TokenStream {
        let span = self.r#type.span();
        let store_name = self.to_store_name();

        if !self.unique {
            quote_spanned! {span =>
                #store_name: Default::default(),
            }
        } else {
            let unique_name = self.to_unique_name();

            quote_spanned! {span =>
                #store_name: (#unique_name,Default::default() ),
            }
        }
    }
    fn to_unique_new_argument(&self) -> TokenStream {
        let typ = &self.r#type;
        let span = self.r#type.span();
    
        if self.unique {
            let unique_name = self.to_unique_name();
            quote_spanned! {span =>
                #unique_name: #typ,
            }
        }
        else {
            quote_spanned! {span =>
            }
        }
    }
    fn to_imports(&self) -> TokenStream {
        let span = self.r#type.span();
        let path = &self.r#type;

        quote_spanned! {span =>
            #[allow(unused_import)]
            use super::#path;
        }
    }
}

impl CodeGenChild for Child {
    fn to_child_name(&self, all: &AllComponents) -> Ident {
        let span = self.span;
        let name = &all.get(&self.id).unwrap().name;
        format_ident!("child_{}", name.to_lowercase(), span = span)
    }
    fn to_new(&self, all: &AllComponents) -> TokenStream {
        let span = self.span;
        let name = self.to_child_name(all);

        quote_spanned! {span =>
            #name: None,
        }
    }
    fn to_store_entry(&self, all: &AllComponents) -> TokenStream {
        let span = self.span;
        let name = self.to_child_name(all);
        let typ = quote_spanned! {span => usize}; //&all.get(&self.id).unwrap().r#type;

        match self.child_type {
            ChildType::Single => {
                quote_spanned! {span =>
                    #name: Option<#typ>,
                }
            }
            ChildType::Array(sz) => {
                quote_spanned! {span =>
                    #name: Option<[#typ;#sz]>,
                }
            }
            ChildType::Vec => {
                quote_spanned! {span =>
                    #name: Option<Vec<#typ>>,
                }
            }
        }
    }
}
