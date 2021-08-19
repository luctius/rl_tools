use crate::validation::ValidatedEcs;
use proc_macro2::TokenStream;
use quote::{format_ident, quote_spanned};
use syn::Ident;

mod component;
use component::CodeGenComponent;

impl From<ValidatedEcs> for proc_macro::TokenStream {
    fn from(ecs: ValidatedEcs) -> proc_macro::TokenStream {
        let tk: TokenStream = ecs.into();
        proc_macro::TokenStream::from(tk)
    }
}

impl From<ValidatedEcs> for TokenStream {
    fn from(ecs: ValidatedEcs) -> TokenStream {
        let vis = &ecs.visibility;
        let span = ecs.name.span();
        let name = &ecs.name;
        let lname = format_ident!("{}", name.to_string().to_lowercase(), span = span);
        let keyname = ecs.key_name();

        let world_struct = ecs.to_world_struct();

        let mut component_imports = Vec::new();
        ecs.components
            .values()
            .for_each(|c| component_imports.push(c.to_imports()));

        let mut store_atoms = Vec::new();
        ecs.components
            .values()
            .for_each(|c| store_atoms.push(c.to_store_atom(&ecs.components)));

        quote_spanned! {span =>
            pub mod #lname {
                use rl_ecs::{slotmap::new_key_type, stores::Store};

                #(#component_imports)*

                new_key_type! { pub struct #keyname; }

                mod stores {
                    #(#component_imports)*

                    #(#store_atoms)*
                }
                use stores::*;

                #world_struct
            }
            #vis use #lname::#name;
        }
    }
}

trait CodeGenEcsExt {
    fn to_world_struct(&self) -> TokenStream;
    fn key_name(&self) -> Ident;
}

impl CodeGenEcsExt for ValidatedEcs {
    fn to_world_struct(&self) -> TokenStream {
        let name = &self.name;
        let span = self.name.span();

        let mut component_stores = Vec::new();
        self.components
            .values()
            .for_each(|c| component_stores.push(c.to_store(self.key_name())));

        let mut components_new = Vec::new();
        self.components
            .values()
            .for_each(|c| components_new.push(c.to_new()));

        let mut unique_arguments = Vec::new();
        self.components
            .values()
            .for_each(|c| unique_arguments.push(c.to_unique_new_argument()));

        quote_spanned! {span =>
            pub struct #name {
                #(#component_stores)*
            }
            impl #name {
                pub fn new(#(#unique_arguments)*) -> Self {
                    Self {
                        #(#components_new)*
                    }
                }
            }
        }
    }
    fn key_name(&self) -> Ident {
        let name = &self.name;
        format_ident!("{}Key", name)
    }
}
