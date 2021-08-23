use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote_spanned};
use syn::Ident;

use crate::codegen::component::{CodeGenChild, CodeGenComponent, CodeGenComponentNames};
use crate::codegen::unique::{CodeGenUnique, CodeGenUniqueNames};
use crate::validation::{
    query::{Atom, Query},
    AllComponents, AllUniques,
};

pub fn gen_mod_queries(ecs: &Ident, all: &AllComponents, uniques: &AllUniques) -> TokenStream {
    let span = ecs.span();

    let component_imports: Vec<TokenStream> = all.values().map(|c| c.gen_imports()).collect();
    let unique_imports: Vec<TokenStream> = unique.values().map(|c| c.gen_imports()).collect();

    let comp_store_atoms: Vec<TokenStream> = all
        .values()
        .map(|c| c.gen_store_atom(all, uniques))
        .collect();

    quote_spanned! {span =>
        mod queries {
            use super::components::*;
            use super::uniques::*;
            use rl_ecs::stores::{StoreExBasic, StoreExBasicMut,
                StoreExCreate,StoreExGetParent,StoreExSetParent,
                StoreExGetChild, StoreExPurge};

            #(#component_imports)*
            #(#unique_imports)*

            // #(#queries_ecs_impls)*
        }
    }
}

pub trait CodeGenQueryNames {
    fn to_query_struct_name(&self) -> Ident;
    fn to_query_member_name(&self) -> Ident;
    fn to_query_atom_name(&self) -> Ident;
}
impl CodeGenQueryNames for Query {
    fn to_query_struct_name(&self) -> Ident {
        let span = self.name.span();
        format_ident!("{}Query", self.name, span = span)
    }
    fn to_query_struct_name(&self) -> Ident {
        let span = self.name.span();
        format_ident!("{}_query", self.name.to_lowercase(), span = span)
    }
    fn to_query_atom_name(&self) -> Ident {
        let span = self.name.span();
        format_ident!("{}Atom", self.name, span = span)
    }
}

trait CodeGenQueryPriv {
    fn gen_query_struct(&self, components: &AllComponents) -> TokenStream;
}
impl CodeGenQueryPriv for Query {
    fn gen_query_struct(&self, components: &AllComponents) -> TokenStream {
        let name = self.to_query_struct_name();

        let types = self
            .children
            .iter()
            .map(|c| components.get(c.id).unwrap().to_key_struct())
            .collect::<Vec<Ident>>();

        quote_spanned! {span =>
            struct #name {
                list: Vec<#(#types)*>,
            }
            impl #name {
                pub fn new() -> Self {
                    Self {
                        list: Vec::new(),
                    }
                }
                pub fn update(&mut self) {

                }
            }
        }
    }
}
