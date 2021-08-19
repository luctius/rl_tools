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
    fn to_parent_enum_key(&self) -> Ident;
    fn to_parent_enum(&self) -> Ident;
    fn to_unique_name(&self) -> Ident;
    fn to_key_name(&self) -> Ident;
    fn gen_store_atom(&self, all: &AllComponents) -> TokenStream;
    fn gen_store(&self) -> TokenStream;
    fn gen_new(&self) -> TokenStream;
    fn gen_unique_new_argument(&self) -> TokenStream;
    fn gen_imports(&self) -> TokenStream;
}

pub trait CodeGenChild {
    fn to_child_name(&self, all: &AllComponents) -> Ident;
    fn gen_store_entry(&self, all: &AllComponents) -> TokenStream;
    fn gen_new(&self, all: &AllComponents) -> TokenStream;
}

impl CodeGenComponent for Component {
    fn to_atom_name(&self) -> Ident {
        let span = self.r#type.span();
        format_ident!("{}Atom", self.name, span = span)
    }
    fn to_parent_enum_key(&self) -> Ident {
        let span = self.r#type.span();
        format_ident!("{}", self.name, span = span)
    }
    fn to_parent_enum(&self) -> Ident {
        let span = self.r#type.span();
        format_ident!("{}Parent", self.name, span = span)
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
        format_ident!("{}", self.name.to_lowercase(), span = span)
    }
    fn to_key_name(&self) -> Ident {
        let span = self.r#type.span();
        format_ident!("{}Key", self.name, span = span)
    }
    fn gen_store_atom(&self, all: &AllComponents) -> TokenStream {
        let span = self.r#type.span();
        let name = self.to_atom_name();
        let key = &self.to_key_name();

        let mut children = Vec::new();
        self.children
            .iter()
            .for_each(|c| children.push(c.gen_store_entry(all)));

        let mut children_new = Vec::new();
        self.children
            .iter()
            .for_each(|c| children_new.push(c.gen_new(all)));

        let (parent_enum, parent_enum_init, parent_enum_new) = if self.unique {
            (
                quote_spanned! {span => },
                quote_spanned! {span => },
                quote_spanned! {span => },
            )
        } else {
            let parent = self.to_parent_enum();
            let parents: Vec<TokenStream> = all
                .values()
                .filter(|c| c.children.iter().any(|c| c.id == self.id))
                .map(|c| {
                    let enum_key: Ident = c.to_parent_enum_key();
                    let key = c.to_key_name();

                    quote_spanned! {span =>
                        #enum_key(#key),
                    }
                })
                .collect();

            (
                quote_spanned! {span =>
                    pub enum #parent {
                        #(#parents)*
                    }
                },
                quote_spanned! {span =>
                    parent: Option<#parent>,
                },
                quote_spanned! {span =>
                    parent: None,
                },
            )
        };
        quote_spanned! {span =>
            new_key_type! { pub struct #key; }
            #parent_enum
            pub(super) struct #name {
                #parent_enum_init
                #(#children)*
            }
            impl #name {
                pub fn new() -> Self {
                    Self {
                        #parent_enum_new
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
    fn gen_store(&self) -> TokenStream {
        let span = self.r#type.span();
        let store_name = self.to_store_name();
        let atom_name = self.to_atom_name();
        let typ = &self.r#type;
        let key = self.to_key_name();

        if self.unique {
            quote_spanned! {span =>
                #store_name: (#typ,#atom_name),
            }
        } else {
            quote_spanned! {span =>
                #store_name: Store<#typ,#atom_name, #key>,
            }
        }
    }
    fn gen_new(&self) -> TokenStream {
        let span = self.r#type.span();
        let store_name = self.to_store_name();

        if self.unique {
            let unique_name = self.to_unique_name();

            quote_spanned! {span =>
                #store_name: (#unique_name,Default::default() ),
            }
        } else {
            quote_spanned! {span =>
                #store_name: Default::default(),
            }
        }
    }
    fn gen_unique_new_argument(&self) -> TokenStream {
        let typ = &self.r#type;
        let span = self.r#type.span();

        if self.unique {
            let unique_name = self.to_unique_name();
            quote_spanned! {span =>
                #unique_name: #typ,
            }
        } else {
            quote_spanned! {span =>
            }
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
}

impl CodeGenChild for Child {
    fn to_child_name(&self, all: &AllComponents) -> Ident {
        let span = self.span;
        let name = &all.get(&self.id).unwrap().name;
        format_ident!("child_{}", name.to_lowercase(), span = span)
    }
    fn gen_new(&self, all: &AllComponents) -> TokenStream {
        let span = self.span;
        let name = self.to_child_name(all);

        quote_spanned! {span =>
            #name: None,
        }
    }
    fn gen_store_entry(&self, all: &AllComponents) -> TokenStream {
        let span = self.span;
        let name = self.to_child_name(all);
        let key = all.get(&self.id).unwrap().to_key_name();
        let typ = quote_spanned! {span => #key};

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
