use crate::validation::ValidatedEcs;
use proc_macro2::TokenStream;
use quote::{format_ident, quote_spanned};
use syn::Ident;

mod component;
mod unique;
use component::{gen_mod_components, CodeGenComponent};
use unique::{gen_mod_uniques, CodeGenUnique};

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
        let unique_imports: Vec<TokenStream> =
            ecs.uniques.values().map(|c| c.gen_imports()).collect();
        let mod_components = gen_mod_components(name, &ecs.components, &ecs.uniques);
        let mod_unique = gen_mod_uniques(name, &ecs.components, &ecs.uniques);

        let comp_keys: Vec<TokenStream> = ecs.components.values().map(|c| c.gen_key()).collect();
        let unique_keys: Vec<TokenStream> = ecs.uniques.values().map(|c| c.gen_key()).collect();

        quote_spanned! {span =>
            pub mod #mod_name {
                use rl_ecs::key::KeyExt;
                use rl_ecs::stores::{StoreExBasic,StoreExBasicMut, StoreExCreate,StoreExGetParent,StoreExSetParent,StoreExGetChild, StoreExPurge};

                pub mod keys {
                    use rl_ecs::{{key::KeyExt}, stores::UniqueStoreKey};
                    use rl_ecs::slotmap::{new_key_type, Key};
                    #(#comp_keys)*
                    #(#unique_keys)*
                }

                #(#component_imports)*
                #(#unique_imports)*

                #mod_components
                #mod_unique
                pub use components::*;
                pub use unique::*;

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

        let component_stores: Vec<_> = self.components
            .values()
            .map(|c| c.gen_store()).collect();
        let unique_stores: Vec<_> = self.uniques
            .values()
            .map(|c| c.gen_store()).collect();

        let components_new: Vec<_> = self.components
            .values()
            .map(|c| c.gen_new()).collect();
        let uniques_new: Vec<_> = self.uniques
            .values()
            .map(|c| c.gen_new()).collect();
        
        let uniques_new_args: Vec<_> = self.uniques
            .values()
            .map(|c| c.gen_new_args()).collect();

        quote_spanned! {span =>
            pub struct #name {
                #(#component_stores)*
                #(#unique_stores)*
            }
            impl #name {
                #[must_use]
                pub fn new(#(#uniques_new_args)*) -> Self {
                    Self {
                        #(#components_new)*
                        #(#uniques_new)*
                    }
                }
            }
        }
    }
}
