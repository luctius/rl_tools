use crate::validation::ValidatedEcs;
use proc_macro2::TokenStream;
use quote::{format_ident, quote_spanned};
use syn::Ident;

mod component;
use component::{gen_mod_components, CodeGenComponent};

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

        let component_imports: Vec<TokenStream> =
            ecs.components.values().map(|c| c.gen_imports()).collect();
        let mod_components = gen_mod_components(name, &ecs.components);

        quote_spanned! {span =>
            pub mod #mod_name {
                use rl_ecs::key::KeyExt;
                use rl_ecs::stores::{StoreExBasic,StoreExBasicMut, StoreExCreate,StoreExGetParent,StoreExSetParent,StoreExGetChild, StoreExPurge};

                #(#component_imports)*

                #mod_components
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
