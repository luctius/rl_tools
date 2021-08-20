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
        let mod_name = format_ident!("{}", name.to_string().to_lowercase(), span = span);

        let world_struct = ecs.gen_world_struct();

        let mut component_imports = Vec::with_capacity(ecs.components.len());
        ecs.components
            .values()
            .for_each(|c| component_imports.push(c.gen_imports()));

        let mut store_atoms = Vec::with_capacity(ecs.components.len());
        ecs.components
            .values()
            .for_each(|c| store_atoms.push(c.gen_store_atom(&ecs.components)));

        let mut keys = Vec::with_capacity(ecs.components.len());
        ecs.components.values().for_each(|c| keys.push(c.gen_key()));

        let mut ecs_impls = Vec::with_capacity(ecs.components.len());
        ecs.components
            .values()
            .for_each(|c| ecs_impls.push(c.gen_ecs_impl(name, &ecs.components)));

        quote_spanned! {span =>
            pub mod #mod_name {
                use rl_ecs::key::KeyExt;
                use rl_ecs::stores::{StoreExBasic, StoreExCreate,StoreExGetParent,StoreExGetChild};
                use components::InventoryKey;

                #(#component_imports)*

                mod components {
                    use core::convert::{TryFrom, TryInto};
                    use rl_ecs::key::KeyExt;
                    use rl_ecs::stores::Store;
                    use rl_ecs::stores::{StoreExBasic, StoreExCreate,StoreExGetParent,StoreExGetChild};
                    use rl_ecs::slotmap::{new_key_type, Key, KeyData};
                    use rl_ecs::arrayvec::{ArrayVec};

                    #(#keys)*

                    #(#component_imports)*
                    #(#store_atoms)*

                    #(#ecs_impls)*
                }
                pub use components::*;

                #world_struct
            }
            #vis use #mod_name::#name;
        }
    }
}

trait CodeGenEcsExt {
    fn gen_world_struct(&self) -> TokenStream;
}

impl CodeGenEcsExt for ValidatedEcs {
    fn gen_world_struct(&self) -> TokenStream {
        let name = &self.name;
        let span = self.name.span();

        let mut component_stores = Vec::new();
        self.components
            .values()
            .for_each(|c| component_stores.push(c.gen_store()));

        let mut components_new = Vec::new();
        self.components
            .values()
            .for_each(|c| components_new.push(c.gen_new()));

        quote_spanned! {span =>
            pub struct #name {
                #(#component_stores)*
            }
            impl #name {
                #[must_use]
                pub fn new() -> Self {
                    Self {
                        #(#components_new)*
                    }
                }
            }
        }
    }
}
